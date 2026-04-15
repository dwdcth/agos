use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    core::db::Database,
    ingest::{IngestRequest, IngestService},
    memory::record::{RecordType, Scope, TruthLayer},
};

fn unique_temp_dir(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-runtime-gate-tests")
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

fn ingest_args() -> Vec<&'static str> {
    vec![
        "ingest",
        "--source-uri",
        "memo://project/runtime-gate",
        "--source-label",
        "runtime gate memo",
        "--content",
        "runtime gate should block invalid modes before ingest runs",
        "--scope",
        "project",
        "--record-type",
        "decision",
        "--truth-layer",
        "t2",
        "--recorded-at",
        "2026-04-16T10:00:00Z",
    ]
}

fn search_args() -> Vec<&'static str> {
    vec!["search", "runtime gate"]
}

fn agent_search_args() -> Vec<&'static str> {
    vec!["agent-search", "runtime gate"]
}

fn operational_commands() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("ingest", ingest_args()),
        ("search", search_args()),
        ("agent-search", agent_search_args()),
    ]
}

fn seed_project_record(db_path: &Path) {
    let db = Database::open(db_path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/runtime-gate-seed".to_string(),
            source_label: Some("runtime gate seed".to_string()),
            source_kind: None,
            content: "runtime gate seed record for lexical search and agent-search".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:05:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("seed ingest should succeed");
}

#[test]
fn gated_commands_fail_for_reserved_or_invalid_runtime_modes() {
    for (name, mode, backend, expected) in [
        (
            "embedding-reserved",
            "embedding_only",
            "reserved",
            "embedding_only is reserved but not implemented in Phase 1",
        ),
        (
            "hybrid-reserved",
            "hybrid",
            "reserved",
            "hybrid keeps lexical as the primary baseline, but the embedding secondary path is reserved in Phase 1",
        ),
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
        write_config(&config_path, &db_path, mode, backend);

        for (command_name, args) in operational_commands() {
            let output = run_cli(&config_path, &args);
            let text = stdout(&output);

            assert!(
                !output.status.success(),
                "{command_name} should fail for {mode}/{backend}: stdout={text} stderr={}",
                stderr(&output)
            );
            assert!(
                text.contains("ready: false"),
                "{command_name} should render doctor-style readiness output for {mode}/{backend}: {text}"
            );
            assert!(
                text.contains(expected),
                "{command_name} should explain the blocked runtime mode {mode}/{backend}: {text}"
            );
        }
    }
}

#[test]
fn gated_commands_succeed_for_ready_lexical_mode() {
    let dir = unique_temp_dir("lexical-ready");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path, "lexical_only", "disabled");

    Database::open(&db_path).expect("database should bootstrap for lexical-ready config");

    let ingest_output = run_cli(&config_path, &ingest_args());
    assert!(
        ingest_output.status.success(),
        "ingest should succeed for lexical_only/disabled: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    seed_project_record(&db_path);

    let search_output = run_cli(&config_path, &search_args());
    assert!(
        search_output.status.success(),
        "search should succeed for lexical_only/disabled: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );

    let agent_search_output = run_cli(&config_path, &agent_search_args());
    assert!(
        agent_search_output.status.success(),
        "agent-search should succeed for lexical_only/disabled: stdout={} stderr={}",
        stdout(&agent_search_output),
        stderr(&agent_search_output)
    );
}
