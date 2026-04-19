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
    write_config_with_runtime(path, db_path, mode, backend, None, None, None);
}

fn write_config_with_runtime(
    path: &Path,
    db_path: &Path,
    mode: &str,
    backend: &str,
    model: Option<&str>,
    endpoint: Option<&str>,
    vector_backend: Option<&str>,
) {
    let parent = path.parent().expect("config path should have parent");
    fs::create_dir_all(parent).expect("config parent should exist");
    let model_line = model
        .map(|value| format!("model = \"{value}\"\n"))
        .unwrap_or_default();
    let endpoint_line = endpoint
        .map(|value| format!("endpoint = \"{value}\"\n"))
        .unwrap_or_default();
    let vector_block = vector_backend
        .map(|backend| {
            format!(
                "\n[vector]\nbackend = \"{backend}\"\ntable = \"object_embeddings_vec\"\nsimilarity = \"cosine\"\n"
            )
        })
        .unwrap_or_default();
    fs::write(
        path,
        format!(
            r#"
db_path = "{}"

[retrieval]
mode = "{mode}"

[embedding]
backend = "{backend}"
{model_line}{endpoint_line}
{vector_block}
"#,
            db_path.display()
        ),
    )
    .expect("config should be written");
}

fn write_config_with_embedding(
    path: &Path,
    db_path: &Path,
    mode: &str,
    backend: &str,
    model: Option<&str>,
    endpoint: Option<&str>,
) {
    write_config_with_runtime(path, db_path, mode, backend, model, endpoint, None);
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
        inspect_text.contains("schema_version: 7"),
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
        text.contains("schema_version: 7"),
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

#[test]
fn diagnostic_commands_remain_informational_while_operational_gate_uses_same_contract() {
    let reserved_dir = unique_temp_dir("diagnostic-reserved");
    let reserved_db_path = reserved_dir.join("agent-memos.sqlite");
    let reserved_config_path = reserved_dir.join("config.toml");
    Database::open(&reserved_db_path).expect("database should bootstrap for reserved diagnostics");
    write_config(
        &reserved_config_path,
        &reserved_db_path,
        "embedding_only",
        "reserved",
    );

    let reserved_status = run_cli(&reserved_config_path, &["status"]);
    assert!(
        reserved_status.status.success(),
        "status should remain informational for reserved modes: stdout={} stderr={}",
        stdout(&reserved_status),
        stderr(&reserved_status)
    );

    let reserved_doctor = run_cli(&reserved_config_path, &["doctor"]);
    let reserved_search = run_cli(&reserved_config_path, &["search", "runtime gate"]);
    assert!(
        !reserved_doctor.status.success(),
        "doctor should fail for reserved semantic modes"
    );
    assert!(
        !reserved_search.status.success(),
        "operational commands should fail for reserved semantic modes"
    );
    assert!(
        stdout(&reserved_search).contains("embedding_only is reserved but not implemented in Phase 1"),
        "operational gate should preserve the explicit reserved semantic-mode failure: {}",
        stdout(&reserved_search)
    );

    let bad_dir = unique_temp_dir("diagnostic-bad-db");
    let bad_db_path = bad_dir.join("data").join("agent-memos.sqlite");
    let bad_config_path = bad_dir.join("config.toml");
    fs::create_dir_all(bad_db_path.parent().expect("db path should have parent"))
        .expect("db parent should exist");
    fs::write(&bad_db_path, b"not a sqlite database").expect("bad db fixture should be written");
    write_config(&bad_config_path, &bad_db_path, "lexical_only", "disabled");

    let bad_status = run_cli(&bad_config_path, &["status"]);
    assert!(
        bad_status.status.success(),
        "status should stay informational for bad db files: stdout={} stderr={}",
        stdout(&bad_status),
        stderr(&bad_status)
    );
    assert!(
        stdout(&bad_status).contains("schema inspection failed for existing database file"),
        "status should explain the broken local db file: {}",
        stdout(&bad_status)
    );

    let inspect_output = run_cli(&bad_config_path, &["inspect", "schema"]);
    assert!(
        inspect_output.status.success(),
        "inspect schema should remain diagnostic-only for bad db files: stdout={} stderr={}",
        stdout(&inspect_output),
        stderr(&inspect_output)
    );

    let bad_search = run_cli(&bad_config_path, &["search", "runtime gate"]);
    assert!(
        !bad_search.status.success(),
        "operational commands should fail for broken local db files"
    );
    assert!(
        stdout(&bad_search).contains("schema inspection failed for existing database file"),
        "operational gate should surface the same diagnostic reason for broken local db files: {}",
        stdout(&bad_search)
    );
}

#[test]
fn embedding_foundation_status_reports_backend_readiness_truthfully() {
    for (name, model, embedding_state, expected_note) in [
        (
            "builtin-ready",
            Some("hash-64"),
            "ready",
            "optional second-channel foundation",
        ),
        (
            "builtin-missing-model",
            None,
            "deferred",
            "no embedding model is set yet",
        ),
    ] {
        let dir = unique_temp_dir(name);
        let db_path = dir.join("agent-memos.sqlite");
        let config_path = dir.join("config.toml");
        Database::open(&db_path).expect("database should bootstrap for status");
        write_config_with_embedding(
            &config_path,
            &db_path,
            "lexical_only",
            "builtin",
            model,
            None,
        );

        let output = run_cli(&config_path, &["status"]);
        let text = stdout(&output);
        assert!(
            output.status.success(),
            "status should stay informational for lexical_only+builtin: stdout={text} stderr={}",
            stderr(&output)
        );
        assert!(
            text.contains("configured_mode: lexical_only"),
            "status should preserve lexical-only configured mode: {text}"
        );
        assert!(
            text.contains("embedding_backend: builtin"),
            "status should render the builtin backend label: {text}"
        );
        assert!(
            text.contains(&format!("embedding_dependency_state: {embedding_state}")),
            "status should reflect builtin backend readiness truthfully: {text}"
        );
        assert!(
            text.contains(expected_note),
            "status notes should explain builtin embedding readiness state: {text}"
        );
        assert!(
            text.contains("ready: true"),
            "builtin foundation should not break lexical-only readiness: {text}"
        );
    }
}

#[test]
fn embedding_foundation_doctor_preserves_lexical_first_contract() {
    let lexical_dir = unique_temp_dir("embedding-foundation-lexical");
    let lexical_db_path = lexical_dir.join("agent-memos.sqlite");
    let lexical_config_path = lexical_dir.join("config.toml");
    Database::open(&lexical_db_path).expect("database should bootstrap");
    write_config_with_embedding(
        &lexical_config_path,
        &lexical_db_path,
        "lexical_only",
        "builtin",
        Some("hash-64"),
        None,
    );

    let lexical_doctor = run_cli(&lexical_config_path, &["doctor"]);
    assert!(
        lexical_doctor.status.success(),
        "doctor should keep lexical-only green when builtin embedding is optional foundation: stdout={} stderr={}",
        stdout(&lexical_doctor),
        stderr(&lexical_doctor)
    );

    for (name, mode, expected) in [
        (
            "embedding-foundation-only",
            "embedding_only",
            "embedding vector sidecar/index is not ready for embedding_only retrieval",
        ),
        (
            "hybrid-foundation-only",
            "hybrid",
            "embedding vector sidecar/index is not ready for hybrid retrieval",
        ),
    ] {
        let dir = unique_temp_dir(name);
        let db_path = dir.join("agent-memos.sqlite");
        let config_path = dir.join("config.toml");
        Database::open(&db_path).expect("database should bootstrap");
        write_config_with_embedding(
            &config_path,
            &db_path,
            mode,
            "builtin",
            Some("hash-64"),
            None,
        );
        {
            let conn = rusqlite::Connection::open(&db_path).expect("sqlite db should open");
            conn.execute("DROP TABLE record_embedding_index_state", [])
                .expect("test fixture should drop embedding index state table");
        }

        let output = run_cli(&config_path, &["doctor"]);
        let text = stdout(&output);
        assert!(
            !output.status.success(),
            "doctor should block semantic-primary foundation-only modes {mode}: stdout={text} stderr={}",
            stderr(&output)
        );
        assert!(
            text.contains(expected),
            "doctor should explain that Phase 8 only establishes the foundation for {mode}: {text}"
        );
    }
}

#[test]
fn embedding_foundation_status_reports_vector_sidecar_state() {
    let ready_dir = unique_temp_dir("embedding-index-ready");
    let ready_db_path = ready_dir.join("agent-memos.sqlite");
    let ready_config_path = ready_dir.join("config.toml");
    Database::open(&ready_db_path).expect("database should bootstrap embedding foundation schema");
    write_config_with_embedding(
        &ready_config_path,
        &ready_db_path,
        "lexical_only",
        "builtin",
        Some("hash-16"),
        None,
    );

    let ready_status = run_cli(&ready_config_path, &["status"]);
    let ready_text = stdout(&ready_status);
    assert!(
        ready_status.status.success(),
        "status should succeed for builtin embedding foundation: stdout={ready_text} stderr={}",
        stderr(&ready_status)
    );
    assert!(
        ready_text.contains("embedding_index_readiness: ready"),
        "status should report ready embedding index sidecar state when the foundation schema exists: {ready_text}"
    );

    let inspect_text = stdout(&run_cli(&ready_config_path, &["inspect", "schema"]));
    assert!(
        inspect_text.contains("embedding_index_readiness: ready"),
        "inspect schema should expose embedding index readiness alongside lexical readiness: {inspect_text}"
    );

    let missing_dir = unique_temp_dir("embedding-index-missing");
    let missing_db_path = missing_dir.join("agent-memos.sqlite");
    let missing_config_path = missing_dir.join("config.toml");
    Database::open(&missing_db_path).expect("database should bootstrap first");
    {
        let conn = rusqlite::Connection::open(&missing_db_path).expect("sqlite db should open");
        conn.execute("DROP TABLE record_embedding_index_state", [])
            .expect("test fixture should drop embedding index state table");
    }
    write_config_with_embedding(
        &missing_config_path,
        &missing_db_path,
        "lexical_only",
        "builtin",
        Some("hash-16"),
        None,
    );

    let missing_status = run_cli(&missing_config_path, &["status"]);
    let missing_text = stdout(&missing_status);
    assert!(
        missing_status.status.success(),
        "status should remain informational when embedding index state is missing: stdout={missing_text} stderr={}",
        stderr(&missing_status)
    );
    assert!(
        missing_text.contains("embedding_index_readiness: missing"),
        "status should distinguish missing embedding sidecar state from lexical readiness: {missing_text}"
    );
    assert!(
        missing_text.contains("index_readiness: ready"),
        "lexical readiness should remain independent from embedding index state: {missing_text}"
    );
}

#[test]
fn dual_channel_status_and_doctor_report_mode_compatibility_truthfully() {
    let ready_dir = unique_temp_dir("dual-channel-ready");
    let ready_db_path = ready_dir.join("agent-memos.sqlite");
    let ready_config_path = ready_dir.join("config.toml");
    Database::open(&ready_db_path).expect("database should bootstrap");
    write_config_with_runtime(
        &ready_config_path,
        &ready_db_path,
        "hybrid",
        "builtin",
        Some("hash-16"),
        None,
        Some("sqlite_vec"),
    );

    let status_output = run_cli(&ready_config_path, &["status"]);
    let status_text = stdout(&status_output);
    assert!(
        status_text.contains("active_channels: lexical,embedding"),
        "status should show both channels active when hybrid substrate is ready: {status_text}"
    );
    assert!(
        status_text.contains("gated_channels: none"),
        "status should show no gated channels when hybrid substrate is ready: {status_text}"
    );

    let doctor_output = run_cli(&ready_config_path, &["doctor"]);
    assert!(
        doctor_output.status.success(),
        "doctor should allow hybrid when both lexical and embedding substrate are ready: stdout={} stderr={}",
        stdout(&doctor_output),
        stderr(&doctor_output)
    );

    let gated_dir = unique_temp_dir("dual-channel-gated");
    let gated_db_path = gated_dir.join("agent-memos.sqlite");
    let gated_config_path = gated_dir.join("config.toml");
    Database::open(&gated_db_path).expect("database should bootstrap");
    {
        let conn = rusqlite::Connection::open(&gated_db_path).expect("sqlite db should open");
        conn.execute("DROP TABLE record_embedding_index_state", [])
            .expect("test fixture should drop embedding index state table");
    }
    write_config_with_runtime(
        &gated_config_path,
        &gated_db_path,
        "embedding_only",
        "builtin",
        Some("hash-16"),
        None,
        Some("sqlite_vec"),
    );

    let gated_status = run_cli(&gated_config_path, &["status"]);
    let gated_text = stdout(&gated_status);
    assert!(
        gated_text.contains("active_channels: none"),
        "status should show no active channels when embedding mode is gated: {gated_text}"
    );
    assert!(
        gated_text.contains("gated_channels: embedding"),
        "status should show embedding channel as gated when vector sidecar is unavailable: {gated_text}"
    );

    let gated_doctor = run_cli(&gated_config_path, &["doctor"]);
    assert!(
        !gated_doctor.status.success(),
        "doctor should fail when embedding-only mode lacks vector sidecar readiness"
    );
    assert!(
        stdout(&gated_doctor).contains("embedding vector sidecar/index is not ready for embedding_only retrieval"),
        "doctor should explain which channel is gated: {}",
        stdout(&gated_doctor)
    );
}
