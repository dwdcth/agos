use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::core::db::Database;

fn unique_temp_dir(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-status-tests")
        .join(format!("{name}-{unique}"))
}

fn write_config(path: &Path, db_path: &Path, mode: &str, backend: &str) {
    let parent = path.parent().expect("config path should have parent");
    fs::create_dir_all(parent).expect("config parent should exist");
    fs::write(
        path,
        format!(
            r#"
db_path = "{}"

[retrieval]
mode = "{mode}"

[embedding]
backend = "{backend}"
"#,
            db_path.display()
        ),
    )
    .expect("config should be written");
}

fn run_cli(config_path: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_agent-memos"))
        .arg("--config")
        .arg(config_path)
        .args(args)
        .output()
        .expect("binary should run")
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf-8")
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be utf-8")
}

#[test]
fn status_exits_successfully_for_reserved_modes() {
    for (name, mode, backend, ready) in [
        ("lexical", "lexical_only", "disabled", "true"),
        ("embedding", "embedding_only", "reserved", "false"),
        ("hybrid", "hybrid", "reserved", "false"),
    ] {
        let dir = unique_temp_dir(name);
        let db_path = dir.join("agent-memos.sqlite");
        let config_path = dir.join("config.toml");
        Database::open(&db_path).expect("database should bootstrap for status");
        write_config(&config_path, &db_path, mode, backend);

        let output = run_cli(&config_path, &["status"]);
        let text = stdout(&output);

        assert!(
            output.status.success(),
            "status should succeed for mode {mode}: stderr={}",
            stderr(&output)
        );
        assert!(
            text.contains(&format!("configured_mode: {mode}")),
            "status should report configured_mode for {mode}: {text}"
        );
        assert!(
            text.contains(&format!("effective_mode: {mode}")),
            "status should report effective_mode for {mode}: {text}"
        );
        assert!(
            text.contains("lexical_dependency_state:"),
            "status should include lexical dependency state: {text}"
        );
        assert!(
            text.contains("index_readiness:"),
            "status should include index readiness: {text}"
        );
        assert!(
            text.contains(&format!("ready: {ready}")),
            "status should report ready={ready} for {mode}: {text}"
        );
    }
}

#[test]
fn doctor_blocks_invalid_mode_backend_combinations() {
    for (name, mode, backend, expected) in [
        (
            "embedding-disabled",
            "embedding_only",
            "disabled",
            "embedding_only requires a non-disabled embedding backend",
        ),
        (
            "hybrid-disabled",
            "hybrid",
            "disabled",
            "hybrid requires an embedding backend for the secondary path",
        ),
    ] {
        let dir = unique_temp_dir(name);
        let db_path = dir.join("agent-memos.sqlite");
        let config_path = dir.join("config.toml");
        Database::open(&db_path).expect("database should bootstrap for doctor");
        write_config(&config_path, &db_path, mode, backend);

        let output = run_cli(&config_path, &["doctor"]);
        let text = stdout(&output);

        assert!(
            !output.status.success(),
            "doctor should fail for invalid combination {mode}/{backend}: stdout={text} stderr={}",
            stderr(&output)
        );
        assert!(
            text.contains(expected),
            "doctor should explain the failure for {mode}/{backend}: {text}"
        );
    }
}
