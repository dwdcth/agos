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
    for (name, mode, backend, ready, lexical_state, embedding_state, index_state) in [
        (
            "lexical",
            "lexical_only",
            "disabled",
            "true",
            "ready",
            "disabled",
            "ready",
        ),
        (
            "embedding",
            "embedding_only",
            "reserved",
            "false",
            "not_applicable",
            "deferred",
            "not_applicable",
        ),
        (
            "hybrid",
            "hybrid",
            "reserved",
            "false",
            "ready",
            "deferred",
            "deferred",
        ),
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
            text.contains(&format!("lexical_dependency_state: {lexical_state}")),
            "status should include lexical dependency state for {mode}: {text}"
        );
        assert!(
            text.contains(&format!("embedding_dependency_state: {embedding_state}")),
            "status should include embedding dependency state for {mode}: {text}"
        );
        assert!(
            text.contains(&format!("index_readiness: {index_state}")),
            "status should include index readiness for {mode}: {text}"
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

#[test]
fn init_creates_database_and_inspect_schema_reports_foundation_state() {
    let dir = unique_temp_dir("init-inspect");
    let db_path = dir.join("data").join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path, "lexical_only", "disabled");

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "init should succeed for a valid lexical_only setup: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );
    assert!(
        db_path.exists(),
        "init should create the sqlite database at {}",
        db_path.display()
    );

    let inspect_output = run_cli(&config_path, &["inspect", "schema"]);
    let inspect_text = stdout(&inspect_output);
    assert!(
        inspect_output.status.success(),
        "inspect schema should succeed after init: stdout={inspect_text} stderr={}",
        stderr(&inspect_output)
    );
    assert!(
        inspect_text.contains("schema_version: 3"),
        "inspect schema should report schema_version: {inspect_text}"
    );
    assert!(
        inspect_text.contains("base_table_state: ready"),
        "inspect schema should report base table readiness: {inspect_text}"
    );
    assert!(
        inspect_text.contains("index_readiness: ready"),
        "inspect schema should report ready lexical indexes after phase 2 bootstrap: {inspect_text}"
    );
}

#[test]
fn status_reports_non_sqlite_db_as_not_ready() {
    let dir = unique_temp_dir("status-bad-db");
    let db_path = dir.join("data").join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    fs::create_dir_all(db_path.parent().expect("db path should have parent"))
        .expect("db parent should exist");
    fs::write(&db_path, b"not a sqlite database").expect("bad db fixture should be written");
    write_config(&config_path, &db_path, "embedding_only", "reserved");

    let output = run_cli(&config_path, &["status"]);
    let text = stdout(&output);

    assert!(
        output.status.success(),
        "status should stay informational for a non-sqlite file: stdout={text} stderr={}",
        stderr(&output)
    );
    assert!(
        text.contains("configured_mode: embedding_only"),
        "status should preserve configured mode: {text}"
    );
    assert!(
        text.contains("effective_mode: embedding_only"),
        "status should preserve effective mode: {text}"
    );
    assert!(
        text.contains("schema_state: missing"),
        "status should report explicit non-ready schema state: {text}"
    );
    assert!(
        text.contains("base_table_state: missing"),
        "status should report explicit non-ready base table state: {text}"
    );
    assert!(
        text.contains("lexical_dependency_state: not_applicable"),
        "status should keep lexical dependency state mode-aware for embedding_only: {text}"
    );
    assert!(
        text.contains("index_readiness: not_applicable"),
        "status should still report index readiness for embedding_only: {text}"
    );
    assert!(
        text.contains("ready: false"),
        "status should report not ready for a bad db file: {text}"
    );
    assert!(
        text.contains("schema inspection failed for existing database file"),
        "status should explain inspection failure in notes: {text}"
    );
}

#[test]
fn init_output_is_truthful_after_successful_bootstrap() {
    let dir = unique_temp_dir("init-truthful-output");
    let db_path = dir.join("data").join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path, "lexical_only", "disabled");

    let output = run_cli(&config_path, &["init"]);
    let text = stdout(&output);

    assert!(
        output.status.success(),
        "init should succeed for a valid lexical_only setup: stdout={text} stderr={}",
        stderr(&output)
    );
    assert!(
        text.contains("initialized: true"),
        "init should confirm initialization: {text}"
    );
    assert!(
        text.contains("schema_version: 3"),
        "init should report the post-bootstrap schema version: {text}"
    );
    assert!(
        !text.contains("database schema is not initialized yet"),
        "init output should not include stale pre-init schema warnings: {text}"
    );
}

#[test]
fn init_allows_reserved_modes_but_rejects_invalid_runtime_requests() {
    let reserved_dir = unique_temp_dir("init-reserved");
    let reserved_db_path = reserved_dir.join("data").join("agent-memos.sqlite");
    let reserved_config_path = reserved_dir.join("config.toml");
    write_config(
        &reserved_config_path,
        &reserved_db_path,
        "embedding_only",
        "reserved",
    );

    let reserved_output = run_cli(&reserved_config_path, &["init"]);
    assert!(
        reserved_output.status.success(),
        "init should stay informational for reserved embedding_only mode: stdout={} stderr={}",
        stdout(&reserved_output),
        stderr(&reserved_output)
    );

    let invalid_dir = unique_temp_dir("init-invalid");
    let invalid_db_path = invalid_dir.join("data").join("agent-memos.sqlite");
    let invalid_config_path = invalid_dir.join("config.toml");
    write_config(
        &invalid_config_path,
        &invalid_db_path,
        "embedding_only",
        "disabled",
    );

    let invalid_output = run_cli(&invalid_config_path, &["init"]);
    assert!(
        !invalid_output.status.success(),
        "init should reject impossible embedding_only/disabled runtime requests"
    );
    assert!(
        stdout(&invalid_output)
            .contains("embedding_only requires a non-disabled embedding backend"),
        "init should explain invalid runtime requests: {}",
        stdout(&invalid_output)
    );
}
