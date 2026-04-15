use std::{
    fs,
    path::Path,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    core::db::Database,
    ingest::{IngestRequest, IngestService},
    memory::record::{RecordType, Scope, TruthLayer},
    search::{SearchFilters, SearchRequest, SearchService},
};
use serde_json::Value;

fn fresh_db_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-retrieval-tests")
        .join(format!("{name}-{unique}"))
        .join("retrieval.sqlite")
}

fn unique_temp_dir(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-retrieval-cli-tests")
        .join(format!("{name}-{unique}"))
}

fn write_config(path: &Path, db_path: &Path) {
    let parent = path.parent().expect("config path should have parent");
    fs::create_dir_all(parent).expect("config parent should exist");
    fs::write(
        path,
        format!(
            r#"
db_path = "{}"

[retrieval]
mode = "lexical_only"

[embedding]
backend = "disabled"
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

fn ingest_record(
    service: &IngestService<'_>,
    source_uri: &str,
    source_label: &str,
    content: &str,
    scope: Scope,
    record_type: RecordType,
    truth_layer: TruthLayer,
    recorded_at: &str,
    valid_from: Option<&str>,
    valid_to: Option<&str>,
) {
    service
        .ingest(IngestRequest {
            source_uri: source_uri.to_string(),
            source_label: Some(source_label.to_string()),
            source_kind: None,
            content: content.to_string(),
            scope,
            record_type,
            truth_layer,
            recorded_at: recorded_at.to_string(),
            valid_from: valid_from.map(ToOwned::to_owned),
            valid_to: valid_to.map(ToOwned::to_owned),
        })
        .expect("ingest should succeed");
}

#[test]
fn library_search_returns_citations_and_filter_trace() {
    let path = fresh_db_path("library-shape");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        "memo://project/search-decision",
        "search decision memo",
        "lexical retrieval must stay explainable and preserve citations for the project team",
        Scope::Project,
        RecordType::Decision,
        TruthLayer::T2,
        "2026-04-15T10:00:00Z",
        Some("2026-04-10T00:00:00Z"),
        Some("2026-04-20T00:00:00Z"),
    );
    ingest_record(
        &ingest,
        "memo://session/search-note",
        "search session note",
        "lexical retrieval notes from a session should be filtered out by scope",
        Scope::Session,
        RecordType::Decision,
        TruthLayer::T2,
        "2026-04-15T09:00:00Z",
        Some("2026-04-10T00:00:00Z"),
        Some("2026-04-20T00:00:00Z"),
    );
    ingest_record(
        &ingest,
        "memo://project/expired-fact",
        "expired fact memo",
        "lexical retrieval fact has expired and should be filtered by validity",
        Scope::Project,
        RecordType::Fact,
        TruthLayer::T2,
        "2026-04-01T09:00:00Z",
        Some("2026-03-01T00:00:00Z"),
        Some("2026-04-05T00:00:00Z"),
    );

    let search = SearchService::new(db.conn());
    let request = SearchRequest::new("lexical retrieval citations")
        .with_limit(5)
        .with_filters(SearchFilters {
            scope: Some(Scope::Project),
            record_type: Some(RecordType::Decision),
            truth_layer: Some(TruthLayer::T2),
            valid_at: Some("2026-04-15T12:00:00Z".to_string()),
            recorded_from: Some("2026-04-10T00:00:00Z".to_string()),
            recorded_to: Some("2026-04-16T00:00:00Z".to_string()),
        });

    let response = search
        .search(&request)
        .expect("library retrieval should succeed");

    assert_eq!(response.results.len(), 1, "filters should narrow results in SQL");
    assert_eq!(
        response.applied_filters.scope,
        Some(Scope::Project),
        "scope filter should be preserved in the response trace"
    );
    assert_eq!(
        response.applied_filters.record_type,
        Some(RecordType::Decision),
        "record type filter should be preserved in the response trace"
    );
    assert_eq!(
        response.applied_filters.truth_layer,
        Some(TruthLayer::T2),
        "truth-layer filter should be preserved in the response trace"
    );
    assert_eq!(
        response.applied_filters.valid_at.as_deref(),
        Some("2026-04-15T12:00:00Z"),
        "valid-at filter should be explicit in the response trace"
    );

    let result = &response.results[0];
    assert_eq!(
        result.record.source.uri,
        "memo://project/search-decision",
        "ordinary retrieval should keep the expected authority-backed row"
    );
    assert_eq!(
        result.citation.record_id, result.record.id,
        "citation should refer to the persisted record instead of the snippet"
    );
    assert_eq!(
        result.citation.source_uri, result.record.source.uri,
        "citation should be derived from persisted source metadata"
    );
    assert_eq!(
        result.citation.anchor.chunk_index, 0,
        "citation should expose chunk provenance"
    );
    assert_eq!(
        result.citation.anchor.chunk_count, 1,
        "citation should expose chunk count provenance"
    );
    assert_eq!(
        result.citation.validity.valid_from.as_deref(),
        Some("2026-04-10T00:00:00Z"),
        "citation should surface explicit validity metadata"
    );
    assert_eq!(
        result.trace.applied_filters, response.applied_filters,
        "per-result trace should remain auditable and reproducible"
    );
    assert!(
        result.score.final_score >= result.score.lexical_base,
        "final score should keep lexical-first weighting while exposing bonuses: {:?}",
        result.score
    );
}

#[test]
fn cli_ingest_and_search_emit_json_reports() {
    let dir = unique_temp_dir("cli-json");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-json",
            "--source-label",
            "cli json memo",
            "--content",
            "ordinary retrieval should stay local-first and return structured citations",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T10:00:00Z",
            "--valid-from",
            "2026-04-10T00:00:00Z",
            "--valid-to",
            "2026-04-20T00:00:00Z",
            "--json",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );
    let ingest_json: Value =
        serde_json::from_str(&stdout(&ingest_output)).expect("ingest should emit json");
    assert_eq!(ingest_json["chunk_count"], 1, "ingest json should surface chunk count");

    let search_output = run_cli(
        &config_path,
        &[
            "search",
            "structured citations",
            "--top-k",
            "5",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--valid-at",
            "2026-04-15T12:00:00Z",
            "--from",
            "2026-04-10T00:00:00Z",
            "--to",
            "2026-04-16T00:00:00Z",
            "--json",
            "--trace",
        ],
    );
    assert!(
        search_output.status.success(),
        "cli search should succeed: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(search_json["results"].as_array().map(Vec::len), Some(1));
    assert_eq!(search_json["applied_filters"]["scope"], "project");
    assert_eq!(
        search_json["results"][0]["citation"]["source_uri"],
        "memo://project/cli-json"
    );
    assert_eq!(
        search_json["results"][0]["trace"]["applied_filters"]["record_type"],
        "decision"
    );
}

#[test]
fn cli_search_text_output_renders_citation_summary() {
    let dir = unique_temp_dir("cli-text");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-text",
            "--content",
            "ordinary retrieval text mode should still render source citations and score summaries",
            "--scope",
            "project",
            "--record-type",
            "observation",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T11:00:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before text search: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(
        &config_path,
        &["search", "score summaries", "--top-k", "3", "--trace"],
    );
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("memo://project/cli-text"),
        "text output should render source citations: {text}"
    );
    assert!(
        text.contains("final_score:"),
        "text output should include score summaries when rendering results: {text}"
    );
    assert!(
        text.contains("filters:"),
        "text output should expose applied filters and trace context: {text}"
    );
}
