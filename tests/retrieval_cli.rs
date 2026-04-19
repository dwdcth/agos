use std::{
    fs,
    path::Path,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    core::config::{EmbeddingBackend, EmbeddingConfig, RootRuntimeConfig},
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
    write_config_with_mode(path, db_path, "lexical_only", "disabled", None, None);
}

fn write_config_with_mode(
    path: &Path,
    db_path: &Path,
    mode: &str,
    backend: &str,
    model: Option<&str>,
    vector_backend: Option<&str>,
) {
    let parent = path.parent().expect("config path should have parent");
    fs::create_dir_all(parent).expect("config parent should exist");
    let model_line = model
        .map(|value| format!("model = \"{value}\"\n"))
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
{model_line}{vector_block}
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

struct FixtureRecord<'a> {
    source_uri: &'a str,
    source_label: &'a str,
    content: &'a str,
    scope: Scope,
    record_type: RecordType,
    truth_layer: TruthLayer,
    recorded_at: &'a str,
    valid_from: Option<&'a str>,
    valid_to: Option<&'a str>,
}

fn ingest_record(service: &IngestService<'_>, record: FixtureRecord<'_>) {
    service
        .ingest(IngestRequest {
            source_uri: record.source_uri.to_string(),
            source_label: Some(record.source_label.to_string()),
            source_kind: None,
            content: record.content.to_string(),
            scope: record.scope,
            record_type: record.record_type,
            truth_layer: record.truth_layer,
            recorded_at: record.recorded_at.to_string(),
            valid_from: record.valid_from.map(ToOwned::to_owned),
            valid_to: record.valid_to.map(ToOwned::to_owned),
        })
        .expect("ingest should succeed");
}

#[test]
fn library_search_returns_citations_and_filter_trace() {
    let path = fresh_db_path("library-shape");
    let db = Database::open(&path).expect("database should open");
    assert_eq!(db.schema_version().expect("schema version"), 7);
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/search-decision",
            source_label: "search decision memo",
            content: "lexical retrieval must stay explainable and preserve citations for the project team",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T10:00:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://session/search-note",
            source_label: "search session note",
            content: "lexical retrieval notes from a session should be filtered out by scope",
            scope: Scope::Session,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T09:00:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/expired-fact",
            source_label: "expired fact memo",
            content: "lexical retrieval fact has expired and should be filtered by validity",
            scope: Scope::Project,
            record_type: RecordType::Fact,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-01T09:00:00Z",
            valid_from: Some("2026-03-01T00:00:00Z"),
            valid_to: Some("2026-04-05T00:00:00Z"),
        },
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

    assert_eq!(
        response.results.len(),
        1,
        "filters should narrow results in SQL"
    );
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
        result.record.source.uri, "memo://project/search-decision",
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

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before ingest/search: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

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
    assert_eq!(
        ingest_json["chunk_count"], 1,
        "ingest json should surface chunk count"
    );

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
    assert_eq!(
        search_json["results"][0]["trace"]["channel_contribution"],
        "lexical_only"
    );
}

#[test]
fn cli_search_text_output_renders_citation_summary() {
    let dir = unique_temp_dir("cli-text");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before text ingest/search: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

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

#[test]
fn search_surface_respects_dual_channel_mode_selection() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("mode-selection");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "lexical_only",
        "builtin",
        Some(&config.embedding.model),
        Some("sqlite_vec"),
    );

    let db = Database::open(&db_path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some(config.embedding.model.clone()),
            endpoint: None,
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/mode-selection",
            source_label: "mode selection memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:00:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let lexical_output = run_cli(
        &config_path,
        &["search", "citations", "--mode", "lexical_only", "--json"],
    );
    let lexical_json: Value =
        serde_json::from_str(&stdout(&lexical_output)).expect("lexical search should emit json");
    assert_eq!(
        lexical_json["results"][0]["trace"]["channel_contribution"],
        "lexical_only"
    );

    let embedding_output = run_cli(
        &config_path,
        &["search", "retrieval fusion", "--mode", "embedding_only", "--json"],
    );
    let embedding_json: Value = serde_json::from_str(&stdout(&embedding_output))
        .expect("embedding search should emit json");
    assert_eq!(
        embedding_json["results"][0]["trace"]["channel_contribution"],
        "embedding_only"
    );

    let hybrid_output = run_cli(
        &config_path,
        &["search", "retrieval fusion", "--mode", "hybrid", "--json"],
    );
    let hybrid_json: Value =
        serde_json::from_str(&stdout(&hybrid_output)).expect("hybrid search should emit json");
    assert_eq!(
        hybrid_json["results"][0]["trace"]["channel_contribution"],
        "hybrid"
    );
}
