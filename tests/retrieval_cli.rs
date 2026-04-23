use std::{
    fs,
    path::Path,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    core::config::{
        Config, EmbeddingBackend, EmbeddingConfig, RetrievalConfig, RetrievalMode,
        RetrievalModeVariant, RootLlmConfig, RootRuntimeConfig, RootVectorConfig,
        VectorBackend,
    },
    core::db::Database,
    ingest::{IngestRequest, IngestService},
    memory::repository::{MemoryRepository, RecordEmbedding},
    memory::record::{
        MemoryRecord, Provenance, RecordTimestamp, RecordType, Scope, SourceKind, SourceRef,
        TruthLayer, ValidityWindow,
    },
    search::{SearchFilters, SearchRequest, SearchService, lexical::MAX_RECALL_LIMIT},
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

fn malformed_record_without_chunk(
    id: &str,
    source_uri: &str,
    source_label: &str,
    content: &str,
    recorded_at: &str,
) -> MemoryRecord {
    MemoryRecord {
        id: id.to_string(),
        source: SourceRef {
            uri: source_uri.to_string(),
            kind: SourceKind::Document,
            label: Some(source_label.to_string()),
        },
        timestamp: RecordTimestamp {
            recorded_at: recorded_at.to_string(),
            created_at: recorded_at.to_string(),
            updated_at: recorded_at.to_string(),
        },
        scope: Scope::Project,
        record_type: RecordType::Decision,
        truth_layer: TruthLayer::T2,
        provenance: Provenance {
            origin: "test".to_string(),
            imported_via: Some("manual_repository_insert".to_string()),
            derived_from: vec![source_uri.to_string()],
        },
        content_text: content.to_string(),
        chunk: None,
        validity: ValidityWindow::default(),
    }
}

#[test]
fn library_search_returns_citations_and_filter_trace() {
    let path = fresh_db_path("library-shape");
    let db = Database::open(&path).expect("database should open");
    assert_eq!(db.schema_version().expect("schema version"), 8);
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
            ..Default::default()
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
        result.citation.recorded_at, "2026-04-15T10:00:00Z",
        "citation should surface the persisted recorded_at timestamp"
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
fn library_search_rejects_records_missing_chunk_metadata_for_citation_output() {
    let path = fresh_db_path("library-missing-chunk-metadata");
    let db = Database::open(&path).expect("database should open");
    let repository = MemoryRepository::new(db.conn());
    repository
        .insert_record(&malformed_record_without_chunk(
            "mem-missing-chunk-library",
            "memo://project/missing-chunk-library",
            "missing chunk library memo",
            "missing chunk metadata should fail closed during citation output",
            "2026-04-18T13:00:00Z",
        ))
        .expect("record insert should succeed even without chunk metadata");

    let err = SearchService::new(db.conn())
        .search(&SearchRequest::new("missing chunk metadata"))
        .expect_err("library search should reject records that lack citation chunk metadata");

    assert!(
        err.to_string()
            .contains("missing persisted chunk metadata required for citation output"),
        "library error should explain the missing chunk metadata requirement: {err}"
    );
}

#[test]
fn cli_search_reports_missing_chunk_metadata_failure() {
    let dir = unique_temp_dir("cli-missing-chunk-metadata");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before missing chunk metadata search check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let db = Database::open(&db_path).expect("database should bootstrap");
    MemoryRepository::new(db.conn())
        .insert_record(&malformed_record_without_chunk(
            "mem-missing-chunk-cli",
            "memo://project/missing-chunk-cli",
            "missing chunk cli memo",
            "missing chunk metadata should fail closed during citation output",
            "2026-04-18T13:05:00Z",
        ))
        .expect("record insert should succeed even without chunk metadata");

    let output = run_cli(&config_path, &["search", "missing chunk metadata"]);
    let combined = format!("{}\n{}", stdout(&output), stderr(&output));

    assert!(
        !output.status.success(),
        "cli search should fail closed when citation chunk metadata is missing: {combined}"
    );
    assert!(
        combined.contains("missing persisted chunk metadata required for citation output"),
        "cli output should explain the missing chunk metadata requirement: {combined}"
    );
}

#[test]
fn library_search_preserves_citation_recorded_at_for_mixed_recall() {
    let path = fresh_db_path("library-mixed-recorded-at");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-mixed-recorded-at",
            source_label: "library mixed recorded-at memo",
            content: "use lexical-first as baseline because explainability matters",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T21:00:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline"))
        .expect("mixed lexical + structured search should succeed");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].citation.recorded_at,
        "2026-04-16T21:00:00Z"
    );
    assert!(
        response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Simple)
            && response.results[0]
                .trace
                .query_strategies
                .contains(&agent_memos::search::QueryStrategy::Structured),
        "library mixed recorded-at recall should preserve both provenance branches"
    );
}

#[test]
fn library_search_preserves_citation_validity_for_mixed_recall() {
    let path = fresh_db_path("library-mixed-validity");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-mixed-validity",
            source_label: "library mixed validity memo",
            content: "use lexical-first as baseline because explainability matters",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T21:02:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline"))
        .expect("mixed lexical + structured search should succeed");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].citation.validity.valid_from.as_deref(),
        Some("2026-04-10T00:00:00Z")
    );
    assert_eq!(
        response.results[0].citation.validity.valid_to.as_deref(),
        Some("2026-04-20T00:00:00Z")
    );
    assert!(
        response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Simple)
            && response.results[0]
                .trace
                .query_strategies
                .contains(&agent_memos::search::QueryStrategy::Structured),
        "library mixed validity recall should preserve both provenance branches"
    );
}

#[test]
fn library_search_preserves_citation_anchor_for_mixed_recall() {
    let path = fresh_db_path("library-mixed-anchor");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-mixed-anchor",
            source_label: "library mixed anchor memo",
            content: "use lexical-first as baseline because explainability matters",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T21:05:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline"))
        .expect("mixed lexical + structured search should succeed");

    assert_eq!(response.results.len(), 1);
    assert_eq!(response.results[0].citation.anchor.chunk_index, 0);
    assert_eq!(response.results[0].citation.anchor.chunk_count, 1);
    assert!(
        response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Simple)
            && response.results[0]
                .trace
                .query_strategies
                .contains(&agent_memos::search::QueryStrategy::Structured),
        "library mixed anchor recall should preserve both provenance branches"
    );
}

#[test]
fn library_search_preserves_line_range_anchor_for_mixed_recall() {
    let path = fresh_db_path("library-mixed-line-range-anchor");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-mixed-line-range-anchor",
            source_label: "library mixed line-range anchor memo",
            content: "use lexical-first as baseline because explainability matters",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T21:10:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline"))
        .expect("mixed lexical + structured search should succeed");

    assert_eq!(response.results.len(), 1);
    assert!(matches!(
        response.results[0].citation.anchor.anchor,
        agent_memos::memory::record::ChunkAnchor::LineRange {
            start_line: 1,
            end_line: 1
        }
    ));
    assert!(
        response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Simple)
            && response.results[0]
                .trace
                .query_strategies
                .contains(&agent_memos::search::QueryStrategy::Structured),
        "library mixed line-range anchor recall should preserve both provenance branches"
    );
}

#[test]
fn library_search_preserves_record_shape_for_mixed_recall() {
    let path = fresh_db_path("library-mixed-record-shape");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-mixed-record-shape",
            source_label: "library mixed record shape memo",
            content: "use lexical-first as baseline because explainability matters",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T21:15:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline"))
        .expect("mixed lexical + structured search should succeed");

    assert_eq!(response.results.len(), 1);
    assert_eq!(response.results[0].record.scope, Scope::Project);
    assert_eq!(response.results[0].record.truth_layer, TruthLayer::T2);
    assert_eq!(response.results[0].record.record_type, RecordType::Decision);
    assert!(
        response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Simple)
            && response.results[0]
                .trace
                .query_strategies
                .contains(&agent_memos::search::QueryStrategy::Structured),
        "library mixed record-shape recall should preserve both provenance branches"
    );
}

#[test]
fn library_search_preserves_matched_query_for_mixed_recall() {
    let path = fresh_db_path("library-mixed-matched-query");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-mixed-matched-query",
            source_label: "library mixed matched-query memo",
            content: "use lexical-first as baseline because explainability matters",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T21:20:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline"))
        .expect("mixed lexical + structured search should succeed");

    assert_eq!(response.results.len(), 1);
    assert_eq!(response.results[0].trace.matched_query, "lexical-first baseline");
    assert!(
        response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Simple)
            && response.results[0]
                .trace
                .query_strategies
                .contains(&agent_memos::search::QueryStrategy::Structured),
        "library mixed matched-query recall should preserve both provenance branches"
    );
}

#[test]
fn library_search_preserves_citation_record_id_for_mixed_recall() {
    let path = fresh_db_path("library-mixed-record-id");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-mixed-record-id",
            source_label: "library mixed record-id memo",
            content: "use lexical-first as baseline because explainability matters",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T21:25:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline"))
        .expect("mixed lexical + structured search should succeed");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].citation.record_id,
        response.results[0].record.id
    );
    assert!(
        response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Simple)
            && response.results[0]
                .trace
                .query_strategies
                .contains(&agent_memos::search::QueryStrategy::Structured),
        "library mixed record-id recall should preserve both provenance branches"
    );
}

#[test]
fn library_search_preserves_source_metadata_for_mixed_recall() {
    let path = fresh_db_path("library-mixed-source-metadata");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-mixed-source-metadata",
            source_label: "library mixed source metadata memo",
            content: "use lexical-first as baseline because explainability matters",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T21:30:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline"))
        .expect("mixed lexical + structured search should succeed");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].citation.source_uri,
        "memo://project/library-mixed-source-metadata"
    );
    assert_eq!(
        response.results[0].citation.source_kind,
        agent_memos::memory::record::SourceKind::Document
    );
    assert_eq!(
        response.results[0].citation.source_label.as_deref(),
        Some("library mixed source metadata memo")
    );
    assert!(
        response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Simple)
            && response.results[0]
                .trace
                .query_strategies
                .contains(&agent_memos::search::QueryStrategy::Structured),
        "library mixed source-metadata recall should preserve both provenance branches"
    );
}

#[test]
fn library_search_preserves_validity_for_mixed_recall() {
    let path = fresh_db_path("library-mixed-validity");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-mixed-validity",
            source_label: "library mixed validity memo",
            content: "use lexical-first as baseline because explainability matters",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T21:31:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline"))
        .expect("mixed lexical + structured search should succeed");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].citation.validity.valid_from.as_deref(),
        Some("2026-04-10T00:00:00Z")
    );
    assert_eq!(
        response.results[0].citation.validity.valid_to.as_deref(),
        Some("2026-04-20T00:00:00Z")
    );
    assert!(
        response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Simple)
            && response.results[0]
                .trace
                .query_strategies
                .contains(&agent_memos::search::QueryStrategy::Structured),
        "library mixed validity recall should preserve both provenance branches"
    );
}

#[test]
fn library_search_preserves_record_source_metadata_for_mixed_recall() {
    let path = fresh_db_path("library-mixed-record-source-metadata");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-mixed-record-source-metadata",
            source_label: "library mixed record source metadata memo",
            content: "use lexical-first as baseline because explainability matters",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T21:32:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline"))
        .expect("mixed lexical + structured search should succeed");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/library-mixed-record-source-metadata"
    );
    assert_eq!(
        response.results[0].record.source.kind,
        agent_memos::memory::record::SourceKind::Document
    );
    assert_eq!(
        response.results[0].record.source.label.as_deref(),
        Some("library mixed record source metadata memo")
    );
    assert!(
        response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Simple)
            && response.results[0]
                .trace
                .query_strategies
                .contains(&agent_memos::search::QueryStrategy::Structured),
        "library mixed record-source metadata recall should preserve both provenance branches"
    );
}

#[test]
fn library_search_preserves_record_timestamp_for_mixed_recall() {
    let path = fresh_db_path("library-mixed-record-timestamp");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-mixed-record-timestamp",
            source_label: "library mixed record timestamp memo",
            content: "use lexical-first as baseline because explainability matters",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T21:33:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline"))
        .expect("mixed lexical + structured search should succeed");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.timestamp.recorded_at,
        "2026-04-16T21:33:00Z"
    );
    assert_eq!(
        response.results[0].record.timestamp.created_at,
        "2026-04-16T21:33:00Z"
    );
    assert_eq!(
        response.results[0].record.timestamp.updated_at,
        "2026-04-16T21:33:00Z"
    );
    assert!(
        response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Simple)
            && response.results[0]
                .trace
                .query_strategies
                .contains(&agent_memos::search::QueryStrategy::Structured),
        "library mixed record-timestamp recall should preserve both provenance branches"
    );
}

#[test]
fn library_search_reports_mixed_recall_as_lexical_only_channel() {
    let path = fresh_db_path("library-mixed-channel");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-mixed-channel",
            source_label: "library mixed channel memo",
            content: "use lexical-first as baseline because explainability matters",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T21:40:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline"))
        .expect("mixed lexical + structured search should succeed");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::LexicalOnly
    );
    assert!(
        response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Simple)
            && response.results[0]
                .trace
                .query_strategies
                .contains(&agent_memos::search::QueryStrategy::Structured),
        "library mixed channel recall should preserve both provenance branches"
    );
}

#[test]
fn library_search_exposes_structured_dsl_sidecars() {
    let path = fresh_db_path("library-dsl");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/dsl-search",
            source_label: "dsl search memo",
            content: "ordinary retrieval should expose the saved structured memory sidecar",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T14:00:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("structured memory sidecar"))
        .expect("library retrieval should succeed");

    assert_eq!(response.results.len(), 1);
    let dsl = response.results[0]
        .dsl
        .as_ref()
        .expect("ordinary retrieval should attach the stored DSL sidecar");
    assert!(!dsl.claim.is_empty());
    assert_eq!(dsl.kind, "decision");
}

#[test]
fn library_search_recalls_structured_taxonomy_terms_without_raw_text_match() {
    let path = fresh_db_path("library-structured-recall");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/structured-recall",
            source_label: "structured recall memo",
            content: "use lexical-first as baseline because explainability matters",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T15:00:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision"))
        .expect("structured lexical retrieval should succeed");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/structured-recall"
    );
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::LexicalOnly,
        "structured-only library recall should stay on the lexical-first channel contract"
    );
    assert!(
        response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Structured),
        "structured taxonomy recall should be visible in the result trace"
    );
}

#[test]
fn library_search_dedupes_repeated_structured_query_terms_before_scoring() {
    let path = fresh_db_path("library-dedupe-repeated-structured-query-terms");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-dedupe-structured-score",
            source_label: "library dedupe structured score memo",
            content: "use lexical-first as baseline because explainability matters",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T15:02:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let single = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision"))
        .expect("single structured-term retrieval should succeed");
    let repeated = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision decision decision"))
        .expect("repeated structured-term retrieval should succeed");

    assert_eq!(single.results.len(), 1);
    assert_eq!(repeated.results.len(), 1);
    assert_eq!(
        repeated.results[0].score.lexical_raw,
        single.results[0].score.lexical_raw,
        "repeating the same structured query term should not inflate structured match score"
    );
    assert_eq!(
        repeated.results[0].score.final_score,
        single.results[0].score.final_score,
        "repeating the same structured query term should not inflate final score"
    );
}

#[test]
fn library_search_preserves_citation_shape_for_structured_only_recall() {
    let path = fresh_db_path("library-structured-citation-shape");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-structured-citation-shape",
            source_label: "library structured citation-shape memo",
            content: "use lexical-first as baseline because explainability matters",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T15:05:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision"))
        .expect("structured lexical retrieval should succeed");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].citation.recorded_at,
        "2026-04-15T15:05:00Z"
    );
    assert_eq!(response.results[0].citation.anchor.chunk_index, 0);
    assert_eq!(response.results[0].citation.anchor.chunk_count, 1);
    assert!(matches!(
        response.results[0].citation.anchor.anchor,
        agent_memos::memory::record::ChunkAnchor::LineRange {
            start_line: 1,
            end_line: 1
        }
    ));
    assert!(
        response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Structured),
        "structured-only citation-shape recall should preserve structured provenance"
    );
}

#[test]
fn library_search_preserves_validity_for_structured_only_recall() {
    let path = fresh_db_path("library-structured-validity");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-structured-validity",
            source_label: "library structured validity memo",
            content: "use lexical-first as baseline because explainability matters",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T15:06:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision"))
        .expect("structured lexical retrieval should succeed");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].citation.validity.valid_from.as_deref(),
        Some("2026-04-10T00:00:00Z")
    );
    assert_eq!(
        response.results[0].citation.validity.valid_to.as_deref(),
        Some("2026-04-20T00:00:00Z")
    );
    assert!(
        response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Structured),
        "structured-only validity recall should preserve structured provenance"
    );
}

#[test]
fn library_search_preserves_source_metadata_for_structured_only_recall() {
    let path = fresh_db_path("library-structured-source-metadata");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-structured-source-metadata",
            source_label: "library structured source metadata memo",
            content: "use lexical-first as baseline because explainability matters",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T15:07:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision"))
        .expect("structured lexical retrieval should succeed");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/library-structured-source-metadata"
    );
    assert_eq!(
        response.results[0].record.source.kind,
        agent_memos::memory::record::SourceKind::Document
    );
    assert_eq!(
        response.results[0].record.source.label.as_deref(),
        Some("library structured source metadata memo")
    );
    assert_eq!(
        response.results[0].citation.source_uri,
        "memo://project/library-structured-source-metadata"
    );
    assert_eq!(
        response.results[0].citation.source_kind,
        agent_memos::memory::record::SourceKind::Document
    );
    assert_eq!(
        response.results[0].citation.source_label.as_deref(),
        Some("library structured source metadata memo")
    );
    assert!(
        response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Structured),
        "structured-only source-metadata recall should preserve structured provenance"
    );
}

#[test]
fn library_search_preserves_citation_record_id_for_structured_only_recall() {
    let path = fresh_db_path("library-structured-record-id");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-structured-record-id",
            source_label: "library structured record-id memo",
            content: "use lexical-first as baseline because explainability matters",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T15:08:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision"))
        .expect("structured lexical retrieval should succeed");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].citation.record_id,
        response.results[0].record.id
    );
    assert!(
        response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Structured),
        "structured-only record-id recall should preserve structured provenance"
    );
}

#[test]
fn library_search_preserves_matched_query_for_structured_only_recall() {
    let path = fresh_db_path("library-structured-matched-query");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-structured-matched-query",
            source_label: "library structured matched-query memo",
            content: "use lexical-first as baseline because explainability matters",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T15:09:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision"))
        .expect("structured lexical retrieval should succeed");

    assert_eq!(response.results.len(), 1);
    assert_eq!(response.results[0].trace.matched_query, "decision");
    assert!(
        response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Structured),
        "structured-only matched-query recall should preserve structured provenance"
    );
}

#[test]
fn library_search_preserves_record_scope_and_truth_layer_for_structured_only_recall() {
    let path = fresh_db_path("library-structured-record-shape");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-structured-record-shape",
            source_label: "library structured record shape memo",
            content: "use lexical-first as baseline because explainability matters",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T15:10:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision"))
        .expect("structured lexical retrieval should succeed");

    assert_eq!(response.results.len(), 1);
    assert_eq!(response.results[0].record.scope, Scope::Project);
    assert_eq!(response.results[0].record.truth_layer, TruthLayer::T2);
    assert_eq!(response.results[0].record.record_type, RecordType::Decision);
    assert!(
        response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Structured),
        "structured-only record-shape recall should preserve structured provenance"
    );
}

#[test]
fn library_search_preserves_record_timestamp_for_structured_only_recall() {
    let path = fresh_db_path("library-structured-record-timestamp");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-structured-record-timestamp",
            source_label: "library structured record timestamp memo",
            content: "use lexical-first as baseline because explainability matters",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T15:11:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision"))
        .expect("structured lexical retrieval should succeed");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.timestamp.recorded_at,
        "2026-04-15T15:11:00Z"
    );
    assert_eq!(
        response.results[0].record.timestamp.created_at,
        "2026-04-15T15:11:00Z"
    );
    assert_eq!(
        response.results[0].record.timestamp.updated_at,
        "2026-04-15T15:11:00Z"
    );
    assert!(
        response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Structured),
        "structured-only record-timestamp recall should preserve structured provenance"
    );
}

#[test]
fn library_search_preserves_record_provenance_for_structured_only_recall() {
    let path = fresh_db_path("library-structured-record-provenance");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-structured-record-provenance",
            source_label: "library structured record provenance memo",
            content: "use lexical-first as baseline because explainability matters",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T15:11:30Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision"))
        .expect("structured lexical retrieval should succeed");

    assert_eq!(response.results.len(), 1);
    assert_eq!(response.results[0].record.provenance.origin, "ingest");
    assert_eq!(
        response.results[0].record.provenance.imported_via.as_deref(),
        Some("ingest_service")
    );
    assert!(
        response.results[0]
            .record
            .provenance
            .derived_from
            .first()
            .is_some_and(|value| value.starts_with("memo://project/library-structured-record-provenance#")),
        "structured-only record provenance should retain the source anchor"
    );
    assert!(
        response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Structured),
        "structured-only record provenance recall should preserve structured provenance"
    );
}

#[test]
fn library_search_preserves_record_provenance_for_mixed_recall() {
    let path = fresh_db_path("library-mixed-record-provenance");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-mixed-record-provenance",
            source_label: "library mixed record provenance memo",
            content: "use lexical-first as baseline because explainability matters",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T21:45:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline"))
        .expect("mixed lexical + structured search should succeed");

    assert_eq!(response.results.len(), 1);
    assert_eq!(response.results[0].record.provenance.origin, "ingest");
    assert_eq!(
        response.results[0].record.provenance.imported_via.as_deref(),
        Some("ingest_service")
    );
    assert!(
        response.results[0]
            .record
            .provenance
            .derived_from
            .first()
            .is_some_and(|value| value.starts_with("memo://project/library-mixed-record-provenance#")),
        "mixed record provenance should retain the source anchor"
    );
    assert!(
        response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Simple)
            && response.results[0]
                .trace
                .query_strategies
                .contains(&agent_memos::search::QueryStrategy::Structured),
        "mixed record provenance recall should preserve both provenance branches"
    );
}

#[test]
fn library_search_filters_results_by_dsl_taxonomy() {
    let path = fresh_db_path("library-taxonomy-filter");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/retrieval-baseline",
            source_label: "retrieval baseline memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T09:00:00Z",
            valid_from: None,
            valid_to: None,
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/config-baseline",
            source_label: "config baseline memo",
            content: "config baseline keeps toml setting review stable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T09:05:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("baseline").with_filters(SearchFilters {
            topic: Some("retrieval".to_string()),
            ..Default::default()
        }))
        .expect("taxonomy-filtered retrieval should succeed");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/retrieval-baseline"
    );
    assert_eq!(response.applied_filters.topic.as_deref(), Some("retrieval"));
}

#[test]
fn library_search_applies_taxonomy_filters_before_top_k_truncation() {
    let path = fresh_db_path("library-taxonomy-filter-before-top-k");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/config-top-k",
            source_label: "config top k memo",
            content: "baseline baseline keeps toml setting review stable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T09:06:00Z",
            valid_from: None,
            valid_to: None,
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/retrieval-top-k",
            source_label: "retrieval top k memo",
            content: "baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T09:00:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("baseline").with_limit(1).with_filters(SearchFilters {
            topic: Some("retrieval".to_string()),
            ..Default::default()
        }))
        .expect("taxonomy-filtered retrieval should succeed even with top-k truncation");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/retrieval-top-k",
        "taxonomy filtering should happen before top-k truncation removes lower-ranked matching records"
    );
}

#[test]
fn library_search_truncates_final_results_to_requested_top_k() {
    let path = fresh_db_path("library-final-top-k-truncation");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/lexical-top-k",
            source_label: "lexical top k memo",
            content: "decision procedures keep lexical ranking inspectable",
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T09:40:00Z",
            valid_from: None,
            valid_to: None,
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/structured-top-k",
            source_label: "structured top k memo",
            content: "baseline remains stable without the keyword",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T09:41:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision").with_limit(1))
        .expect("mixed recall should succeed while respecting final top-k");

    assert_eq!(
        response.results.len(),
        1,
        "final search response should truncate merged recall results to the requested top-k"
    );
}

#[test]
fn library_search_clamps_zero_top_k_to_one_result() {
    let path = fresh_db_path("library-zero-top-k-clamp");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-zero-top-k",
            source_label: "library zero top k memo",
            content: "baseline retrieval should still return one bounded result",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T09:42:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("bounded result").with_limit(0))
        .expect("library search should clamp top-k=0 to one bounded result");

    assert_eq!(
        response.results.len(),
        1,
        "top-k=0 should clamp to one result instead of returning an empty response"
    );
}

#[test]
fn search_request_bounded_limit_clamps_zero_to_one() {
    let request = SearchRequest::new("bounded limit").with_limit(0);
    assert_eq!(
        request.bounded_limit(),
        1,
        "search request limit helper should clamp zero to one"
    );
}

#[test]
fn search_request_new_starts_with_default_limit_and_filters() {
    let request = SearchRequest::new("default request");
    assert_eq!(request.limit, SearchRequest::DEFAULT_LIMIT);
    assert_eq!(request.filters, SearchFilters::default());
}

#[test]
fn search_request_bounded_limit_clamps_excessive_values_to_max_recall_limit() {
    let request = SearchRequest::new("bounded limit").with_limit(999);
    assert_eq!(
        request.bounded_limit(),
        MAX_RECALL_LIMIT,
        "search request limit helper should clamp excessive values to MAX_RECALL_LIMIT"
    );
}

#[test]
fn library_search_returns_empty_results_for_whitespace_query() {
    let path = fresh_db_path("library-whitespace-query");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-whitespace-query",
            source_label: "library whitespace query memo",
            content: "baseline retrieval should stay stable when the query is blank",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T09:43:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("   "))
        .expect("library search should accept whitespace-only queries");

    assert!(
        response.results.is_empty(),
        "whitespace-only query should return an empty result set instead of a misleading match"
    );
}

#[test]
fn library_search_clamps_excessive_top_k_to_max_recall_limit() {
    let path = fresh_db_path("library-excessive-top-k-clamp");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    for index in 0..30 {
        ingest_record(
            &ingest,
            FixtureRecord {
                source_uri: &format!("memo://project/library-top-k-{index}"),
                source_label: "library excessive top k memo",
                content: "bounded recall should clamp excessive top-k requests",
                scope: Scope::Project,
                record_type: RecordType::Observation,
                truth_layer: TruthLayer::T2,
                recorded_at: &format!("2026-04-16T09:{index:02}:00Z"),
                valid_from: None,
                valid_to: None,
            },
        );
    }

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("bounded recall").with_limit(999))
        .expect("library search should clamp excessive top-k requests");

    assert_eq!(
        response.results.len(),
        MAX_RECALL_LIMIT,
        "excessive top-k should clamp to MAX_RECALL_LIMIT instead of returning the full corpus"
    );
}

#[test]
fn library_search_filters_results_by_dsl_domain() {
    let path = fresh_db_path("library-domain-filter");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/domain-project",
            source_label: "domain project memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T09:10:00Z",
            valid_from: None,
            valid_to: None,
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/domain-system",
            source_label: "domain system memo",
            content: "runtime architecture keeps storage integration inspectable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T09:12:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let response = SearchService::new(db.conn())
        .search(
            &SearchRequest::new("inspectable").with_filters(SearchFilters {
                domain: Some("system".to_string()),
                ..Default::default()
            }),
        )
        .expect("domain-filtered retrieval should succeed");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/domain-system"
    );
    assert_eq!(response.applied_filters.domain.as_deref(), Some("system"));
}

#[test]
fn library_search_filters_results_by_dsl_aspect() {
    let path = fresh_db_path("library-aspect-filter");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/aspect-behavior",
            source_label: "aspect behavior memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T09:20:00Z",
            valid_from: None,
            valid_to: None,
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/aspect-risk",
            source_label: "aspect risk memo",
            content: "retrieval drift risk causes debugging failure",
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T09:22:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("risk").with_filters(SearchFilters {
            aspect: Some("risk".to_string()),
            ..Default::default()
        }))
        .expect("aspect-filtered retrieval should succeed");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/aspect-risk"
    );
    assert_eq!(response.applied_filters.aspect.as_deref(), Some("risk"));
}

#[test]
fn library_search_filters_results_by_dsl_kind() {
    let path = fresh_db_path("library-kind-filter");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/kind-decision",
            source_label: "kind decision memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T09:30:00Z",
            valid_from: None,
            valid_to: None,
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/kind-observation",
            source_label: "kind observation memo",
            content: "retrieval baseline was observed in the latest run",
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T09:32:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let response = SearchService::new(db.conn())
        .search(
            &SearchRequest::new("retrieval baseline").with_filters(SearchFilters {
                kind: Some("observation".to_string()),
                ..Default::default()
            }),
        )
        .expect("kind-filtered retrieval should succeed");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/kind-observation"
    );
    assert_eq!(
        response.applied_filters.kind.as_deref(),
        Some("observation")
    );
}

#[test]
fn library_search_prefers_structured_snippet_when_structured_match_exists() {
    let path = fresh_db_path("library-structured-snippet");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/structured-snippet",
            source_label: "structured snippet memo",
            content: "use lexical-first as baseline because explainability matters",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:30:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline decision"))
        .expect("merged lexical and structured retrieval should succeed");

    assert_eq!(response.results.len(), 1);
    assert!(
        response.results[0].snippet.contains("WHY:"),
        "merged results should prefer the structured snippet over a raw lexical snippet: {:?}",
        response.results[0].snippet
    );
}

#[test]
fn library_search_prefers_structured_taxonomy_bonus_in_ranking() {
    let path = fresh_db_path("library-structured-ranking");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/decision-ranking",
            source_label: "decision ranking memo",
            content: "baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T19:00:00Z",
            valid_from: None,
            valid_to: None,
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/observation-ranking",
            source_label: "observation ranking memo",
            content: "baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T19:00:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision baseline"))
        .expect("ranking retrieval should succeed");

    assert_eq!(response.results.len(), 2);
    assert_eq!(
        response.results[0].record.source.uri, "memo://project/decision-ranking",
        "structured kind matches should lift the decision record above a lexical tie"
    );
    assert!(
        response.results[0].score.keyword_bonus > response.results[1].score.keyword_bonus,
        "structured fields should contribute to ranking bonuses"
    );
}

#[test]
fn cli_search_json_reports_structured_query_strategy() {
    let dir = unique_temp_dir("cli-structured-strategy");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before structured retrieval check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-structured",
            "--source-label",
            "cli structured memo",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T16:00:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before structured retrieval check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "decision", "--json"]);
    assert!(
        search_output.status.success(),
        "cli search should succeed for structured recall: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );

    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(
        search_json["results"][0]["trace"]["query_strategies"][0],
        "Structured"
    );
}

#[test]
fn cli_search_json_reports_structured_only_channel_as_lexical_first() {
    let dir = unique_temp_dir("cli-json-structured-channel");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before structured-channel check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-json-structured-channel",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T10:30:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before structured-channel check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "decision", "--json"]);
    assert!(
        search_output.status.success(),
        "cli json search should succeed for structured-only channel check: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(
        search_json["results"][0]["trace"]["channel_contribution"],
        "lexical_only"
    );
    assert_eq!(search_json["results"][0]["trace"]["matched_query"], "decision");
    assert!(
        search_json["results"][0]["trace"]["query_strategies"]
            .as_array()
            .is_some_and(|strategies| strategies.iter().any(|value| value == "Structured")),
        "structured-only query should still be reported as lexical-first with structured provenance"
    );
}

#[test]
fn cli_search_json_preserves_citation_shape_for_structured_only_recall() {
    let dir = unique_temp_dir("cli-json-structured-only-citation-shape");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before structured-only citation-shape check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-json-structured-only-citation-shape",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T10:32:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before structured-only citation-shape check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "decision", "--json"]);
    assert!(
        search_output.status.success(),
        "cli json search should succeed for structured-only citation-shape: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(
        search_json["results"][0]["citation"]["recorded_at"],
        "2026-04-15T10:32:00Z"
    );
    assert_eq!(search_json["results"][0]["record"]["scope"], "project");
    assert_eq!(search_json["results"][0]["record"]["truth_layer"], "t2");
    assert_eq!(search_json["results"][0]["record"]["record_type"], "decision");
    assert!(
        search_json["results"][0]["citation"]["record_id"]
            .as_str()
            .is_some_and(|value| value.starts_with("mem-")),
        "structured-only json recall should expose the authority citation record id"
    );
    assert_eq!(search_json["results"][0]["citation"]["anchor"]["chunk_index"], 0);
    assert_eq!(search_json["results"][0]["citation"]["anchor"]["chunk_count"], 1);
    assert_eq!(
        search_json["results"][0]["citation"]["anchor"]["anchor"]["kind"],
        "line_range"
    );
    assert_eq!(
        search_json["results"][0]["citation"]["anchor"]["anchor"]["start_line"],
        1
    );
    assert_eq!(
        search_json["results"][0]["citation"]["anchor"]["anchor"]["end_line"],
        1
    );
    assert!(
        search_json["results"][0]["trace"]["query_strategies"]
            .as_array()
            .is_some_and(|strategies| strategies.iter().any(|value| value == "Structured")),
        "structured-only citation-shape recall should preserve structured provenance"
    );
}

#[test]
fn cli_search_json_preserves_source_metadata_for_structured_only_recall() {
    let dir = unique_temp_dir("cli-json-structured-only-source-metadata");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before structured-only source-metadata check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-json-structured-only-source-metadata",
            "--source-label",
            "structured-only source metadata memo",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T10:33:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before structured-only source-metadata check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "decision", "--json"]);
    assert!(
        search_output.status.success(),
        "cli json search should succeed for structured-only source-metadata: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(
        search_json["results"][0]["record"]["source"]["uri"],
        "memo://project/cli-json-structured-only-source-metadata"
    );
    assert_eq!(
        search_json["results"][0]["record"]["source"]["kind"],
        "document"
    );
    assert_eq!(
        search_json["results"][0]["record"]["source"]["label"],
        "structured-only source metadata memo"
    );
    assert_eq!(
        search_json["results"][0]["citation"]["source_uri"],
        "memo://project/cli-json-structured-only-source-metadata"
    );
    assert_eq!(
        search_json["results"][0]["citation"]["source_kind"],
        "document"
    );
    assert_eq!(
        search_json["results"][0]["citation"]["source_label"],
        "structured-only source metadata memo"
    );
    assert!(
        search_json["results"][0]["trace"]["query_strategies"]
            .as_array()
            .is_some_and(|strategies| strategies.iter().any(|value| value == "Structured")),
        "structured-only source-metadata recall should preserve structured provenance"
    );
}

#[test]
fn cli_search_json_preserves_record_timestamp_for_structured_only_recall() {
    let dir = unique_temp_dir("cli-json-structured-only-record-timestamp");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before structured-only record-timestamp check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-json-structured-only-record-timestamp",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T10:34:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before structured-only record-timestamp check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "decision", "--json"]);
    assert!(
        search_output.status.success(),
        "cli json search should succeed for structured-only record-timestamp: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(
        search_json["results"][0]["record"]["timestamp"]["recorded_at"],
        "2026-04-15T10:34:00Z"
    );
    assert_eq!(
        search_json["results"][0]["record"]["timestamp"]["created_at"],
        "2026-04-15T10:34:00Z"
    );
    assert_eq!(
        search_json["results"][0]["record"]["timestamp"]["updated_at"],
        "2026-04-15T10:34:00Z"
    );
    assert!(
        search_json["results"][0]["trace"]["query_strategies"]
            .as_array()
            .is_some_and(|strategies| strategies.iter().any(|value| value == "Structured")),
        "structured-only record-timestamp recall should preserve structured provenance"
    );
}

#[test]
fn cli_search_json_preserves_record_provenance_for_structured_only_recall() {
    let dir = unique_temp_dir("cli-json-structured-only-record-provenance");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before structured-only record-provenance check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-json-structured-only-record-provenance",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T10:34:30Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before structured-only record-provenance check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "decision", "--json"]);
    assert!(
        search_output.status.success(),
        "cli json search should succeed for structured-only record-provenance: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(search_json["results"][0]["record"]["provenance"]["origin"], "ingest");
    assert_eq!(
        search_json["results"][0]["record"]["provenance"]["imported_via"],
        "ingest_service"
    );
    assert!(
        search_json["results"][0]["record"]["provenance"]["derived_from"]
            .as_array()
            .is_some_and(|items| items.iter().any(|value| value
                .as_str()
                .is_some_and(|item| item.starts_with(
                    "memo://project/cli-json-structured-only-record-provenance#"
                )))),
        "structured-only json recall should preserve the source-derived provenance anchor"
    );
    assert!(
        search_json["results"][0]["trace"]["query_strategies"]
            .as_array()
            .is_some_and(|strategies| strategies.iter().any(|value| value == "Structured")),
        "structured-only record-provenance recall should preserve structured provenance"
    );
}

#[test]
fn cli_search_accepts_taxonomy_filter_flags() {
    let dir = unique_temp_dir("cli-taxonomy-flags");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before taxonomy flag check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest = Database::open(&db_path).expect("database should bootstrap");
    let service = IngestService::new(ingest.conn());
    ingest_record(
        &service,
        FixtureRecord {
            source_uri: "memo://project/taxonomy-retrieval",
            source_label: "taxonomy retrieval memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T18:00:00Z",
            valid_from: None,
            valid_to: None,
        },
    );
    ingest_record(
        &service,
        FixtureRecord {
            source_uri: "memo://project/taxonomy-config",
            source_label: "taxonomy config memo",
            content: "config baseline keeps toml settings review stable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T18:05:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(
        &config_path,
        &[
            "search",
            "baseline",
            "--topic",
            "retrieval",
            "--kind",
            "decision",
            "--json",
        ],
    );
    let search_stdout = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli search should succeed with taxonomy flags: stdout={search_stdout} stderr={}",
        stderr(&search_output)
    );
    let json: Value =
        serde_json::from_str(&search_stdout).expect("taxonomy-filtered search should emit json");
    assert_eq!(json["applied_filters"]["topic"], "retrieval");
    assert_eq!(json["applied_filters"]["kind"], "decision");
    assert_eq!(
        json["results"][0]["citation"]["source_uri"],
        "memo://project/taxonomy-retrieval"
    );
}

#[test]
fn cli_search_json_echoes_domain_and_aspect_filters() {
    let dir = unique_temp_dir("cli-taxonomy-domain-aspect");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before domain/aspect filter check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest = Database::open(&db_path).expect("database should bootstrap");
    let service = IngestService::new(ingest.conn());
    ingest_record(
        &service,
        FixtureRecord {
            source_uri: "memo://project/domain-aspect-retrieval",
            source_label: "domain aspect retrieval memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T18:10:00Z",
            valid_from: None,
            valid_to: None,
        },
    );
    ingest_record(
        &service,
        FixtureRecord {
            source_uri: "memo://system/domain-aspect-runtime",
            source_label: "domain aspect runtime memo",
            content: "runtime status ready",
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T18:15:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(
        &config_path,
        &[
            "search", "baseline", "--domain", "project", "--aspect", "behavior", "--json",
        ],
    );
    let search_stdout = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli search should succeed with domain/aspect flags: stdout={search_stdout} stderr={}",
        stderr(&search_output)
    );
    let json: Value = serde_json::from_str(&search_stdout).expect("search should emit json");
    assert_eq!(json["applied_filters"]["domain"], "project");
    assert_eq!(json["applied_filters"]["aspect"], "behavior");
    assert_eq!(
        json["results"][0]["trace"]["applied_filters"]["domain"],
        "project"
    );
    assert_eq!(
        json["results"][0]["trace"]["applied_filters"]["aspect"],
        "behavior"
    );
}

#[test]
fn cli_search_json_echoes_domain_aspect_and_kind_filters_together() {
    let dir = unique_temp_dir("cli-taxonomy-domain-aspect-kind");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before domain/aspect/kind filter check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest = Database::open(&db_path).expect("database should bootstrap");
    let service = IngestService::new(ingest.conn());
    ingest_record(
        &service,
        FixtureRecord {
            source_uri: "memo://project/domain-aspect-kind-hit",
            source_label: "domain aspect kind hit memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T18:20:00Z",
            valid_from: None,
            valid_to: None,
        },
    );
    ingest_record(
        &service,
        FixtureRecord {
            source_uri: "memo://project/domain-aspect-kind-miss",
            source_label: "domain aspect kind miss memo",
            content: "retrieval status ready",
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T18:25:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(
        &config_path,
        &[
            "search", "baseline", "--domain", "project", "--aspect", "behavior", "--kind",
            "decision", "--json",
        ],
    );
    let search_stdout = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli search should succeed with domain/aspect/kind flags: stdout={search_stdout} stderr={}",
        stderr(&search_output)
    );
    let json: Value = serde_json::from_str(&search_stdout).expect("search should emit json");
    assert_eq!(json["applied_filters"]["domain"], "project");
    assert_eq!(json["applied_filters"]["aspect"], "behavior");
    assert_eq!(json["applied_filters"]["kind"], "decision");
    assert_eq!(json["results"].as_array().map(Vec::len), Some(1));
    assert_eq!(
        json["results"][0]["trace"]["applied_filters"]["domain"],
        "project"
    );
    assert_eq!(
        json["results"][0]["trace"]["applied_filters"]["aspect"],
        "behavior"
    );
    assert_eq!(
        json["results"][0]["trace"]["applied_filters"]["kind"],
        "decision"
    );
}

#[test]
fn cli_search_rejects_unknown_taxonomy_filter_values() {
    let dir = unique_temp_dir("cli-invalid-taxonomy");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let output = run_cli(
        &config_path,
        &["search", "baseline", "--topic", "searchingness", "--json"],
    );

    assert!(
        !output.status.success(),
        "cli search should reject unsupported taxonomy filters"
    );
    assert!(
        stderr(&output).contains("unsupported taxonomy topic: searchingness"),
        "stderr should explain the invalid taxonomy value: {}",
        stderr(&output)
    );
}

#[test]
fn cli_search_text_rejects_unknown_taxonomy_filter_values() {
    let dir = unique_temp_dir("cli-text-invalid-taxonomy");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let output = run_cli(
        &config_path,
        &["search", "baseline", "--topic", "searchingness"],
    );

    assert!(
        !output.status.success(),
        "cli text search should reject unsupported taxonomy filters"
    );
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(
        combined.contains("unsupported taxonomy topic: searchingness"),
        "text-mode output should explain the invalid taxonomy value: {combined}",
    );
}

#[test]
fn cli_search_rejects_invalid_domain_topic_combinations() {
    let dir = unique_temp_dir("cli-invalid-taxonomy-combo");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let output = run_cli(
        &config_path,
        &[
            "search", "baseline", "--domain", "project", "--topic", "storage", "--json",
        ],
    );

    assert!(
        !output.status.success(),
        "cli search should reject invalid domain/topic combinations"
    );
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(
        combined.contains("domain=project does not allow topic=storage"),
        "output should explain the invalid taxonomy combination: {combined}",
    );
}

#[test]
fn cli_search_text_rejects_invalid_domain_topic_combinations() {
    let dir = unique_temp_dir("cli-text-invalid-taxonomy-combo");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let output = run_cli(
        &config_path,
        &["search", "baseline", "--domain", "project", "--topic", "storage"],
    );

    assert!(
        !output.status.success(),
        "cli text search should reject invalid domain/topic combinations"
    );
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(
        combined.contains("domain=project does not allow topic=storage"),
        "text-mode output should explain the invalid taxonomy combination: {combined}",
    );
}

#[test]
fn cli_search_rejects_inverted_temporal_ranges() {
    let dir = unique_temp_dir("cli-invalid-temporal-range");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before temporal-range validation check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let output = run_cli(
        &config_path,
        &[
            "search",
            "baseline",
            "--from",
            "2026-04-20T00:00:00Z",
            "--to",
            "2026-04-10T00:00:00Z",
            "--json",
        ],
    );

    assert!(
        !output.status.success(),
        "cli search should reject inverted temporal ranges"
    );
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(
        combined.contains(
            "invalid temporal range: from=2026-04-20T00:00:00Z is later than to=2026-04-10T00:00:00Z"
        ),
        "output should explain the invalid temporal range: {combined}",
    );
}

#[test]
fn cli_search_rejects_invalid_rfc3339_temporal_filters() {
    let dir = unique_temp_dir("cli-invalid-rfc3339-temporal-filter");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before invalid-rfc3339 check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let output = run_cli(
        &config_path,
        &["search", "baseline", "--valid-at", "not-a-time", "--json"],
    );

    assert!(
        !output.status.success(),
        "cli search should reject invalid RFC3339 temporal filters"
    );
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(
        combined.contains("invalid RFC3339 value for valid_at: not-a-time"),
        "output should explain the invalid RFC3339 filter value: {combined}",
    );
}

#[test]
fn cli_search_json_rejects_invalid_rfc3339_recorded_from() {
    let dir = unique_temp_dir("cli-json-invalid-rfc3339-from");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before json invalid-from check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let output = run_cli(
        &config_path,
        &["search", "baseline", "--from", "not-a-time", "--json"],
    );

    assert!(
        !output.status.success(),
        "cli json search should reject invalid RFC3339 recorded_from filters"
    );
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(
        combined.contains("invalid RFC3339 value for from: not-a-time"),
        "json-mode output should explain the invalid RFC3339 from value: {combined}",
    );
}

#[test]
fn cli_search_json_rejects_invalid_rfc3339_recorded_to() {
    let dir = unique_temp_dir("cli-json-invalid-rfc3339-to");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before json invalid-to check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let output = run_cli(
        &config_path,
        &["search", "baseline", "--to", "not-a-time", "--json"],
    );

    assert!(
        !output.status.success(),
        "cli json search should reject invalid RFC3339 recorded_to filters"
    );
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(
        combined.contains("invalid RFC3339 value for to: not-a-time"),
        "json-mode output should explain the invalid RFC3339 to value: {combined}",
    );
}

#[test]
fn cli_search_text_rejects_invalid_rfc3339_recorded_from() {
    let dir = unique_temp_dir("cli-text-invalid-rfc3339-from");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before text invalid-from check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let output = run_cli(&config_path, &["search", "baseline", "--from", "not-a-time"]);

    assert!(
        !output.status.success(),
        "cli text search should reject invalid RFC3339 recorded_from filters"
    );
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(
        combined.contains("invalid RFC3339 value for from: not-a-time"),
        "text-mode output should explain the invalid RFC3339 from value: {combined}",
    );
}

#[test]
fn cli_search_text_rejects_invalid_rfc3339_recorded_to() {
    let dir = unique_temp_dir("cli-text-invalid-rfc3339-to");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before text invalid-to check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let output = run_cli(&config_path, &["search", "baseline", "--to", "not-a-time"]);

    assert!(
        !output.status.success(),
        "cli text search should reject invalid RFC3339 recorded_to filters"
    );
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(
        combined.contains("invalid RFC3339 value for to: not-a-time"),
        "text-mode output should explain the invalid RFC3339 to value: {combined}",
    );
}

#[test]
fn cli_search_text_rejects_inverted_temporal_ranges() {
    let dir = unique_temp_dir("cli-text-invalid-temporal-range");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before text temporal-range validation check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let output = run_cli(
        &config_path,
        &[
            "search",
            "baseline",
            "--from",
            "2026-04-20T00:00:00Z",
            "--to",
            "2026-04-10T00:00:00Z",
        ],
    );

    assert!(
        !output.status.success(),
        "cli text search should reject inverted temporal ranges"
    );
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(
        combined.contains(
            "invalid temporal range: from=2026-04-20T00:00:00Z is later than to=2026-04-10T00:00:00Z"
        ),
        "text-mode output should explain the invalid temporal range: {combined}",
    );
}

#[test]
fn cli_search_text_rejects_invalid_rfc3339_temporal_filters() {
    let dir = unique_temp_dir("cli-text-invalid-rfc3339-temporal-filter");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before text invalid-rfc3339 check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let output = run_cli(&config_path, &["search", "baseline", "--valid-at", "not-a-time"]);

    assert!(
        !output.status.success(),
        "cli text search should reject invalid RFC3339 temporal filters"
    );
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(
        combined.contains("invalid RFC3339 value for valid_at: not-a-time"),
        "text-mode output should explain the invalid RFC3339 filter value: {combined}",
    );
}

#[test]
fn library_search_rejects_unknown_taxonomy_values() {
    let path = fresh_db_path("library-invalid-taxonomy");
    let db = Database::open(&path).expect("database should open");

    let err = SearchService::new(db.conn())
        .search(&SearchRequest::new("baseline").with_filters(SearchFilters {
            topic: Some("searchingness".to_string()),
            ..Default::default()
        }))
        .expect_err("library search should reject unsupported taxonomy values");

    assert!(
        err.to_string()
            .contains("unsupported taxonomy topic: searchingness")
    );
}

#[test]
fn library_search_rejects_invalid_rfc3339_temporal_filters() {
    let path = fresh_db_path("library-invalid-rfc3339-temporal-filter");
    let db = Database::open(&path).expect("database should open");

    let err = SearchService::new(db.conn())
        .search(&SearchRequest::new("baseline").with_filters(SearchFilters {
            valid_at: Some("not-a-time".to_string()),
            ..Default::default()
        }))
        .expect_err("library search should reject invalid RFC3339 temporal filters");

    assert!(
        err.to_string()
            .contains("invalid RFC3339 value for valid_at: not-a-time")
    );
}

#[test]
fn library_search_rejects_invalid_rfc3339_recorded_from() {
    let path = fresh_db_path("library-invalid-rfc3339-from");
    let db = Database::open(&path).expect("database should open");

    let err = SearchService::new(db.conn())
        .search(&SearchRequest::new("baseline").with_filters(SearchFilters {
            recorded_from: Some("not-a-time".to_string()),
            ..Default::default()
        }))
        .expect_err("library search should reject invalid RFC3339 recorded_from filters");

    assert!(
        err.to_string()
            .contains("invalid RFC3339 value for from: not-a-time")
    );
}

#[test]
fn library_search_rejects_invalid_rfc3339_recorded_to() {
    let path = fresh_db_path("library-invalid-rfc3339-to");
    let db = Database::open(&path).expect("database should open");

    let err = SearchService::new(db.conn())
        .search(&SearchRequest::new("baseline").with_filters(SearchFilters {
            recorded_to: Some("not-a-time".to_string()),
            ..Default::default()
        }))
        .expect_err("library search should reject invalid RFC3339 recorded_to filters");

    assert!(
        err.to_string()
            .contains("invalid RFC3339 value for to: not-a-time")
    );
}

#[test]
fn library_search_rejects_inverted_temporal_ranges() {
    let path = fresh_db_path("library-invalid-temporal-range");
    let db = Database::open(&path).expect("database should open");

    let err = SearchService::new(db.conn())
        .search(&SearchRequest::new("baseline").with_filters(SearchFilters {
            recorded_from: Some("2026-04-20T00:00:00Z".to_string()),
            recorded_to: Some("2026-04-10T00:00:00Z".to_string()),
            ..Default::default()
        }))
        .expect_err("library search should reject inverted temporal ranges");

    assert!(
        err.to_string().contains(
            "invalid temporal range: from=2026-04-20T00:00:00Z is later than to=2026-04-10T00:00:00Z"
        )
    );
}

#[test]
fn library_search_rejects_invalid_domain_topic_combinations() {
    let path = fresh_db_path("library-invalid-taxonomy-combo");
    let db = Database::open(&path).expect("database should open");

    let err = SearchService::new(db.conn())
        .search(&SearchRequest::new("baseline").with_filters(SearchFilters {
            domain: Some("project".to_string()),
            topic: Some("storage".to_string()),
            ..Default::default()
        }))
        .expect_err("library search should reject invalid taxonomy combinations");

    assert!(
        err.to_string().contains(
            "unsupported taxonomy combination: domain=project does not allow topic=storage"
        )
    );
}

#[test]
fn library_search_rejects_unknown_domain_values() {
    let path = fresh_db_path("library-invalid-domain");
    let db = Database::open(&path).expect("database should open");

    let err = SearchService::new(db.conn())
        .search(&SearchRequest::new("baseline").with_filters(SearchFilters {
            domain: Some("workspace".to_string()),
            ..Default::default()
        }))
        .expect_err("library search should reject unsupported domain values");

    assert!(
        err.to_string()
            .contains("unsupported taxonomy domain: workspace")
    );
}

#[test]
fn library_search_rejects_unknown_aspect_values() {
    let path = fresh_db_path("library-invalid-aspect");
    let db = Database::open(&path).expect("database should open");

    let err = SearchService::new(db.conn())
        .search(&SearchRequest::new("baseline").with_filters(SearchFilters {
            aspect: Some("attitude".to_string()),
            ..Default::default()
        }))
        .expect_err("library search should reject unsupported aspect values");

    assert!(
        err.to_string()
            .contains("unsupported taxonomy aspect: attitude")
    );
}

#[test]
fn library_search_rejects_unknown_kind_values() {
    let path = fresh_db_path("library-invalid-kind");
    let db = Database::open(&path).expect("database should open");

    let err = SearchService::new(db.conn())
        .search(&SearchRequest::new("baseline").with_filters(SearchFilters {
            kind: Some("decisionish".to_string()),
            ..Default::default()
        }))
        .expect_err("library search should reject unsupported kind values");

    assert!(
        err.to_string()
            .contains("unsupported taxonomy kind: decisionish")
    );
}

#[test]
fn library_search_accepts_valid_domain_topic_combinations() {
    let path = fresh_db_path("library-valid-taxonomy-combo");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/valid-combo",
            source_label: "valid combo memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T20:00:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("baseline").with_filters(SearchFilters {
            domain: Some("project".to_string()),
            topic: Some("retrieval".to_string()),
            ..Default::default()
        }))
        .expect("library search should accept valid taxonomy combinations");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/valid-combo"
    );
}

#[test]
fn library_search_accepts_valid_domain_filters() {
    let path = fresh_db_path("library-valid-domain");
    let db = Database::open(&path).expect("database should open");

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("baseline").with_filters(SearchFilters {
            domain: Some("project".to_string()),
            ..Default::default()
        }))
        .expect("library search should accept valid domain filters");

    assert!(response.results.is_empty());
    assert_eq!(response.applied_filters.domain.as_deref(), Some("project"));
}

#[test]
fn library_search_accepts_valid_topic_filters() {
    let path = fresh_db_path("library-valid-topic");
    let db = Database::open(&path).expect("database should open");

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("baseline").with_filters(SearchFilters {
            topic: Some("retrieval".to_string()),
            ..Default::default()
        }))
        .expect("library search should accept valid topic filters");

    assert!(response.results.is_empty());
    assert_eq!(response.applied_filters.topic.as_deref(), Some("retrieval"));
}

#[test]
fn library_search_accepts_valid_aspect_filters() {
    let path = fresh_db_path("library-valid-aspect");
    let db = Database::open(&path).expect("database should open");

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("baseline").with_filters(SearchFilters {
            aspect: Some("behavior".to_string()),
            ..Default::default()
        }))
        .expect("library search should accept valid aspect filters");

    assert!(response.results.is_empty());
    assert_eq!(response.applied_filters.aspect.as_deref(), Some("behavior"));
}

#[test]
fn library_search_accepts_valid_kind_filters() {
    let path = fresh_db_path("library-valid-kind");
    let db = Database::open(&path).expect("database should open");

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("baseline").with_filters(SearchFilters {
            kind: Some("decision".to_string()),
            ..Default::default()
        }))
        .expect("library search should accept valid kind filters");

    assert!(response.results.is_empty());
    assert_eq!(response.applied_filters.kind.as_deref(), Some("decision"));
}

#[test]
fn library_search_accepts_valid_domain_aspect_and_kind_filters_together() {
    let path = fresh_db_path("library-valid-domain-aspect-kind");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/domain-aspect-kind-valid",
            source_label: "domain aspect kind valid memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T19:00:00Z",
            valid_from: None,
            valid_to: None,
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/domain-aspect-kind-other",
            source_label: "domain aspect kind other memo",
            content: "retrieval status ready",
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T19:05:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("baseline").with_filters(SearchFilters {
            domain: Some("project".to_string()),
            aspect: Some("behavior".to_string()),
            kind: Some("decision".to_string()),
            ..Default::default()
        }))
        .expect("library search should accept valid domain/aspect/kind filters together");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/domain-aspect-kind-valid"
    );
    assert_eq!(response.applied_filters.domain.as_deref(), Some("project"));
    assert_eq!(response.applied_filters.aspect.as_deref(), Some("behavior"));
    assert_eq!(response.applied_filters.kind.as_deref(), Some("decision"));
}

#[test]
fn library_search_echoes_valid_taxonomy_filters_in_response() {
    let path = fresh_db_path("library-taxonomy-echo");
    let db = Database::open(&path).expect("database should open");

    let response = SearchService::new(db.conn())
        .search(&SearchRequest::new("baseline").with_filters(SearchFilters {
            domain: Some("project".to_string()),
            topic: Some("retrieval".to_string()),
            aspect: Some("behavior".to_string()),
            kind: Some("decision".to_string()),
            ..Default::default()
        }))
        .expect("library search should accept valid taxonomy filters");

    assert_eq!(response.applied_filters.domain.as_deref(), Some("project"));
    assert_eq!(response.applied_filters.topic.as_deref(), Some("retrieval"));
    assert_eq!(response.applied_filters.aspect.as_deref(), Some("behavior"));
    assert_eq!(response.applied_filters.kind.as_deref(), Some("decision"));
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
    assert!(
        search_json["results"][0]["dsl"]["claim"]
            .as_str()
            .is_some_and(|claim| !claim.is_empty()),
        "cli json should expose the structured DSL sidecar"
    );
}

#[test]
fn cli_search_json_exposes_conditional_dsl_fields() {
    let dir = unique_temp_dir("cli-json-conditional-dsl");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before conditional json check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-json-conditional",
            "--content",
            "If embedding replaces lexical baseline, recall may drift, so debugging becomes harder.",
            "--scope",
            "project",
            "--record-type",
            "observation",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T14:00:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before conditional json check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "debugging", "--json"]);
    assert!(
        search_output.status.success(),
        "cli json search should succeed for conditional dsl fields: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert!(
        search_json["results"][0]["dsl"]["cond"]
            .as_str()
            .is_some_and(|value| !value.is_empty()),
        "json output should expose DSL COND when present"
    );
    assert!(
        search_json["results"][0]["dsl"]["impact"]
            .as_str()
            .is_some_and(|value| !value.is_empty()),
        "json output should expose DSL IMPACT when present"
    );
}

#[test]
fn cli_search_json_exposes_record_shape_metadata() {
    let dir = unique_temp_dir("cli-json-record-shape");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before record-shape json check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-json-record-shape",
            "--source-label",
            "json record shape memo",
            "--content",
            "retrieval baseline keeps lexical search explainable",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T14:45:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before record-shape json check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "baseline", "--json"]);
    assert!(
        search_output.status.success(),
        "cli json search should succeed for record-shape metadata: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert!(
        search_json["results"][0]["record"]["id"]
            .as_str()
            .is_some_and(|value| value.starts_with("mem-")),
        "json output should expose the authority record id"
    );
    assert_eq!(
        search_json["results"][0]["record"]["source"]["label"],
        "json record shape memo"
    );
    assert_eq!(search_json["results"][0]["record"]["scope"], "project");
    assert_eq!(search_json["results"][0]["record"]["truth_layer"], "t2");
}

#[test]
fn cli_search_json_echoes_temporal_and_truth_filters() {
    let dir = unique_temp_dir("cli-json-filter-echo");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before filter-echo check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-json-filter-echo",
            "--content",
            "retrieval baseline keeps lexical search explainable",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T15:30:00Z",
            "--valid-from",
            "2026-04-10T00:00:00Z",
            "--valid-to",
            "2026-04-20T00:00:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before filter-echo check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(
        &config_path,
        &[
            "search",
            "baseline",
            "--truth-layer",
            "t2",
            "--valid-at",
            "2026-04-15T12:00:00Z",
            "--from",
            "2026-04-10T00:00:00Z",
            "--to",
            "2026-04-16T00:00:00Z",
            "--json",
        ],
    );
    assert!(
        search_output.status.success(),
        "cli json search should succeed for filter-echo contract: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(search_json["applied_filters"]["truth_layer"], "t2");
    assert_eq!(
        search_json["applied_filters"]["valid_at"],
        "2026-04-15T12:00:00Z"
    );
    assert_eq!(
        search_json["applied_filters"]["recorded_from"],
        "2026-04-10T00:00:00Z"
    );
    assert_eq!(
        search_json["applied_filters"]["recorded_to"],
        "2026-04-16T00:00:00Z"
    );
    assert_eq!(
        search_json["results"][0]["trace"]["applied_filters"]["truth_layer"],
        "t2"
    );
}

#[test]
fn cli_search_json_orders_recency_by_parsed_rfc3339_instant() {
    let dir = unique_temp_dir("cli-json-parsed-recency-order");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before parsed-recency check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let older_offset = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-older-offset",
            "--source-label",
            "cli older offset memo",
            "--content",
            "recency ordering should use parsed instants across offsets",
            "--scope",
            "project",
            "--record-type",
            "observation",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-16T08:00:00+02:00",
        ],
    );
    assert!(
        older_offset.status.success(),
        "cli ingest should succeed for older offset record: stdout={} stderr={}",
        stdout(&older_offset),
        stderr(&older_offset)
    );

    let newer_zulu = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-newer-zulu",
            "--source-label",
            "cli newer zulu memo",
            "--content",
            "recency ordering should use parsed instants across offsets",
            "--scope",
            "project",
            "--record-type",
            "observation",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-16T06:30:00Z",
        ],
    );
    assert!(
        newer_zulu.status.success(),
        "cli ingest should succeed for newer zulu record: stdout={} stderr={}",
        stdout(&newer_zulu),
        stderr(&newer_zulu)
    );

    let search_output = run_cli(
        &config_path,
        &["search", "parsed instants offsets", "--top-k", "2", "--json"],
    );
    assert!(
        search_output.status.success(),
        "cli json search should succeed for parsed-recency order: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );

    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(
        search_json["results"][0]["record"]["source"]["uri"],
        "memo://project/cli-newer-zulu",
        "cli search should rank the truly newer instant ahead of the lexically larger offset timestamp"
    );
}

#[test]
fn cli_search_json_applies_recorded_from_filter_by_parsed_rfc3339_instant() {
    let dir = unique_temp_dir("cli-json-parsed-recorded-from-filter");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before parsed-recorded-from check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let older_offset = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-older-offset-filter",
            "--source-label",
            "cli older offset filter memo",
            "--content",
            "temporal filtering should use parsed instants across offsets",
            "--scope",
            "project",
            "--record-type",
            "observation",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-16T08:00:00+02:00",
        ],
    );
    assert!(
        older_offset.status.success(),
        "cli ingest should succeed for older offset filter record: stdout={} stderr={}",
        stdout(&older_offset),
        stderr(&older_offset)
    );

    let newer_zulu = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-newer-zulu-filter",
            "--source-label",
            "cli newer zulu filter memo",
            "--content",
            "temporal filtering should use parsed instants across offsets",
            "--scope",
            "project",
            "--record-type",
            "observation",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-16T06:30:00Z",
        ],
    );
    assert!(
        newer_zulu.status.success(),
        "cli ingest should succeed for newer zulu filter record: stdout={} stderr={}",
        stdout(&newer_zulu),
        stderr(&newer_zulu)
    );

    let search_output = run_cli(
        &config_path,
        &[
            "search",
            "parsed instants offsets",
            "--from",
            "2026-04-16T06:15:00Z",
            "--top-k",
            "2",
            "--json",
        ],
    );
    assert!(
        search_output.status.success(),
        "cli json search should succeed for parsed recorded_from filtering: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );

    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(search_json["results"].as_array().map(Vec::len), Some(1));
    assert_eq!(
        search_json["results"][0]["record"]["source"]["uri"],
        "memo://project/cli-newer-zulu-filter",
        "recorded_from should exclude the older offset-formatted instant even when its raw string sorts later"
    );
}

#[test]
fn cli_search_json_exposes_source_kind_and_record_type() {
    let dir = unique_temp_dir("cli-json-source-kind");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before source-kind json check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-json-source-kind",
            "--source-label",
            "json source kind memo",
            "--content",
            "retrieval baseline keeps lexical search explainable",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T15:00:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before source-kind json check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "baseline", "--json"]);
    assert!(
        search_output.status.success(),
        "cli json search should succeed for source-kind metadata: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(
        search_json["results"][0]["record"]["source"]["kind"],
        "document"
    );
    assert_eq!(
        search_json["results"][0]["record"]["record_type"],
        "decision"
    );
}

#[test]
fn cli_search_json_exposes_citation_source_metadata() {
    let dir = unique_temp_dir("cli-json-citation-source");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before citation-source json check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-json-citation-source",
            "--source-label",
            "json citation source memo",
            "--content",
            "retrieval baseline keeps lexical search explainable",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T15:10:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before citation-source json check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "baseline", "--json"]);
    assert!(
        search_output.status.success(),
        "cli json search should succeed for citation-source metadata: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert!(
        search_json["results"][0]["citation"]["record_id"]
            .as_str()
            .is_some_and(|value| value.starts_with("mem-")),
        "json output should expose the citation record id"
    );
    assert_eq!(
        search_json["results"][0]["citation"]["source_kind"],
        "document"
    );
    assert_eq!(
        search_json["results"][0]["citation"]["source_label"],
        "json citation source memo"
    );
}

#[test]
fn cli_search_json_exposes_citation_recorded_at_and_anchor_shape() {
    let dir = unique_temp_dir("cli-json-citation-shape");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before citation-shape json check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-json-citation-shape",
            "--content",
            "retrieval baseline keeps lexical search explainable",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T15:12:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before citation-shape json check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "baseline", "--json"]);
    assert!(
        search_output.status.success(),
        "cli json search should succeed for citation-shape metadata: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(
        search_json["results"][0]["citation"]["recorded_at"],
        "2026-04-15T15:12:00Z"
    );
    assert_eq!(search_json["results"][0]["citation"]["anchor"]["chunk_index"], 0);
    assert_eq!(search_json["results"][0]["citation"]["anchor"]["chunk_count"], 1);
    assert_eq!(
        search_json["results"][0]["citation"]["anchor"]["anchor"]["kind"],
        "line_range"
    );
    assert_eq!(
        search_json["results"][0]["citation"]["anchor"]["anchor"]["start_line"],
        1
    );
    assert_eq!(
        search_json["results"][0]["citation"]["anchor"]["anchor"]["end_line"],
        1
    );
}

#[test]
fn cli_search_json_preserves_source_metadata_for_mixed_recall() {
    let dir = unique_temp_dir("cli-json-mixed-source-metadata");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before mixed-source json check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-json-mixed-source-metadata",
            "--source-label",
            "json mixed source memo",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T15:40:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before mixed-source json check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(
        &config_path,
        &["search", "lexical-first baseline", "--json"],
    );
    assert!(
        search_output.status.success(),
        "cli json search should succeed for mixed-source metadata: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(
        search_json["results"][0]["record"]["source"]["label"],
        "json mixed source memo"
    );
    assert!(
        search_json["results"][0]["citation"]["record_id"]
            .as_str()
            .is_some_and(|value| value.starts_with("mem-")),
        "mixed json recall should expose the authority citation record id"
    );
    assert_eq!(
        search_json["results"][0]["citation"]["source_label"],
        "json mixed source memo"
    );
    assert_eq!(
        search_json["results"][0]["record"]["source"]["kind"],
        "document"
    );
    assert_eq!(
        search_json["results"][0]["citation"]["source_kind"],
        "document"
    );
    assert_eq!(
        search_json["results"][0]["record"]["source"]["uri"],
        "memo://project/cli-json-mixed-source-metadata"
    );
    assert_eq!(search_json["results"][0]["record"]["scope"], "project");
    assert_eq!(search_json["results"][0]["record"]["truth_layer"], "t2");
    assert_eq!(search_json["results"][0]["record"]["record_type"], "decision");
    assert_eq!(
        search_json["results"][0]["citation"]["source_uri"],
        "memo://project/cli-json-mixed-source-metadata"
    );
    assert!(
        search_json["results"][0]["trace"]["query_strategies"]
            .as_array()
            .is_some_and(|strategies| {
                strategies.iter().any(|value| value == "Simple")
                    && strategies.iter().any(|value| value == "Structured")
            }),
        "mixed recall should still preserve both raw lexical and structured provenance"
    );
}

#[test]
fn cli_search_json_preserves_record_provenance_for_mixed_recall() {
    let dir = unique_temp_dir("cli-json-mixed-record-provenance");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before mixed record-provenance json check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-json-mixed-record-provenance",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T15:42:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before mixed record-provenance json check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(
        &config_path,
        &["search", "lexical-first baseline", "--json"],
    );
    assert!(
        search_output.status.success(),
        "cli json search should succeed for mixed record-provenance: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(search_json["results"][0]["record"]["provenance"]["origin"], "ingest");
    assert_eq!(
        search_json["results"][0]["record"]["provenance"]["imported_via"],
        "ingest_service"
    );
    assert!(
        search_json["results"][0]["record"]["provenance"]["derived_from"]
            .as_array()
            .is_some_and(|items| items.iter().any(|value| value
                .as_str()
                .is_some_and(|item| item.starts_with(
                    "memo://project/cli-json-mixed-record-provenance#"
                )))),
        "mixed json recall should preserve the source-derived provenance anchor"
    );
    assert!(
        search_json["results"][0]["trace"]["query_strategies"]
            .as_array()
            .is_some_and(|strategies| {
                strategies.iter().any(|value| value == "Simple")
                    && strategies.iter().any(|value| value == "Structured")
            }),
        "mixed record-provenance recall should preserve both provenance branches"
    );
}

#[test]
fn cli_search_json_preserves_citation_validity_for_mixed_recall() {
    let dir = unique_temp_dir("cli-json-mixed-citation-validity");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before mixed citation-validity check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-json-mixed-citation-validity",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T15:45:00Z",
            "--valid-from",
            "2026-04-10T00:00:00Z",
            "--valid-to",
            "2026-04-20T00:00:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before mixed citation-validity check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(
        &config_path,
        &["search", "lexical-first baseline", "--json"],
    );
    assert!(
        search_output.status.success(),
        "cli json search should succeed for mixed citation-validity: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(
        search_json["results"][0]["citation"]["validity"]["valid_from"],
        "2026-04-10T00:00:00Z"
    );
    assert_eq!(
        search_json["results"][0]["citation"]["validity"]["valid_to"],
        "2026-04-20T00:00:00Z"
    );
    assert!(
        search_json["results"][0]["trace"]["query_strategies"]
            .as_array()
            .is_some_and(|strategies| {
                strategies.iter().any(|value| value == "Simple")
                    && strategies.iter().any(|value| value == "Structured")
            }),
        "mixed citation-validity recall should preserve both provenance branches"
    );
}

#[test]
fn cli_search_json_preserves_citation_recorded_at_and_anchor_for_mixed_recall() {
    let dir = unique_temp_dir("cli-json-mixed-citation-shape");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before mixed citation-shape check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-json-mixed-citation-shape",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T15:47:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before mixed citation-shape check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(
        &config_path,
        &["search", "lexical-first baseline", "--json"],
    );
    assert!(
        search_output.status.success(),
        "cli json search should succeed for mixed citation-shape: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(
        search_json["results"][0]["citation"]["recorded_at"],
        "2026-04-15T15:47:00Z"
    );
    assert_eq!(search_json["results"][0]["citation"]["anchor"]["chunk_index"], 0);
    assert_eq!(search_json["results"][0]["citation"]["anchor"]["chunk_count"], 1);
    assert!(
        search_json["results"][0]["trace"]["query_strategies"]
            .as_array()
            .is_some_and(|strategies| {
                strategies.iter().any(|value| value == "Simple")
                    && strategies.iter().any(|value| value == "Structured")
            }),
        "mixed citation-shape recall should preserve both provenance branches"
    );
}

#[test]
fn cli_search_json_preserves_citation_record_id_for_mixed_recall() {
    let dir = unique_temp_dir("cli-json-mixed-record-id");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before mixed record-id json check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-json-mixed-record-id",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T15:48:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before mixed record-id json check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(
        &config_path,
        &["search", "lexical-first baseline", "--json"],
    );
    assert!(
        search_output.status.success(),
        "cli json search should succeed for mixed record-id contract: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert!(
        search_json["results"][0]["citation"]["record_id"]
            .as_str()
            .is_some_and(|value| value.starts_with("mem-")),
        "mixed json recall should expose the authority citation record id"
    );
    assert!(
        search_json["results"][0]["trace"]["query_strategies"]
            .as_array()
            .is_some_and(|strategies| {
                strategies.iter().any(|value| value == "Simple")
                    && strategies.iter().any(|value| value == "Structured")
            }),
        "mixed record-id recall should preserve both provenance branches"
    );
}

#[test]
fn cli_search_json_preserves_record_timestamp_for_mixed_recall() {
    let dir = unique_temp_dir("cli-json-mixed-record-timestamp");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before mixed record-timestamp json check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-json-mixed-record-timestamp",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T15:49:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before mixed record-timestamp json check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(
        &config_path,
        &["search", "lexical-first baseline", "--json"],
    );
    assert!(
        search_output.status.success(),
        "cli json search should succeed for mixed record-timestamp contract: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(
        search_json["results"][0]["record"]["timestamp"]["recorded_at"],
        "2026-04-15T15:49:00Z"
    );
    assert_eq!(
        search_json["results"][0]["record"]["timestamp"]["created_at"],
        "2026-04-15T15:49:00Z"
    );
    assert_eq!(
        search_json["results"][0]["record"]["timestamp"]["updated_at"],
        "2026-04-15T15:49:00Z"
    );
    assert!(
        search_json["results"][0]["trace"]["query_strategies"]
            .as_array()
            .is_some_and(|strategies| {
                strategies.iter().any(|value| value == "Simple")
                    && strategies.iter().any(|value| value == "Structured")
            }),
        "mixed record-timestamp recall should preserve both provenance branches"
    );
}

#[test]
fn cli_search_json_exposes_citation_validity_window() {
    let dir = unique_temp_dir("cli-json-citation-validity");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before citation-validity json check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-json-citation-validity",
            "--content",
            "retrieval baseline keeps lexical search explainable",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T15:20:00Z",
            "--valid-from",
            "2026-04-10T00:00:00Z",
            "--valid-to",
            "2026-04-20T00:00:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before citation-validity json check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "baseline", "--json"]);
    assert!(
        search_output.status.success(),
        "cli json search should succeed for citation-validity metadata: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(
        search_json["results"][0]["citation"]["validity"]["valid_from"],
        "2026-04-10T00:00:00Z"
    );
    assert_eq!(
        search_json["results"][0]["citation"]["validity"]["valid_to"],
        "2026-04-20T00:00:00Z"
    );
}

#[test]
fn cli_search_json_exposes_dsl_source_ref_and_summary_fields() {
    let dir = unique_temp_dir("cli-json-dsl-shape");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before dsl-shape json check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-json-dsl-shape",
            "--content",
            "2026-04 use lexical-first as baseline because explainability matters.",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T15:15:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before dsl-shape json check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "lexical-first", "--json"]);
    assert!(
        search_output.status.success(),
        "cli json search should succeed for dsl-shape metadata: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(
        search_json["results"][0]["dsl"]["source_ref"],
        "memo://project/cli-json-dsl-shape"
    );
    assert_eq!(search_json["results"][0]["dsl"]["domain"], "project");
    assert_eq!(search_json["results"][0]["dsl"]["kind"], "decision");
    assert_eq!(search_json["results"][0]["dsl"]["time"], "2026-04");
    assert!(
        search_json["results"][0]["dsl"]["why"]
            .as_str()
            .is_some_and(|value| !value.is_empty()),
        "json output should expose DSL WHY when present"
    );
}

#[test]
fn cli_search_json_prefers_structured_snippet_when_available() {
    let dir = unique_temp_dir("cli-json-structured-snippet");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before structured-snippet json check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-json-structured-snippet",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T15:30:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before structured-snippet json check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(
        &config_path,
        &["search", "lexical-first baseline", "--json"],
    );
    assert!(
        search_output.status.success(),
        "cli json search should succeed for structured snippet contract: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert!(
        search_json["results"][0]["snippet"]
            .as_str()
            .is_some_and(|value| value.contains("WHY:")),
        "json output should prefer the structured snippet when structured recall is present"
    );
}

#[test]
fn cli_search_json_uses_structured_snippet_for_structured_only_queries() {
    let dir = unique_temp_dir("cli-json-structured-only-snippet");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before structured-only snippet check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-json-structured-only-snippet",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T15:50:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before structured-only snippet check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "decision", "--json"]);
    assert!(
        search_output.status.success(),
        "cli json search should succeed for structured-only snippet contract: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert!(
        search_json["results"][0]["snippet"]
            .as_str()
            .is_some_and(|value| value.contains("WHY:")),
        "structured-only query should also use the structured snippet surface"
    );
}

#[test]
fn cli_search_json_preserves_both_lexical_and_structured_strategies() {
    let dir = unique_temp_dir("cli-json-mixed-strategies");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before mixed-strategy json check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-json-mixed-strategies",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T15:45:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before mixed-strategy json check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(
        &config_path,
        &["search", "lexical-first baseline", "--json"],
    );
    assert!(
        search_output.status.success(),
        "cli json search should succeed for mixed-strategy contract: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    let strategies = search_json["results"][0]["trace"]["query_strategies"]
        .as_array()
        .expect("query strategies should be an array");
    assert!(
        strategies.iter().any(|value| value == "Simple"),
        "mixed-strategy result should preserve raw lexical provenance"
    );
    assert!(
        strategies.iter().any(|value| value == "Structured"),
        "mixed-strategy result should preserve structured recall provenance"
    );
}

#[test]
fn cli_search_json_reports_mixed_hits_as_lexical_only_channel() {
    let dir = unique_temp_dir("cli-json-mixed-channel");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before mixed-channel json check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-json-mixed-channel",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T16:00:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before mixed-channel json check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(
        &config_path,
        &["search", "lexical-first baseline", "--json"],
    );
    assert!(
        search_output.status.success(),
        "cli json search should succeed for mixed-channel contract: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(
        search_json["results"][0]["trace"]["channel_contribution"],
        "lexical_only"
    );
}

#[test]
fn cli_search_json_preserves_matched_query_for_mixed_recall() {
    let dir = unique_temp_dir("cli-json-mixed-matched-query");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before mixed matched-query json check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-json-mixed-matched-query",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T16:05:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before mixed matched-query json check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(
        &config_path,
        &["search", "lexical-first baseline", "--json"],
    );
    assert!(
        search_output.status.success(),
        "cli json search should succeed for mixed matched-query contract: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(
        search_json["results"][0]["trace"]["matched_query"],
        "lexical-first baseline"
    );
    assert!(
        search_json["results"][0]["trace"]["query_strategies"]
            .as_array()
            .is_some_and(|strategies| {
                strategies.iter().any(|value| value == "Simple")
                    && strategies.iter().any(|value| value == "Structured")
            }),
        "mixed matched-query recall should preserve both provenance branches"
    );
}

#[test]
fn cli_search_json_keeps_structured_snippet_and_mixed_strategies_in_sync() {
    let dir = unique_temp_dir("cli-json-mixed-contract");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before mixed-contract json check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-json-mixed-contract",
            "--source-label",
            "json mixed contract memo",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T15:55:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before mixed-contract json check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(
        &config_path,
        &["search", "lexical-first baseline", "--json"],
    );
    assert!(
        search_output.status.success(),
        "cli json search should succeed for mixed-contract check: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert!(
        search_json["results"][0]["snippet"]
            .as_str()
            .is_some_and(|value| value.contains("WHY:")),
        "mixed-hit json output should keep the structured snippet surface"
    );
    let strategies = search_json["results"][0]["trace"]["query_strategies"]
        .as_array()
        .expect("query strategies should be an array");
    assert!(
        strategies.iter().any(|value| value == "Simple")
            && strategies.iter().any(|value| value == "Structured"),
        "mixed-hit json output should keep both lexical and structured provenance"
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
fn cli_search_text_output_renders_structured_dsl_summary() {
    let dir = unique_temp_dir("cli-text-dsl");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before text dsl check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-text-dsl",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T11:30:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before text dsl check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "decision"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for dsl rendering: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("dsl:"),
        "text output should render the structured DSL summary: {text}"
    );
    assert!(
        text.contains("decision |"),
        "text output should include taxonomy and claim in the DSL summary: {text}"
    );
    assert!(
        text.contains("WHY:"),
        "text output should include compact DSL reasoning fields when present: {text}"
    );
    assert!(
        text.contains("SRC:"),
        "text output should include the structured source reference in the DSL summary: {text}"
    );
}

#[test]
fn cli_search_text_output_renders_taxonomy_filters() {
    let dir = unique_temp_dir("cli-text-taxonomy-filters");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before text taxonomy filter check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-text-taxonomy",
            "--content",
            "retrieval baseline keeps lexical search explainable",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T11:45:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before text taxonomy filter check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(
        &config_path,
        &[
            "search",
            "baseline",
            "--topic",
            "retrieval",
            "--kind",
            "decision",
        ],
    );
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for taxonomy filter display: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("topic=retrieval") && text.contains("kind=decision"),
        "text output should display taxonomy filters in the filter summary: {text}"
    );
}

#[test]
fn cli_search_json_applies_taxonomy_filters_before_top_k_truncation() {
    let dir = unique_temp_dir("cli-taxonomy-filter-before-top-k");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before taxonomy/top-k check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let config_ingest = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-config-top-k",
            "--source-label",
            "cli config top k memo",
            "--content",
            "baseline baseline keeps toml setting review stable",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-16T09:06:00Z",
        ],
    );
    assert!(
        config_ingest.status.success(),
        "cli ingest should succeed for non-matching top-k candidate: stdout={} stderr={}",
        stdout(&config_ingest),
        stderr(&config_ingest)
    );

    let retrieval_ingest = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-retrieval-top-k",
            "--source-label",
            "cli retrieval top k memo",
            "--content",
            "baseline keeps lexical search explainable",
            "--scope",
            "project",
            "--record-type",
            "observation",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-16T09:00:00Z",
        ],
    );
    assert!(
        retrieval_ingest.status.success(),
        "cli ingest should succeed for matching top-k candidate: stdout={} stderr={}",
        stdout(&retrieval_ingest),
        stderr(&retrieval_ingest)
    );

    let output = run_cli(
        &config_path,
        &[
            "search",
            "baseline",
            "--topic",
            "retrieval",
            "--top-k",
            "1",
            "--json",
        ],
    );
    assert!(
        output.status.success(),
        "cli search should succeed for taxonomy/top-k contract: stdout={} stderr={}",
        stdout(&output),
        stderr(&output)
    );

    let search_json: Value =
        serde_json::from_str(&stdout(&output)).expect("search should emit json");
    assert_eq!(search_json["results"].as_array().map(Vec::len), Some(1));
    assert_eq!(
        search_json["results"][0]["record"]["source"]["uri"],
        "memo://project/cli-retrieval-top-k",
        "taxonomy filtering should be applied before top-k truncation in cli search"
    );
}

#[test]
fn cli_search_json_truncates_final_results_to_requested_top_k() {
    let dir = unique_temp_dir("cli-final-top-k-truncation");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before final top-k check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let lexical_ingest = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-lexical-top-k",
            "--source-label",
            "cli lexical top k memo",
            "--content",
            "decision procedures keep lexical ranking inspectable",
            "--scope",
            "project",
            "--record-type",
            "observation",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-16T09:40:00Z",
        ],
    );
    assert!(
        lexical_ingest.status.success(),
        "cli ingest should succeed for lexical top-k candidate: stdout={} stderr={}",
        stdout(&lexical_ingest),
        stderr(&lexical_ingest)
    );

    let structured_ingest = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-structured-top-k",
            "--source-label",
            "cli structured top k memo",
            "--content",
            "baseline remains stable without the keyword",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-16T09:41:00Z",
        ],
    );
    assert!(
        structured_ingest.status.success(),
        "cli ingest should succeed for structured top-k candidate: stdout={} stderr={}",
        stdout(&structured_ingest),
        stderr(&structured_ingest)
    );

    let output = run_cli(
        &config_path,
        &["search", "decision", "--top-k", "1", "--json"],
    );
    assert!(
        output.status.success(),
        "cli search should succeed for final top-k contract: stdout={} stderr={}",
        stdout(&output),
        stderr(&output)
    );

    let search_json: Value =
        serde_json::from_str(&stdout(&output)).expect("search should emit json");
    assert_eq!(
        search_json["results"].as_array().map(Vec::len),
        Some(1),
        "cli search should truncate merged recall results to the requested top-k"
    );
}

#[test]
fn cli_search_json_clamps_zero_top_k_to_one_result() {
    let dir = unique_temp_dir("cli-zero-top-k-clamp");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before zero-top-k check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-zero-top-k",
            "--source-label",
            "cli zero top k memo",
            "--content",
            "baseline retrieval should still return one bounded result",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-16T09:42:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before zero-top-k search: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let output = run_cli(
        &config_path,
        &["search", "bounded result", "--top-k", "0", "--json"],
    );
    assert!(
        output.status.success(),
        "cli search should succeed for zero-top-k clamp contract: stdout={} stderr={}",
        stdout(&output),
        stderr(&output)
    );

    let search_json: Value =
        serde_json::from_str(&stdout(&output)).expect("search should emit json");
    assert_eq!(
        search_json["results"].as_array().map(Vec::len),
        Some(1),
        "cli top-k=0 should clamp to one result instead of returning an empty response"
    );
}

#[test]
fn cli_search_json_returns_empty_results_for_whitespace_query() {
    let dir = unique_temp_dir("cli-whitespace-query");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before whitespace-query check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-whitespace-query",
            "--source-label",
            "cli whitespace query memo",
            "--content",
            "baseline retrieval should stay stable when the query is blank",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-16T09:43:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before whitespace-query search: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let output = run_cli(&config_path, &["search", "   ", "--json"]);
    assert!(
        output.status.success(),
        "cli search should accept whitespace-only query input: stdout={} stderr={}",
        stdout(&output),
        stderr(&output)
    );

    let search_json: Value =
        serde_json::from_str(&stdout(&output)).expect("search should emit json");
    assert_eq!(
        search_json["results"].as_array().map(Vec::len),
        Some(0),
        "whitespace-only cli query should return an empty result set instead of a misleading match"
    );
}

#[test]
fn cli_search_text_returns_empty_results_for_whitespace_query() {
    let dir = unique_temp_dir("cli-text-whitespace-query");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before text whitespace-query check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-text-whitespace-query",
            "--source-label",
            "cli text whitespace query memo",
            "--content",
            "blank text queries should not fabricate visible result rows",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-16T09:44:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before text whitespace-query search: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let output = run_cli(&config_path, &["search", "   "]);
    let text = stdout(&output);
    assert!(
        output.status.success(),
        "cli text search should accept whitespace-only query input: stdout={text} stderr={}",
        stderr(&output)
    );
    assert!(
        text.contains("results: 0"),
        "text output should report an empty result count for whitespace-only queries: {text}"
    );
    assert!(
        !text.contains("1. memo://"),
        "text output should not fabricate visible result rows for whitespace-only queries: {text}"
    );
}

#[test]
fn cli_search_text_clamps_zero_top_k_to_one_result() {
    let dir = unique_temp_dir("cli-text-zero-top-k-clamp");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before text zero-top-k check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-text-zero-top-k",
            "--source-label",
            "cli text zero top k memo",
            "--content",
            "bounded text recall should still show one result when top-k is zero",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-16T09:45:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before text zero-top-k search: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let output = run_cli(&config_path, &["search", "bounded text recall", "--top-k", "0"]);
    let text = stdout(&output);
    assert!(
        output.status.success(),
        "cli text search should succeed for zero-top-k clamp contract: stdout={text} stderr={}",
        stderr(&output)
    );
    assert!(
        text.contains("results: 1"),
        "text output should clamp top-k=0 to one visible result: {text}"
    );
    assert!(
        text.contains("1. memo://project/cli-text-zero-top-k"),
        "text output should still render the first result when top-k is clamped from zero: {text}"
    );
}

#[test]
fn cli_search_text_clamps_excessive_top_k_to_max_recall_limit() {
    let dir = unique_temp_dir("cli-text-excessive-top-k-clamp");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before text excessive-top-k check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    for index in 0..30 {
        let ingest_output = run_cli(
            &config_path,
            &[
                "ingest",
                "--source-uri",
                &format!("memo://project/cli-text-top-k-{index}"),
                "--source-label",
                "cli text excessive top k memo",
                "--content",
                "bounded text recall should clamp excessive top-k requests",
                "--scope",
                "project",
                "--record-type",
                "observation",
                "--truth-layer",
                "t2",
                "--recorded-at",
                &format!("2026-04-16T09:{index:02}:00Z"),
            ],
        );
        assert!(
            ingest_output.status.success(),
            "cli ingest should succeed for text excessive-top-k corpus: stdout={} stderr={}",
            stdout(&ingest_output),
            stderr(&ingest_output)
        );
    }

    let output = run_cli(&config_path, &["search", "bounded text recall", "--top-k", "999"]);
    let text = stdout(&output);
    assert!(
        output.status.success(),
        "cli text search should succeed for excessive-top-k clamp contract: stdout={text} stderr={}",
        stderr(&output)
    );
    assert!(
        text.contains(&format!("results: {}", agent_memos::search::lexical::MAX_RECALL_LIMIT)),
        "text output should clamp excessive top-k to MAX_RECALL_LIMIT: {text}"
    );
}

#[test]
fn cli_search_json_clamps_excessive_top_k_to_max_recall_limit() {
    let dir = unique_temp_dir("cli-excessive-top-k-clamp");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before excessive-top-k check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    for index in 0..30 {
        let ingest_output = run_cli(
            &config_path,
            &[
                "ingest",
                "--source-uri",
                &format!("memo://project/cli-top-k-{index}"),
                "--source-label",
                "cli excessive top k memo",
                "--content",
                "bounded recall should clamp excessive top-k requests",
                "--scope",
                "project",
                "--record-type",
                "observation",
                "--truth-layer",
                "t2",
                "--recorded-at",
                &format!("2026-04-16T09:{index:02}:00Z"),
            ],
        );
        assert!(
            ingest_output.status.success(),
            "cli ingest should succeed for excessive-top-k corpus: stdout={} stderr={}",
            stdout(&ingest_output),
            stderr(&ingest_output)
        );
    }

    let output = run_cli(
        &config_path,
        &["search", "bounded recall", "--top-k", "999", "--json"],
    );
    assert!(
        output.status.success(),
        "cli search should succeed for excessive-top-k clamp contract: stdout={} stderr={}",
        stdout(&output),
        stderr(&output)
    );

    let search_json: Value =
        serde_json::from_str(&stdout(&output)).expect("search should emit json");
    assert_eq!(
        search_json["results"].as_array().map(Vec::len),
        Some(MAX_RECALL_LIMIT),
        "cli excessive top-k should clamp to MAX_RECALL_LIMIT instead of returning the full corpus"
    );
}

#[test]
fn cli_search_text_output_renders_domain_aspect_and_kind_filters() {
    let dir = unique_temp_dir("cli-text-domain-aspect-kind");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before domain/aspect/kind text check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-text-domain-aspect-kind",
            "--content",
            "retrieval baseline keeps lexical search explainable",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T11:50:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before domain/aspect/kind text check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(
        &config_path,
        &[
            "search", "baseline", "--domain", "project", "--aspect", "behavior", "--kind",
            "decision",
        ],
    );
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for domain/aspect/kind filter display: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("domain=project")
            && text.contains("aspect=behavior")
            && text.contains("kind=decision"),
        "text output should display domain/aspect/kind filters in the filter summary: {text}"
    );
}

#[test]
fn cli_search_text_output_renders_channel_and_strategy_summary() {
    let dir = unique_temp_dir("cli-text-channel-summary");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before channel summary check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-text-channel",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T12:15:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before channel summary check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "decision"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for channel summary: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("channel: lexical_only") && text.contains("strategies=structured"),
        "text output should expose the recall channel and strategies summary: {text}"
    );
}

#[test]
fn cli_search_text_output_renders_mixed_raw_and_structured_strategies() {
    let dir = unique_temp_dir("cli-text-mixed-strategies");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before mixed-strategy text check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-text-mixed-strategies",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T12:20:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before mixed-strategy text check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "lexical-first baseline"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for mixed-strategy summary: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("channel: lexical_only")
            && text.contains("simple")
            && text.contains("structured"),
        "text output should expose mixed raw lexical + structured strategies: {text}"
    );
}

#[test]
fn cli_search_text_prefers_structured_snippet_when_mixed_recall_occurs() {
    let dir = unique_temp_dir("cli-text-mixed-structured-snippet");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before mixed structured-snippet check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-text-mixed-structured-snippet",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T12:25:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before mixed structured-snippet check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "lexical-first baseline"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for mixed structured-snippet contract: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains(
            "snippet: use lexical-first as baseline because explainability matters | WHY:"
        ),
        "text output should prefer the structured snippet when mixed recall occurs: {text}"
    );
}

#[test]
fn cli_search_text_preserves_record_and_citation_shape_for_mixed_recall() {
    let dir = unique_temp_dir("cli-text-mixed-record-citation-shape");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before mixed record/citation text check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-text-mixed-record-citation-shape",
            "--source-label",
            "mixed text shape memo",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T12:27:00Z",
            "--valid-from",
            "2026-04-10T00:00:00Z",
            "--valid-to",
            "2026-04-20T00:00:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before mixed record/citation text check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "lexical-first baseline"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for mixed record/citation shape: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("channel: lexical_only")
            && text.contains("simple")
            && text.contains("structured"),
        "mixed text recall should preserve both lexical and structured provenance: {text}"
    );
    assert!(
        text.contains("record: id=mem-")
            && text.contains("scope=project")
            && text.contains("type=decision truth_layer=t2"),
        "mixed text recall should preserve the authority record summary: {text}"
    );
    assert!(
        text.contains("kind=document")
            && text.contains("label=mixed text shape memo"),
        "mixed text recall should preserve source kind and label in the record summary: {text}"
    );
    assert!(
        text.contains("citation: chunk 1/1 recorded_at=2026-04-15T12:27:00Z")
            && text.contains("valid_from=2026-04-10T00:00:00Z")
            && text.contains("valid_to=2026-04-20T00:00:00Z"),
        "mixed text recall should preserve citation shape and validity on the text surface: {text}"
    );
}

#[test]
fn cli_search_text_preserves_record_provenance_for_mixed_recall() {
    let dir = unique_temp_dir("cli-text-mixed-record-provenance");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before mixed text record-provenance check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-text-mixed-record-provenance",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T12:28:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before mixed text record-provenance check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "lexical-first baseline"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for mixed text record-provenance: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("channel: lexical_only")
            && text.contains("simple")
            && text.contains("structured"),
        "mixed text recall should preserve both lexical and structured provenance branches: {text}"
    );
    assert!(
        text.contains("provenance: origin=ingest imported_via=ingest_service")
            && text.contains("memo://project/cli-text-mixed-record-provenance#"),
        "mixed text recall should preserve record provenance in the text surface: {text}"
    );
}

#[test]
fn cli_search_text_uses_structured_snippet_for_structured_only_queries() {
    let dir = unique_temp_dir("cli-text-structured-only-snippet");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before structured-only text snippet check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-text-structured-only-snippet",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T12:35:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before structured-only text snippet check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "decision"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for structured-only snippet contract: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains(
            "snippet: use lexical-first as baseline because explainability matters | WHY:"
        ),
        "text output should use the structured snippet for structured-only queries too: {text}"
    );
}

#[test]
fn cli_search_text_preserves_citation_shape_for_structured_only_recall() {
    let dir = unique_temp_dir("cli-text-structured-only-citation-shape");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before structured-only text citation-shape check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-text-structured-only-citation-shape",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T12:36:00Z",
            "--valid-from",
            "2026-04-10T00:00:00Z",
            "--valid-to",
            "2026-04-20T00:00:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before structured-only text citation-shape check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "decision"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for structured-only citation-shape: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("channel: lexical_only") && text.contains("strategies=structured"),
        "structured-only text recall should preserve lexical-first structured provenance: {text}"
    );
    assert!(
        text.contains("citation: chunk 1/1 recorded_at=2026-04-15T12:36:00Z"),
        "structured-only text recall should preserve citation chunk shape and recorded_at: {text}"
    );
    assert!(
        text.contains("valid_from=2026-04-10T00:00:00Z")
            && text.contains("valid_to=2026-04-20T00:00:00Z"),
        "structured-only text recall should preserve citation validity in the text surface: {text}"
    );
}

#[test]
fn cli_search_text_preserves_record_summary_for_structured_only_recall() {
    let dir = unique_temp_dir("cli-text-structured-only-record-shape");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before structured-only text record-shape check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-text-structured-only-record-shape",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T12:37:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before structured-only text record-shape check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "decision"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for structured-only record-shape: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("record: id=mem-")
            && text.contains("scope=project")
            && text.contains("type=decision truth_layer=t2"),
        "structured-only text recall should preserve the authority record summary: {text}"
    );
}

#[test]
fn cli_search_text_preserves_source_metadata_for_structured_only_recall() {
    let dir = unique_temp_dir("cli-text-structured-only-source-metadata");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before structured-only text source-metadata check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-text-structured-only-source-metadata",
            "--source-label",
            "structured-only text source metadata memo",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T12:38:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before structured-only text source-metadata check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "decision"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for structured-only source-metadata: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("kind=document")
            && text.contains("label=structured-only text source metadata memo"),
        "structured-only text recall should preserve source kind and label in the record summary: {text}"
    );
}

#[test]
fn cli_search_text_preserves_record_provenance_for_structured_only_recall() {
    let dir = unique_temp_dir("cli-text-structured-only-record-provenance");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before structured-only text record-provenance check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-text-structured-only-record-provenance",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T12:38:30Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before structured-only text record-provenance check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "decision"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for structured-only text record-provenance: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("provenance: origin=ingest imported_via=ingest_service")
            && text.contains("memo://project/cli-text-structured-only-record-provenance#"),
        "structured-only text recall should preserve record provenance in the text surface: {text}"
    );
}

#[test]
fn cli_search_text_output_renders_conditional_dsl_fields() {
    let dir = unique_temp_dir("cli-text-conditional-dsl");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before conditional dsl check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-text-conditional",
            "--content",
            "If embedding replaces lexical baseline, recall may drift, so debugging becomes harder.",
            "--scope",
            "project",
            "--record-type",
            "observation",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T12:45:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before conditional dsl check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "debugging"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for conditional dsl rendering: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("COND:") && text.contains("IMPACT:"),
        "text output should render compact conditional DSL fields when present: {text}"
    );
}

#[test]
fn cli_search_text_output_renders_record_type_and_truth_layer() {
    let dir = unique_temp_dir("cli-text-record-shape");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before record-shape check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-text-record-shape",
            "--content",
            "retrieval baseline keeps lexical search explainable",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T13:00:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before record-shape check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "baseline"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for record-shape display: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("record: id=") && text.contains("type=decision truth_layer=t2"),
        "text output should expose record type and truth layer for each result: {text}"
    );
    assert!(
        text.contains("record: id=mem-"),
        "text output should expose the authority record id for direct lookup: {text}"
    );
}

#[test]
fn cli_search_text_output_renders_source_kind_and_label() {
    let dir = unique_temp_dir("cli-text-source-shape");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before source-shape check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-text-source-shape",
            "--source-label",
            "source shape memo",
            "--content",
            "retrieval baseline keeps lexical search explainable",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T13:15:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before source-shape check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "baseline"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for source-shape display: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("kind=document") || text.contains("kind=note"),
        "text output should expose the persisted source kind: {text}"
    );
    assert!(
        text.contains("label=source shape memo"),
        "text output should expose the persisted source label: {text}"
    );
}

#[test]
fn cli_search_text_output_renders_source_uri() {
    let dir = unique_temp_dir("cli-text-source-uri");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before source-uri check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-text-source-uri",
            "--source-label",
            "source uri memo",
            "--content",
            "retrieval baseline keeps lexical search explainable",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T13:20:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before source-uri check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "baseline"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for source-uri display: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("1. memo://project/cli-text-source-uri"),
        "text output should expose the authority source uri in the result header: {text}"
    );
}

#[test]
fn cli_search_text_output_renders_validity_window() {
    let dir = unique_temp_dir("cli-text-validity-window");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before validity-window check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-text-validity",
            "--content",
            "retrieval baseline keeps lexical search explainable",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T14:30:00Z",
            "--valid-from",
            "2026-04-10T00:00:00Z",
            "--valid-to",
            "2026-04-20T00:00:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before validity-window check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "baseline"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for validity-window display: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("valid_from=2026-04-10T00:00:00Z")
            && text.contains("valid_to=2026-04-20T00:00:00Z"),
        "text output should render the explicit validity window in the citation line: {text}"
    );
}

#[test]
fn cli_search_text_output_renders_recorded_at_in_citation_line() {
    let dir = unique_temp_dir("cli-text-recorded-at");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before recorded-at check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-text-recorded-at",
            "--content",
            "retrieval baseline keeps lexical search explainable",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T14:35:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before recorded-at check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "baseline"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for recorded-at display: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("recorded_at=2026-04-15T14:35:00Z"),
        "text output should render recorded_at in the citation line: {text}"
    );
}

#[test]
fn cli_search_text_output_renders_citation_chunk_shape() {
    let dir = unique_temp_dir("cli-text-citation-chunk");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before citation-chunk check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-text-citation-chunk",
            "--content",
            "retrieval baseline keeps lexical search explainable",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T14:40:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before citation-chunk check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "baseline"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for citation-chunk display: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("citation: chunk 1/1"),
        "text output should render the citation chunk anchor shape: {text}"
    );
}

#[test]
fn cli_search_text_output_renders_scope_in_record_summary() {
    let dir = unique_temp_dir("cli-text-scope-shape");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before scope-shape check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-text-scope-shape",
            "--content",
            "retrieval baseline keeps lexical search explainable",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T13:45:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before scope-shape check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "baseline"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for scope-shape display: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("scope=project"),
        "text output should expose the persisted scope in the record summary: {text}"
    );
}

#[test]
fn cli_search_text_output_renders_dsl_source_ref() {
    let dir = unique_temp_dir("cli-text-dsl-source-ref");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before dsl source-ref check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest_output = run_cli(
        &config_path,
        &[
            "ingest",
            "--source-uri",
            "memo://project/cli-text-dsl-source",
            "--content",
            "use lexical-first as baseline because explainability matters",
            "--scope",
            "project",
            "--record-type",
            "decision",
            "--truth-layer",
            "t2",
            "--recorded-at",
            "2026-04-15T13:30:00Z",
        ],
    );
    assert!(
        ingest_output.status.success(),
        "cli ingest should succeed before dsl source-ref check: stdout={} stderr={}",
        stdout(&ingest_output),
        stderr(&ingest_output)
    );

    let search_output = run_cli(&config_path, &["search", "decision"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for dsl source-ref display: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("SRC: memo://project/cli-text-dsl-source"),
        "text output should expose the structured source reference in the DSL summary: {text}"
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
        &[
            "search",
            "retrieval fusion",
            "--mode",
            "embedding_only",
            "--json",
        ],
    );
    let embedding_json: Value = serde_json::from_str(&stdout(&embedding_output))
        .expect("embedding search should emit json");
    assert_eq!(
        embedding_json["results"][0]["trace"]["channel_contribution"],
        "embedding_only"
    );
    let embedding_strategies = embedding_json["results"][0]["trace"]["query_strategies"]
        .as_array()
        .expect("embedding query_strategies should be an array");
    assert_eq!(
        embedding_strategies,
        &vec![Value::String("Embedding".to_string())],
        "embedding_only json output should expose only the embedding strategy when the second channel is ready"
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
    let hybrid_strategies = hybrid_json["results"][0]["trace"]["query_strategies"]
        .as_array()
        .expect("hybrid query_strategies should be an array");
    assert!(
        hybrid_strategies.iter().any(|value| value == "Embedding"),
        "hybrid json output should expose embedding participation when the second channel is ready: {}",
        stdout(&hybrid_output)
    );
}

#[test]
fn cli_search_json_exposes_dsl_sidecar_for_embedding_only_ready_path() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("cli-json-embedding-only-ready-dsl");
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
            source_uri: "memo://project/cli-json-embedding-only-ready-dsl",
            source_label: "cli json embedding only ready dsl memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:09:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(
        &config_path,
        &[
            "search",
            "retrieval fusion",
            "--mode",
            "embedding_only",
            "--json",
        ],
    );
    assert!(
        search_output.status.success(),
        "cli json search should succeed for embedding_only ready-path dsl: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );

    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(
        search_json["results"][0]["trace"]["channel_contribution"],
        "embedding_only"
    );
    assert_eq!(search_json["results"][0]["dsl"]["domain"], "project");
    assert_eq!(search_json["results"][0]["dsl"]["kind"], "decision");
    assert_eq!(
        search_json["results"][0]["dsl"]["source_ref"],
        "memo://project/cli-json-embedding-only-ready-dsl"
    );
}

#[test]
fn cli_search_json_exposes_dsl_sidecar_for_hybrid_ready_path() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("cli-json-hybrid-ready-dsl");
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
            source_uri: "memo://project/cli-json-hybrid-ready-dsl",
            source_label: "cli json hybrid ready dsl memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:10:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(
        &config_path,
        &["search", "retrieval fusion", "--mode", "hybrid", "--json"],
    );
    assert!(
        search_output.status.success(),
        "cli json search should succeed for hybrid ready-path dsl: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );

    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(search_json["results"][0]["trace"]["channel_contribution"], "hybrid");
    assert_eq!(search_json["results"][0]["dsl"]["domain"], "project");
    assert_eq!(search_json["results"][0]["dsl"]["kind"], "decision");
    assert_eq!(
        search_json["results"][0]["dsl"]["source_ref"],
        "memo://project/cli-json-hybrid-ready-dsl"
    );
}

#[test]
fn cli_search_json_embedding_only_applies_taxonomy_filters_before_top_k_when_ready() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("cli-json-embedding-only-taxonomy-top-k");
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
            source_uri: "memo://project/cli-json-embedding-only-config-top-k",
            source_label: "cli json embedding_only config top k memo",
            content: "baseline baseline keeps toml setting review stable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:09:30Z",
            valid_from: None,
            valid_to: None,
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/cli-json-embedding-only-retrieval-top-k",
            source_label: "cli json embedding_only retrieval top k memo",
            content: "baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:09:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(
        &config_path,
        &[
            "search",
            "baseline",
            "--mode",
            "embedding_only",
            "--topic",
            "retrieval",
            "--top-k",
            "1",
            "--json",
        ],
    );
    assert!(
        search_output.status.success(),
        "cli json embedding_only taxonomy/top-k search should succeed: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );

    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(
        search_json["results"][0]["record"]["source"]["uri"],
        "memo://project/cli-json-embedding-only-retrieval-top-k"
    );
    assert_eq!(
        search_json["results"][0]["trace"]["channel_contribution"],
        "embedding_only"
    );
    assert_eq!(search_json["applied_filters"]["topic"], "retrieval");
}

#[test]
fn cli_search_json_hybrid_applies_taxonomy_filters_before_top_k_when_ready() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("cli-json-hybrid-taxonomy-top-k");
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
            source_uri: "memo://project/cli-json-hybrid-config-top-k",
            source_label: "cli json hybrid config top k memo",
            content: "baseline baseline keeps toml setting review stable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:10:30Z",
            valid_from: None,
            valid_to: None,
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/cli-json-hybrid-retrieval-top-k",
            source_label: "cli json hybrid retrieval top k memo",
            content: "baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:10:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(
        &config_path,
        &[
            "search",
            "baseline",
            "--mode",
            "hybrid",
            "--topic",
            "retrieval",
            "--top-k",
            "1",
            "--json",
        ],
    );
    assert!(
        search_output.status.success(),
        "cli json hybrid taxonomy/top-k search should succeed: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );

    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(
        search_json["results"][0]["record"]["source"]["uri"],
        "memo://project/cli-json-hybrid-retrieval-top-k"
    );
    assert_eq!(search_json["results"][0]["trace"]["channel_contribution"], "hybrid");
    assert_eq!(search_json["applied_filters"]["topic"], "retrieval");
}

#[test]
fn cli_search_json_embedding_only_applies_temporal_filters_before_top_k_when_ready() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("cli-json-embedding-only-temporal-top-k");
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
            source_uri: "memo://project/cli-json-embedding-only-current-temporal",
            source_label: "cli json embedding_only current temporal memo",
            content: "baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:11:30Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/cli-json-embedding-only-stale-temporal",
            source_label: "cli json embedding_only stale temporal memo",
            content: "baseline baseline keeps stale review around",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-01T10:11:30Z",
            valid_from: Some("2026-03-01T00:00:00Z"),
            valid_to: Some("2026-04-05T00:00:00Z"),
        },
    );

    let search_output = run_cli(
        &config_path,
        &[
            "search",
            "baseline",
            "--mode",
            "embedding_only",
            "--valid-at",
            "2026-04-17T12:00:00Z",
            "--from",
            "2026-04-10T00:00:00Z",
            "--to",
            "2026-04-18T00:00:00Z",
            "--top-k",
            "1",
            "--json",
        ],
    );
    assert!(
        search_output.status.success(),
        "cli json embedding_only temporal/top-k search should succeed: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );

    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(
        search_json["results"][0]["record"]["source"]["uri"],
        "memo://project/cli-json-embedding-only-current-temporal"
    );
    assert_eq!(
        search_json["results"][0]["trace"]["channel_contribution"],
        "embedding_only"
    );
    assert_eq!(search_json["applied_filters"]["valid_at"], "2026-04-17T12:00:00Z");
}

#[test]
fn cli_search_json_hybrid_applies_temporal_filters_before_top_k_when_ready() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("cli-json-hybrid-temporal-top-k");
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
            source_uri: "memo://project/cli-json-hybrid-current-temporal",
            source_label: "cli json hybrid current temporal memo",
            content: "baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:12:30Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/cli-json-hybrid-stale-temporal",
            source_label: "cli json hybrid stale temporal memo",
            content: "baseline baseline keeps stale review around",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-01T10:12:30Z",
            valid_from: Some("2026-03-01T00:00:00Z"),
            valid_to: Some("2026-04-05T00:00:00Z"),
        },
    );

    let search_output = run_cli(
        &config_path,
        &[
            "search",
            "baseline",
            "--mode",
            "hybrid",
            "--valid-at",
            "2026-04-17T12:00:00Z",
            "--from",
            "2026-04-10T00:00:00Z",
            "--to",
            "2026-04-18T00:00:00Z",
            "--top-k",
            "1",
            "--json",
        ],
    );
    assert!(
        search_output.status.success(),
        "cli json hybrid temporal/top-k search should succeed: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );

    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(
        search_json["results"][0]["record"]["source"]["uri"],
        "memo://project/cli-json-hybrid-current-temporal"
    );
    assert_eq!(search_json["results"][0]["trace"]["channel_contribution"], "hybrid");
    assert_eq!(search_json["applied_filters"]["valid_at"], "2026-04-17T12:00:00Z");
}

#[test]
fn cli_search_json_configured_embedding_only_applies_temporal_filters_before_top_k() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("configured-embedding-only-json-temporal-top-k");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "embedding_only",
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
            source_uri: "memo://project/configured-embedding-only-current-temporal",
            source_label: "configured embedding_only current temporal memo",
            content: "baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:13:30Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/configured-embedding-only-stale-temporal",
            source_label: "configured embedding_only stale temporal memo",
            content: "baseline baseline keeps stale review around",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-01T10:13:30Z",
            valid_from: Some("2026-03-01T00:00:00Z"),
            valid_to: Some("2026-04-05T00:00:00Z"),
        },
    );

    let search_output = run_cli(
        &config_path,
        &[
            "search",
            "baseline",
            "--valid-at",
            "2026-04-17T12:00:00Z",
            "--from",
            "2026-04-10T00:00:00Z",
            "--to",
            "2026-04-18T00:00:00Z",
            "--top-k",
            "1",
            "--json",
        ],
    );
    assert!(
        search_output.status.success(),
        "configured embedding_only json temporal/top-k search should succeed: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );

    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(
        search_json["results"][0]["record"]["source"]["uri"],
        "memo://project/configured-embedding-only-current-temporal"
    );
    assert_eq!(
        search_json["results"][0]["trace"]["channel_contribution"],
        "embedding_only"
    );
    assert_eq!(search_json["applied_filters"]["valid_at"], "2026-04-17T12:00:00Z");
}

#[test]
fn cli_search_json_configured_hybrid_applies_temporal_filters_before_top_k() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("configured-hybrid-json-temporal-top-k");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "hybrid",
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
            source_uri: "memo://project/configured-hybrid-current-temporal",
            source_label: "configured hybrid current temporal memo",
            content: "baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:14:30Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/configured-hybrid-stale-temporal",
            source_label: "configured hybrid stale temporal memo",
            content: "baseline baseline keeps stale review around",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-01T10:14:30Z",
            valid_from: Some("2026-03-01T00:00:00Z"),
            valid_to: Some("2026-04-05T00:00:00Z"),
        },
    );

    let search_output = run_cli(
        &config_path,
        &[
            "search",
            "baseline",
            "--valid-at",
            "2026-04-17T12:00:00Z",
            "--from",
            "2026-04-10T00:00:00Z",
            "--to",
            "2026-04-18T00:00:00Z",
            "--top-k",
            "1",
            "--json",
        ],
    );
    assert!(
        search_output.status.success(),
        "configured hybrid json temporal/top-k search should succeed: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );

    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(
        search_json["results"][0]["record"]["source"]["uri"],
        "memo://project/configured-hybrid-current-temporal"
    );
    assert_eq!(search_json["results"][0]["trace"]["channel_contribution"], "hybrid");
    assert_eq!(search_json["applied_filters"]["valid_at"], "2026-04-17T12:00:00Z");
}

#[test]
fn cli_search_json_preserves_record_and_citation_shape_for_embedding_only_ready_path() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("cli-json-embedding-only-citation-shape");
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
            source_uri: "memo://project/cli-json-embedding-only-citation-shape",
            source_label: "cli json embedding_only citation memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:19:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let search_output = run_cli(
        &config_path,
        &[
            "search",
            "retrieval fusion",
            "--mode",
            "embedding_only",
            "--json",
        ],
    );
    assert!(
        search_output.status.success(),
        "cli json embedding_only citation-shape search should succeed: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );

    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(search_json["results"][0]["record"]["scope"], "project");
    assert_eq!(search_json["results"][0]["record"]["truth_layer"], "t2");
    assert_eq!(search_json["results"][0]["record"]["record_type"], "decision");
    assert_eq!(
        search_json["results"][0]["citation"]["source_uri"],
        "memo://project/cli-json-embedding-only-citation-shape"
    );
    assert_eq!(
        search_json["results"][0]["citation"]["recorded_at"],
        "2026-04-17T10:19:00Z"
    );
    assert_eq!(search_json["results"][0]["citation"]["anchor"]["chunk_index"], 0);
    assert_eq!(search_json["results"][0]["citation"]["anchor"]["chunk_count"], 1);
}

#[test]
fn cli_search_json_preserves_record_and_citation_shape_for_hybrid_ready_path() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("cli-json-hybrid-citation-shape");
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
            source_uri: "memo://project/cli-json-hybrid-citation-shape",
            source_label: "cli json hybrid citation memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:20:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let search_output = run_cli(
        &config_path,
        &["search", "retrieval fusion", "--mode", "hybrid", "--json"],
    );
    assert!(
        search_output.status.success(),
        "cli json hybrid citation-shape search should succeed: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );

    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(search_json["results"][0]["record"]["scope"], "project");
    assert_eq!(search_json["results"][0]["record"]["truth_layer"], "t2");
    assert_eq!(search_json["results"][0]["record"]["record_type"], "decision");
    assert_eq!(
        search_json["results"][0]["citation"]["source_uri"],
        "memo://project/cli-json-hybrid-citation-shape"
    );
    assert_eq!(
        search_json["results"][0]["citation"]["recorded_at"],
        "2026-04-17T10:20:00Z"
    );
    assert_eq!(search_json["results"][0]["citation"]["anchor"]["chunk_index"], 0);
    assert_eq!(search_json["results"][0]["citation"]["anchor"]["chunk_count"], 1);
}

#[test]
fn cli_search_text_preserves_source_metadata_for_embedding_only_ready_path() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("cli-text-embedding-only-source-shape");
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
            source_uri: "memo://project/cli-text-embedding-only-source-shape",
            source_label: "cli text embedding_only source memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:27:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(
        &config_path,
        &["search", "retrieval fusion", "--mode", "embedding_only"],
    );
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text embedding_only source-shape search should succeed: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(text.contains("channel: embedding_only"));
    assert!(text.contains("kind=document"));
    assert!(text.contains("label=cli text embedding_only source memo"));
}

#[test]
fn cli_search_text_preserves_source_metadata_for_hybrid_ready_path() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("cli-text-hybrid-source-shape");
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
            source_uri: "memo://project/cli-text-hybrid-source-shape",
            source_label: "cli text hybrid source memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:28:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(&config_path, &["search", "retrieval fusion", "--mode", "hybrid"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text hybrid source-shape search should succeed: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(text.contains("channel: hybrid"));
    assert!(text.contains("kind=document"));
    assert!(text.contains("label=cli text hybrid source memo"));
}

#[test]
fn cli_search_text_embedding_only_applies_temporal_filters_before_top_k_when_ready() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("cli-text-embedding-only-temporal-top-k");
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
            source_uri: "memo://project/cli-text-embedding-only-current-temporal",
            source_label: "cli text embedding_only current temporal memo",
            content: "baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:15:30Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/cli-text-embedding-only-stale-temporal",
            source_label: "cli text embedding_only stale temporal memo",
            content: "baseline baseline keeps stale review around",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-01T10:15:30Z",
            valid_from: Some("2026-03-01T00:00:00Z"),
            valid_to: Some("2026-04-05T00:00:00Z"),
        },
    );

    let search_output = run_cli(
        &config_path,
        &[
            "search",
            "baseline",
            "--mode",
            "embedding_only",
            "--valid-at",
            "2026-04-17T12:00:00Z",
            "--from",
            "2026-04-10T00:00:00Z",
            "--to",
            "2026-04-18T00:00:00Z",
            "--top-k",
            "1",
        ],
    );
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text embedding_only temporal/top-k search should succeed: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(text.contains("channel: embedding_only"));
    assert!(text.contains("valid_at=2026-04-17T12:00:00Z"));
    assert!(text.contains("memo://project/cli-text-embedding-only-current-temporal"));
    assert!(
        !text.contains("memo://project/cli-text-embedding-only-stale-temporal"),
        "stale temporal result should be filtered out before text top-k rendering: {text}"
    );
}

#[test]
fn cli_search_text_hybrid_applies_temporal_filters_before_top_k_when_ready() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("cli-text-hybrid-temporal-top-k");
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
            source_uri: "memo://project/cli-text-hybrid-current-temporal",
            source_label: "cli text hybrid current temporal memo",
            content: "baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:16:30Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/cli-text-hybrid-stale-temporal",
            source_label: "cli text hybrid stale temporal memo",
            content: "baseline baseline keeps stale review around",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-01T10:16:30Z",
            valid_from: Some("2026-03-01T00:00:00Z"),
            valid_to: Some("2026-04-05T00:00:00Z"),
        },
    );

    let search_output = run_cli(
        &config_path,
        &[
            "search",
            "baseline",
            "--mode",
            "hybrid",
            "--valid-at",
            "2026-04-17T12:00:00Z",
            "--from",
            "2026-04-10T00:00:00Z",
            "--to",
            "2026-04-18T00:00:00Z",
            "--top-k",
            "1",
        ],
    );
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text hybrid temporal/top-k search should succeed: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(text.contains("channel: hybrid"));
    assert!(text.contains("valid_at=2026-04-17T12:00:00Z"));
    assert!(text.contains("memo://project/cli-text-hybrid-current-temporal"));
    assert!(
        !text.contains("memo://project/cli-text-hybrid-stale-temporal"),
        "stale temporal result should be filtered out before text top-k rendering: {text}"
    );
}

#[test]
fn cli_search_json_uses_configured_embedding_only_mode_when_second_channel_is_ready() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("configured-embedding-only-json-ready");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "embedding_only",
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
            source_uri: "memo://project/configured-embedding-only-json-ready",
            source_label: "configured embedding_only json ready memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:11:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(&config_path, &["search", "retrieval fusion", "--json"]);
    assert!(
        search_output.status.success(),
        "cli json search should honor configured embedding_only mode when ready: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );

    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(
        search_json["results"][0]["trace"]["channel_contribution"],
        "embedding_only"
    );
    assert_eq!(
        search_json["results"][0]["trace"]["query_strategies"],
        serde_json::json!(["Embedding"]),
        "configured embedding_only json search should only expose the embedding strategy when the second channel is ready"
    );
}

#[test]
fn cli_search_json_exposes_dsl_sidecar_for_configured_embedding_only_ready_path() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("configured-embedding-only-json-ready-dsl");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "embedding_only",
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
            source_uri: "memo://project/configured-embedding-only-json-ready-dsl",
            source_label: "configured embedding_only json ready dsl memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:11:30Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(&config_path, &["search", "retrieval fusion", "--json"]);
    assert!(
        search_output.status.success(),
        "cli json search should expose dsl for configured embedding_only ready mode: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );

    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(
        search_json["results"][0]["trace"]["channel_contribution"],
        "embedding_only"
    );
    assert_eq!(search_json["results"][0]["dsl"]["domain"], "project");
    assert_eq!(search_json["results"][0]["dsl"]["kind"], "decision");
    assert_eq!(
        search_json["results"][0]["dsl"]["source_ref"],
        "memo://project/configured-embedding-only-json-ready-dsl"
    );
}

#[test]
fn cli_search_json_uses_configured_hybrid_mode_when_second_channel_is_ready() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("configured-hybrid-json-ready");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "hybrid",
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
            source_uri: "memo://project/configured-hybrid-json-ready",
            source_label: "configured hybrid json ready memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:12:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(&config_path, &["search", "retrieval fusion", "--json"]);
    assert!(
        search_output.status.success(),
        "cli json search should honor configured hybrid mode when ready: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );

    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(
        search_json["results"][0]["trace"]["channel_contribution"],
        "hybrid"
    );
    assert_eq!(
        search_json["results"][0]["trace"]["query_strategies"],
        serde_json::json!(["Jieba", "Simple", "Structured", "Embedding"]),
        "configured hybrid json search should preserve the full ready-path strategy ordering"
    );
}

#[test]
fn cli_search_json_exposes_dsl_sidecar_for_configured_hybrid_ready_path() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("configured-hybrid-json-ready-dsl");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "hybrid",
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
            source_uri: "memo://project/configured-hybrid-json-ready-dsl",
            source_label: "configured hybrid json ready dsl memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:12:30Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(&config_path, &["search", "retrieval fusion", "--json"]);
    assert!(
        search_output.status.success(),
        "cli json search should expose dsl for configured hybrid ready mode: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );

    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(search_json["results"][0]["trace"]["channel_contribution"], "hybrid");
    assert_eq!(search_json["results"][0]["dsl"]["domain"], "project");
    assert_eq!(search_json["results"][0]["dsl"]["kind"], "decision");
    assert_eq!(
        search_json["results"][0]["dsl"]["source_ref"],
        "memo://project/configured-hybrid-json-ready-dsl"
    );
}

#[test]
fn cli_search_json_preserves_record_and_citation_shape_for_configured_embedding_only_ready_path() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("configured-embedding-only-json-citation-shape");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "embedding_only",
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
            source_uri: "memo://project/configured-embedding-only-json-citation-shape",
            source_label: "configured embedding_only json citation memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:21:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let search_output = run_cli(&config_path, &["search", "retrieval fusion", "--json"]);
    assert!(
        search_output.status.success(),
        "configured embedding_only json citation-shape search should succeed: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );

    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(search_json["results"][0]["record"]["scope"], "project");
    assert_eq!(search_json["results"][0]["record"]["truth_layer"], "t2");
    assert_eq!(search_json["results"][0]["record"]["record_type"], "decision");
    assert_eq!(
        search_json["results"][0]["citation"]["source_uri"],
        "memo://project/configured-embedding-only-json-citation-shape"
    );
    assert_eq!(
        search_json["results"][0]["citation"]["recorded_at"],
        "2026-04-17T10:21:00Z"
    );
    assert_eq!(search_json["results"][0]["citation"]["anchor"]["chunk_index"], 0);
    assert_eq!(search_json["results"][0]["citation"]["anchor"]["chunk_count"], 1);
}

#[test]
fn cli_search_json_preserves_record_and_citation_shape_for_configured_hybrid_ready_path() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("configured-hybrid-json-citation-shape");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "hybrid",
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
            source_uri: "memo://project/configured-hybrid-json-citation-shape",
            source_label: "configured hybrid json citation memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:22:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let search_output = run_cli(&config_path, &["search", "retrieval fusion", "--json"]);
    assert!(
        search_output.status.success(),
        "configured hybrid json citation-shape search should succeed: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );

    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(search_json["results"][0]["record"]["scope"], "project");
    assert_eq!(search_json["results"][0]["record"]["truth_layer"], "t2");
    assert_eq!(search_json["results"][0]["record"]["record_type"], "decision");
    assert_eq!(
        search_json["results"][0]["citation"]["source_uri"],
        "memo://project/configured-hybrid-json-citation-shape"
    );
    assert_eq!(
        search_json["results"][0]["citation"]["recorded_at"],
        "2026-04-17T10:22:00Z"
    );
    assert_eq!(search_json["results"][0]["citation"]["anchor"]["chunk_index"], 0);
    assert_eq!(search_json["results"][0]["citation"]["anchor"]["chunk_count"], 1);
}

#[test]
fn cli_search_json_preserves_source_metadata_for_configured_embedding_only_ready_path() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("configured-embedding-only-json-source-shape");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "embedding_only",
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
            source_uri: "memo://project/configured-embedding-only-json-source-shape",
            source_label: "configured embedding_only json source memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:23:30Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(&config_path, &["search", "retrieval fusion", "--json"]);
    assert!(
        search_output.status.success(),
        "configured embedding_only json source-shape search should succeed: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );

    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(search_json["results"][0]["record"]["source"]["kind"], "document");
    assert_eq!(
        search_json["results"][0]["record"]["source"]["label"],
        "configured embedding_only json source memo"
    );
    assert_eq!(search_json["results"][0]["citation"]["source_kind"], "document");
    assert_eq!(
        search_json["results"][0]["citation"]["source_label"],
        "configured embedding_only json source memo"
    );
}

#[test]
fn cli_search_json_preserves_source_metadata_for_configured_hybrid_ready_path() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("configured-hybrid-json-source-shape");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "hybrid",
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
            source_uri: "memo://project/configured-hybrid-json-source-shape",
            source_label: "configured hybrid json source memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:24:30Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(&config_path, &["search", "retrieval fusion", "--json"]);
    assert!(
        search_output.status.success(),
        "configured hybrid json source-shape search should succeed: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );

    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(search_json["results"][0]["record"]["source"]["kind"], "document");
    assert_eq!(
        search_json["results"][0]["record"]["source"]["label"],
        "configured hybrid json source memo"
    );
    assert_eq!(search_json["results"][0]["citation"]["source_kind"], "document");
    assert_eq!(
        search_json["results"][0]["citation"]["source_label"],
        "configured hybrid json source memo"
    );
}

#[test]
fn cli_search_text_preserves_record_and_citation_shape_for_configured_embedding_only_ready_path() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("configured-embedding-only-text-citation-shape");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "embedding_only",
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
            source_uri: "memo://project/configured-embedding-only-text-citation-shape",
            source_label: "configured embedding_only text citation memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:23:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let search_output = run_cli(&config_path, &["search", "retrieval fusion"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "configured embedding_only text citation-shape search should succeed: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(text.contains("channel: embedding_only"));
    assert!(text.contains("memo://project/configured-embedding-only-text-citation-shape"));
    assert!(text.contains("citation: chunk 1/1 recorded_at=2026-04-17T10:23:00Z"));
    assert!(text.contains("valid_from=2026-04-10T00:00:00Z"));
    assert!(text.contains("valid_to=2026-04-20T00:00:00Z"));
}

#[test]
fn cli_search_text_preserves_record_and_citation_shape_for_configured_hybrid_ready_path() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("configured-hybrid-text-citation-shape");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "hybrid",
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
            source_uri: "memo://project/configured-hybrid-text-citation-shape",
            source_label: "configured hybrid text citation memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:24:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let search_output = run_cli(&config_path, &["search", "retrieval fusion"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "configured hybrid text citation-shape search should succeed: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(text.contains("channel: hybrid"));
    assert!(text.contains("memo://project/configured-hybrid-text-citation-shape"));
    assert!(text.contains("citation: chunk 1/1 recorded_at=2026-04-17T10:24:00Z"));
    assert!(text.contains("valid_from=2026-04-10T00:00:00Z"));
    assert!(text.contains("valid_to=2026-04-20T00:00:00Z"));
}

#[test]
fn cli_search_text_preserves_source_metadata_for_configured_embedding_only_ready_path() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("configured-embedding-only-text-source-shape");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "embedding_only",
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
            source_uri: "memo://project/configured-embedding-only-text-source-shape",
            source_label: "configured embedding_only text source memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:25:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(&config_path, &["search", "retrieval fusion"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "configured embedding_only text source-shape search should succeed: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(text.contains("channel: embedding_only"));
    assert!(text.contains("kind=document"));
    assert!(text.contains("label=configured embedding_only text source memo"));
}

#[test]
fn cli_search_text_preserves_source_metadata_for_configured_hybrid_ready_path() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("configured-hybrid-text-source-shape");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "hybrid",
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
            source_uri: "memo://project/configured-hybrid-text-source-shape",
            source_label: "configured hybrid text source memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:26:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(&config_path, &["search", "retrieval fusion"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "configured hybrid text source-shape search should succeed: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(text.contains("channel: hybrid"));
    assert!(text.contains("kind=document"));
    assert!(text.contains("label=configured hybrid text source memo"));
}

#[test]
fn cli_search_text_configured_embedding_only_applies_temporal_filters_before_top_k() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("configured-embedding-only-text-temporal-top-k");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "embedding_only",
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
            source_uri: "memo://project/configured-embedding-only-current-temporal",
            source_label: "configured embedding_only current temporal text memo",
            content: "baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:17:30Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/configured-embedding-only-stale-temporal",
            source_label: "configured embedding_only stale temporal text memo",
            content: "baseline baseline keeps stale review around",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-01T10:17:30Z",
            valid_from: Some("2026-03-01T00:00:00Z"),
            valid_to: Some("2026-04-05T00:00:00Z"),
        },
    );

    let search_output = run_cli(
        &config_path,
        &[
            "search",
            "baseline",
            "--valid-at",
            "2026-04-17T12:00:00Z",
            "--from",
            "2026-04-10T00:00:00Z",
            "--to",
            "2026-04-18T00:00:00Z",
            "--top-k",
            "1",
        ],
    );
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "configured embedding_only text temporal/top-k search should succeed: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(text.contains("channel: embedding_only"));
    assert!(text.contains("valid_at=2026-04-17T12:00:00Z"));
    assert!(text.contains("memo://project/configured-embedding-only-current-temporal"));
    assert!(
        !text.contains("memo://project/configured-embedding-only-stale-temporal"),
        "stale temporal result should be filtered out before configured text top-k rendering: {text}"
    );
}

#[test]
fn cli_search_text_configured_hybrid_applies_temporal_filters_before_top_k() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("configured-hybrid-text-temporal-top-k");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "hybrid",
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
            source_uri: "memo://project/configured-hybrid-current-temporal",
            source_label: "configured hybrid current temporal text memo",
            content: "baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:18:30Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/configured-hybrid-stale-temporal",
            source_label: "configured hybrid stale temporal text memo",
            content: "baseline baseline keeps stale review around",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-01T10:18:30Z",
            valid_from: Some("2026-03-01T00:00:00Z"),
            valid_to: Some("2026-04-05T00:00:00Z"),
        },
    );

    let search_output = run_cli(
        &config_path,
        &[
            "search",
            "baseline",
            "--valid-at",
            "2026-04-17T12:00:00Z",
            "--from",
            "2026-04-10T00:00:00Z",
            "--to",
            "2026-04-18T00:00:00Z",
            "--top-k",
            "1",
        ],
    );
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "configured hybrid text temporal/top-k search should succeed: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(text.contains("channel: hybrid"));
    assert!(text.contains("valid_at=2026-04-17T12:00:00Z"));
    assert!(text.contains("memo://project/configured-hybrid-current-temporal"));
    assert!(
        !text.contains("memo://project/configured-hybrid-stale-temporal"),
        "stale temporal result should be filtered out before configured text top-k rendering: {text}"
    );
}

#[test]
fn cli_search_text_reports_embedding_only_channel_and_strategy_when_ready() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("mode-selection-text-embedding-only");
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
            source_uri: "memo://project/mode-selection-text-embedding-only",
            source_label: "mode selection text embedding only memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:05:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(
        &config_path,
        &["search", "retrieval fusion", "--mode", "embedding_only"],
    );
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for embedding_only mode-selection contract: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("channel: embedding_only"),
        "text output should expose embedding_only channel when the second channel is ready: {text}"
    );
    assert!(
        text.contains("strategies=embedding"),
        "text output should expose the embedding-only strategy summary: {text}"
    );
}

#[test]
fn cli_search_text_reports_hybrid_channel_and_embedding_strategy_when_ready() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("mode-selection-text-hybrid");
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
            source_uri: "memo://project/mode-selection-text-hybrid",
            source_label: "mode selection text hybrid memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:06:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(&config_path, &["search", "retrieval fusion", "--mode", "hybrid"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for hybrid mode-selection contract: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("channel: hybrid"),
        "text output should expose hybrid channel when the second channel is ready: {text}"
    );
    assert!(
        text.contains("embedding"),
        "text output should expose embedding participation in the hybrid strategy summary: {text}"
    );
}

#[test]
fn cli_search_text_renders_dsl_summary_for_embedding_only_ready_path() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("mode-selection-text-embedding-only-dsl");
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
            source_uri: "memo://project/mode-selection-text-embedding-only-dsl",
            source_label: "mode selection text embedding only dsl memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:05:30Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(
        &config_path,
        &["search", "retrieval fusion", "--mode", "embedding_only"],
    );
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for embedding_only ready-path dsl: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("channel: embedding_only"),
        "text output should still expose the embedding_only channel: {text}"
    );
    assert!(
        text.contains("dsl:"),
        "embedding_only ready-path text output should still render the DSL summary: {text}"
    );
    assert!(
        text.contains("SRC: memo://project/mode-selection-text-embedding-only-dsl"),
        "embedding_only ready-path text output should preserve the DSL source reference: {text}"
    );
}

#[test]
fn cli_search_text_renders_dsl_summary_for_hybrid_ready_path() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("mode-selection-text-hybrid-dsl");
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
            source_uri: "memo://project/mode-selection-text-hybrid-dsl",
            source_label: "mode selection text hybrid dsl memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:06:30Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(&config_path, &["search", "retrieval fusion", "--mode", "hybrid"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for hybrid ready-path dsl: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("channel: hybrid"),
        "text output should still expose the hybrid channel: {text}"
    );
    assert!(
        text.contains("dsl:"),
        "hybrid ready-path text output should still render the DSL summary: {text}"
    );
    assert!(
        text.contains("SRC: memo://project/mode-selection-text-hybrid-dsl"),
        "hybrid ready-path text output should preserve the DSL source reference: {text}"
    );
}

#[test]
fn cli_search_text_renders_dsl_summary_for_configured_embedding_only_ready_path() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("configured-embedding-only-text-ready-dsl");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "embedding_only",
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
            source_uri: "memo://project/configured-embedding-only-text-ready-dsl",
            source_label: "configured embedding_only text ready dsl memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:11:40Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(&config_path, &["search", "retrieval fusion"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should expose dsl for configured embedding_only ready mode: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(text.contains("channel: embedding_only"));
    assert!(text.contains("dsl:"));
    assert!(text.contains(
        "SRC: memo://project/configured-embedding-only-text-ready-dsl"
    ));
}

#[test]
fn cli_search_text_renders_dsl_summary_for_configured_hybrid_ready_path() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("configured-hybrid-text-ready-dsl");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "hybrid",
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
            source_uri: "memo://project/configured-hybrid-text-ready-dsl",
            source_label: "configured hybrid text ready dsl memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:12:40Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(&config_path, &["search", "retrieval fusion"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should expose dsl for configured hybrid ready mode: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(text.contains("channel: hybrid"));
    assert!(text.contains("dsl:"));
    assert!(text.contains("SRC: memo://project/configured-hybrid-text-ready-dsl"));
}

#[test]
fn library_search_with_runtime_config_embedding_only_applies_taxonomy_filters_before_top_k() {
    let path = fresh_db_path("runtime-config-embedding-only-taxonomy-top-k");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-embedding-only-config-top-k",
            source_label: "runtime config embedding_only config top k memo",
            content: "baseline baseline keeps toml setting review stable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:41:00Z",
            valid_from: None,
            valid_to: None,
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-embedding-only-retrieval-top-k",
            source_label: "runtime config embedding_only retrieval top k memo",
            content: "baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:40:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::EmbeddingOnly,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(db.conn(), &config, None)
        .search(
            &SearchRequest::new("baseline")
                .with_limit(1)
                .with_filters(SearchFilters {
                    topic: Some("retrieval".to_string()),
                    ..Default::default()
                }),
        )
        .expect("configured embedding_only search should apply taxonomy filters before top-k");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/runtime-config-embedding-only-retrieval-top-k"
    );
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::EmbeddingOnly
    );
    assert_eq!(response.applied_filters.topic.as_deref(), Some("retrieval"));
}

#[test]
fn library_search_with_runtime_config_hybrid_applies_taxonomy_filters_before_top_k() {
    let path = fresh_db_path("runtime-config-hybrid-taxonomy-top-k");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-hybrid-config-top-k",
            source_label: "runtime config hybrid config top k memo",
            content: "baseline baseline keeps toml setting review stable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:43:00Z",
            valid_from: None,
            valid_to: None,
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-hybrid-retrieval-top-k",
            source_label: "runtime config hybrid retrieval top k memo",
            content: "baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:42:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::Hybrid,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(db.conn(), &config, None)
        .search(
            &SearchRequest::new("baseline")
                .with_limit(1)
                .with_filters(SearchFilters {
                    topic: Some("retrieval".to_string()),
                    ..Default::default()
                }),
        )
        .expect("configured hybrid search should apply taxonomy filters before top-k");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/runtime-config-hybrid-retrieval-top-k"
    );
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::Hybrid
    );
    assert_eq!(response.applied_filters.topic.as_deref(), Some("retrieval"));
}

#[test]
fn cli_search_text_reports_exact_embedding_only_strategy_summary_when_ready() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("mode-selection-text-embedding-only-exact");
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
            source_uri: "memo://project/mode-selection-text-embedding-only-exact",
            source_label: "mode selection text embedding only exact memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:07:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(
        &config_path,
        &["search", "retrieval fusion", "--mode", "embedding_only"],
    );
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for exact embedding_only strategy contract: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("channel: embedding_only strategies=embedding"),
        "text output should expose the exact embedding_only strategy summary when the second channel is ready: {text}"
    );
}

#[test]
fn cli_search_text_reports_exact_hybrid_strategy_summary_when_ready() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("mode-selection-text-hybrid-exact");
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
            source_uri: "memo://project/mode-selection-text-hybrid-exact",
            source_label: "mode selection text hybrid exact memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:08:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(&config_path, &["search", "retrieval fusion", "--mode", "hybrid"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for exact hybrid strategy contract: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("channel: hybrid strategies=jieba,simple,structured,embedding"),
        "text output should expose the exact hybrid strategy summary when the second channel is ready: {text}"
    );
}

#[test]
fn cli_search_json_keeps_lexical_only_when_ready_embedding_channel_is_configured() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("cli-json-default-lexical-ready-embedding");
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
            source_uri: "memo://project/cli-json-default-lexical-ready-embedding",
            source_label: "cli json default lexical ready embedding memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T12:30:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(&config_path, &["search", "retrieval fusion", "--json"]);
    assert!(
        search_output.status.success(),
        "cli json search should succeed for lexical_only ready-embedding contract: stdout={} stderr={}",
        stdout(&search_output),
        stderr(&search_output)
    );

    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    assert_eq!(
        search_json["results"][0]["trace"]["channel_contribution"],
        "lexical_only"
    );
    let strategies = search_json["results"][0]["trace"]["query_strategies"]
        .as_array()
        .expect("query strategies should be an array");
    assert!(
        !strategies.iter().any(|value| value == "Embedding"),
        "default lexical_only config should not surface embedding strategies even when the embedding channel is ready: {}",
        stdout(&search_output)
    );
}

#[test]
fn cli_search_json_reports_exact_lexical_only_strategy_order_when_ready_embedding_is_configured() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("cli-json-default-lexical-ready-embedding-exact");
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
            source_uri: "memo://project/cli-json-default-lexical-ready-embedding-exact",
            source_label: "cli json default lexical ready embedding exact memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:09:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(&config_path, &["search", "retrieval fusion", "--json"]);
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    let strategies = search_json["results"][0]["trace"]["query_strategies"]
        .as_array()
        .expect("query strategies should be an array");
    assert_eq!(
        strategies,
        &vec![
            Value::String("Jieba".to_string()),
            Value::String("Simple".to_string()),
            Value::String("Structured".to_string()),
        ],
        "default lexical_only config should preserve the exact lexical-only strategy ordering even when the embedding channel is ready"
    );
}

#[test]
fn cli_search_json_reports_exact_hybrid_strategy_order_when_ready() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("cli-json-hybrid-exact-strategies");
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
            source_uri: "memo://project/cli-json-hybrid-exact-strategies",
            source_label: "cli json hybrid exact strategies memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:10:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(
        &config_path,
        &["search", "retrieval fusion", "--mode", "hybrid", "--json"],
    );
    let search_json: Value =
        serde_json::from_str(&stdout(&search_output)).expect("search should emit json");
    let strategies = search_json["results"][0]["trace"]["query_strategies"]
        .as_array()
        .expect("query strategies should be an array");
    assert_eq!(
        strategies,
        &vec![
            Value::String("Jieba".to_string()),
            Value::String("Simple".to_string()),
            Value::String("Structured".to_string()),
            Value::String("Embedding".to_string()),
        ],
        "hybrid json output should preserve the exact ready-path strategy ordering"
    );
}

#[test]
fn cli_search_text_keeps_lexical_only_when_ready_embedding_channel_is_configured() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("cli-text-default-lexical-ready-embedding");
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
            source_uri: "memo://project/cli-text-default-lexical-ready-embedding",
            source_label: "cli text default lexical ready embedding memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T12:35:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(&config_path, &["search", "retrieval fusion"]);
    let text = stdout(&search_output);
    assert!(
        search_output.status.success(),
        "cli text search should succeed for lexical_only ready-embedding contract: stdout={text} stderr={}",
        stderr(&search_output)
    );
    assert!(
        text.contains("channel: lexical_only"),
        "default lexical_only config should preserve lexical_only channel output even when the embedding channel is ready: {text}"
    );
    assert!(
        !text.contains("strategies=embedding"),
        "default lexical_only config should not expose embedding strategies in text output when the second channel is merely ready: {text}"
    );
}

#[test]
fn library_search_with_runtime_config_embedding_only_returns_no_results_when_embedding_is_disabled()
{
    let path = fresh_db_path("runtime-config-embedding-disabled");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-embedding-disabled",
            source_label: "runtime config embedding disabled memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T10:00:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::LexicalOnly,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Disabled,
            model: None,
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::None,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(
        db.conn(),
        &config,
        Some(RetrievalMode::EmbeddingOnly),
    )
    .search(&SearchRequest::new("baseline"))
    .expect("embedding_only library search should succeed even when the embedding channel is disabled");

    assert!(
        response.results.is_empty(),
        "embedding_only service-level search should return no results when the embedding channel is disabled instead of falling back to lexical recall"
    );
}

#[test]
fn library_search_with_runtime_config_hybrid_falls_back_to_lexical_when_embedding_is_disabled() {
    let path = fresh_db_path("runtime-config-hybrid-disabled");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-hybrid-disabled",
            source_label: "runtime config hybrid disabled memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T10:05:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::LexicalOnly,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Disabled,
            model: None,
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::None,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(db.conn(), &config, Some(RetrievalMode::Hybrid))
        .search(&SearchRequest::new("baseline"))
        .expect("hybrid library search should succeed when the embedding channel is disabled");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/runtime-config-hybrid-disabled"
    );
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::LexicalOnly,
        "hybrid service-level search should degrade to lexical-only channel contribution when the embedding channel is unavailable"
    );
}

#[test]
fn library_search_with_runtime_config_embedding_only_returns_no_results_when_embedding_model_is_missing()
{
    let path = fresh_db_path("runtime-config-embedding-missing-model");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-embedding-missing-model",
            source_label: "runtime config embedding missing model memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T10:20:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::LexicalOnly,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: None,
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(
        db.conn(),
        &config,
        Some(RetrievalMode::EmbeddingOnly),
    )
    .search(&SearchRequest::new("baseline"))
    .expect("embedding_only library search should succeed even when the embedding model is missing");

    assert!(
        response.results.is_empty(),
        "embedding_only service-level search should return no results when the embedding model is missing instead of falling back to lexical recall"
    );
}

#[test]
fn library_search_with_runtime_config_hybrid_falls_back_to_lexical_when_embedding_model_is_missing()
{
    let path = fresh_db_path("runtime-config-hybrid-missing-model");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-hybrid-missing-model",
            source_label: "runtime config hybrid missing model memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T10:25:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::LexicalOnly,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: None,
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(db.conn(), &config, Some(RetrievalMode::Hybrid))
        .search(&SearchRequest::new("baseline"))
        .expect("hybrid library search should succeed when the embedding model is missing");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/runtime-config-hybrid-missing-model"
    );
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::LexicalOnly,
        "hybrid service-level search should degrade to lexical-only channel contribution when the embedding model is missing"
    );
}

#[test]
fn library_search_with_runtime_config_embedding_only_returns_no_results_when_vector_backend_is_none()
{
    let path = fresh_db_path("runtime-config-embedding-missing-vector");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-embedding-missing-vector",
            source_label: "runtime config embedding missing vector memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T10:40:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::LexicalOnly,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::None,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(
        db.conn(),
        &config,
        Some(RetrievalMode::EmbeddingOnly),
    )
    .search(&SearchRequest::new("baseline"))
    .expect("embedding_only library search should succeed even when the vector backend is missing");

    assert!(
        response.results.is_empty(),
        "embedding_only service-level search should return no results when the vector backend is missing instead of falling back to lexical recall"
    );
}

#[test]
fn library_search_with_runtime_config_hybrid_falls_back_to_lexical_when_vector_backend_is_none() {
    let path = fresh_db_path("runtime-config-hybrid-missing-vector");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-hybrid-missing-vector",
            source_label: "runtime config hybrid missing vector memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T10:45:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::LexicalOnly,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::None,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(db.conn(), &config, Some(RetrievalMode::Hybrid))
        .search(&SearchRequest::new("baseline"))
        .expect("hybrid library search should succeed when the vector backend is missing");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/runtime-config-hybrid-missing-vector"
    );
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::LexicalOnly,
        "hybrid service-level search should degrade to lexical-only channel contribution when the vector backend is missing"
    );
}

#[test]
fn library_search_with_runtime_config_uses_configured_embedding_only_mode_when_model_is_missing() {
    let path = fresh_db_path("runtime-config-configured-embedding-only-missing-model");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-configured-embedding-only-missing-model",
            source_label: "configured embedding_only missing model memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T11:20:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::EmbeddingOnly,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: None,
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(db.conn(), &config, None)
        .search(&SearchRequest::new("baseline"))
        .expect("embedding_only library search should honor the configured mode when mode_override is omitted");

    assert!(
        response.results.is_empty(),
        "configured embedding_only mode should return no results when the embedding model is missing instead of falling back to lexical recall"
    );
}

#[test]
fn library_search_with_runtime_config_uses_configured_hybrid_mode_when_model_is_missing() {
    let path = fresh_db_path("runtime-config-configured-hybrid-missing-model");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-configured-hybrid-missing-model",
            source_label: "configured hybrid missing model memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T11:25:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::Hybrid,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: None,
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(db.conn(), &config, None)
        .search(&SearchRequest::new("baseline"))
        .expect("hybrid library search should honor the configured mode when mode_override is omitted");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/runtime-config-configured-hybrid-missing-model"
    );
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::LexicalOnly,
        "configured hybrid mode should degrade to lexical-only channel contribution when the embedding model is missing"
    );
}

#[test]
fn library_search_with_runtime_config_uses_configured_embedding_only_mode_when_embedding_is_ready()
{
    let path = fresh_db_path("runtime-config-configured-embedding-only-ready");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-configured-embedding-only-ready",
            source_label: "configured embedding_only ready memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T11:27:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::EmbeddingOnly,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(db.conn(), &config, None)
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("embedding_only library search should honor the configured mode when the embedding channel is ready");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::EmbeddingOnly,
        "configured embedding_only mode should surface embedding-only contribution when the embedding channel is ready"
    );
    assert_eq!(
        response.results[0].trace.query_strategies,
        vec![agent_memos::search::QueryStrategy::Embedding],
        "configured embedding_only mode should only surface the embedding strategy when the embedding channel is ready"
    );
}

#[test]
fn library_search_preserves_record_and_citation_shape_for_embedding_only_ready_path() {
    let path = fresh_db_path("library-embedding-only-record-citation-shape");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-embedding-only-record-citation-shape",
            source_label: "library embedding only record citation memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:30:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::LexicalOnly,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(
        db.conn(),
        &config,
        Some(RetrievalMode::EmbeddingOnly),
    )
    .search(&SearchRequest::new("retrieval fusion"))
    .expect("embedding_only ready-path search should succeed");

    assert_eq!(response.results.len(), 1);
    let result = &response.results[0];
    assert_eq!(result.record.scope, Scope::Project);
    assert_eq!(result.record.truth_layer, TruthLayer::T2);
    assert_eq!(result.record.record_type, RecordType::Decision);
    assert_eq!(result.citation.record_id, result.record.id);
    assert_eq!(result.citation.source_uri, result.record.source.uri);
    assert_eq!(result.citation.recorded_at, "2026-04-18T13:30:00Z");
    assert_eq!(result.citation.validity.valid_from.as_deref(), Some("2026-04-10T00:00:00Z"));
    assert_eq!(result.citation.validity.valid_to.as_deref(), Some("2026-04-20T00:00:00Z"));
    assert_eq!(result.citation.anchor.chunk_index, 0);
    assert_eq!(result.citation.anchor.chunk_count, 1);
}

#[test]
fn library_search_preserves_dsl_sidecar_for_embedding_only_ready_path() {
    let path = fresh_db_path("library-embedding-only-dsl-sidecar");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-embedding-only-dsl-sidecar",
            source_label: "library embedding only dsl sidecar memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:31:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::LexicalOnly,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(
        db.conn(),
        &config,
        Some(RetrievalMode::EmbeddingOnly),
    )
    .search(&SearchRequest::new("retrieval fusion"))
    .expect("embedding_only ready-path search should preserve the dsl sidecar");

    assert_eq!(response.results.len(), 1);
    let dsl = response.results[0]
        .dsl
        .as_ref()
        .expect("embedding_only ready-path search should attach the structured dsl sidecar");
    assert_eq!(dsl.domain, "project");
    assert_eq!(dsl.kind, "decision");
    assert_eq!(dsl.source_ref, "memo://project/library-embedding-only-dsl-sidecar");
    assert!(!dsl.claim.is_empty());
}

#[test]
fn library_search_with_runtime_config_uses_unsuffixed_builtin_model_as_16_dimensions() {
    let path = fresh_db_path("runtime-config-unsuffixed-builtin-model");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-unsuffixed-builtin-model",
            source_label: "runtime config unsuffixed builtin model memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:10:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::EmbeddingOnly,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(db.conn(), &config, None)
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("unsuffixed builtin model should still work for embedding_only retrieval");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::EmbeddingOnly
    );
    assert_eq!(
        response.results[0].trace.query_strategies,
        vec![agent_memos::search::QueryStrategy::Embedding],
        "unsuffixed builtin model should fall back to the default 16-dimensional embedding path"
    );
}

#[test]
fn library_search_with_runtime_config_uses_unsuffixed_builtin_model_for_hybrid_ready_path() {
    let path = fresh_db_path("runtime-config-unsuffixed-builtin-model-hybrid");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-unsuffixed-builtin-model-hybrid",
            source_label: "runtime config unsuffixed builtin hybrid memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:20:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::Hybrid,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(db.conn(), &config, None)
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("unsuffixed builtin model should still work for configured hybrid retrieval");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::Hybrid
    );
    assert_eq!(
        response.results[0].trace.query_strategies,
        vec![
            agent_memos::search::QueryStrategy::Jieba,
            agent_memos::search::QueryStrategy::Simple,
            agent_memos::search::QueryStrategy::Structured,
            agent_memos::search::QueryStrategy::Embedding,
        ],
        "unsuffixed builtin model should fall back to the default 16-dimensional embedding path for configured hybrid retrieval"
    );
}

#[test]
fn library_search_with_runtime_config_mode_override_can_force_lexical_only_when_embedding_is_ready()
{
    let path = fresh_db_path("runtime-config-override-lexical-only");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-override-lexical-only",
            source_label: "runtime config override lexical_only memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T11:50:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::EmbeddingOnly,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(
        db.conn(),
        &config,
        Some(RetrievalMode::LexicalOnly),
    )
    .search(&SearchRequest::new("retrieval fusion"))
    .expect("mode_override=lexical_only should succeed when the embedding channel is ready");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::LexicalOnly,
        "mode_override=lexical_only should suppress embedding contribution even when config.retrieval.mode is embedding_only"
    );
    assert!(
        !response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Embedding),
        "mode_override=lexical_only should suppress embedding strategies even when the embedding channel is ready"
    );
}

#[test]
fn library_search_with_runtime_config_mode_override_can_force_hybrid_when_embedding_is_ready() {
    let path = fresh_db_path("runtime-config-override-hybrid");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-override-hybrid",
            source_label: "runtime config override hybrid memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T11:55:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::LexicalOnly,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(
        db.conn(),
        &config,
        Some(RetrievalMode::Hybrid),
    )
    .search(&SearchRequest::new("retrieval fusion"))
    .expect("mode_override=hybrid should succeed when the embedding channel is ready");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::Hybrid,
        "mode_override=hybrid should enable hybrid contribution even when config.retrieval.mode is lexical_only"
    );
    assert!(
        response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Embedding),
        "mode_override=hybrid should surface embedding strategies when the embedding channel is ready"
    );
}

#[test]
fn library_search_with_runtime_config_mode_override_can_force_embedding_only_when_embedding_is_ready()
{
    let path = fresh_db_path("runtime-config-override-embedding-only");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-override-embedding-only",
            source_label: "runtime config override embedding_only memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T12:40:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::LexicalOnly,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(
        db.conn(),
        &config,
        Some(RetrievalMode::EmbeddingOnly),
    )
    .search(&SearchRequest::new("retrieval fusion"))
    .expect("mode_override=embedding_only should succeed when the embedding channel is ready");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::EmbeddingOnly,
        "mode_override=embedding_only should enable embedding-only contribution when the embedding channel is ready"
    );
    assert_eq!(
        response.results[0].trace.query_strategies,
        vec![agent_memos::search::QueryStrategy::Embedding],
        "mode_override=embedding_only should only surface embedding strategies when the embedding channel is ready"
    );
}

#[test]
fn library_search_with_runtime_config_embedding_only_uses_truncated_raw_snippet_when_ready() {
    let path = fresh_db_path("runtime-config-embedding-only-snippet-truncation");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    let content = "embedding snippets should preserve the original content surface even when they are routed through the second channel without lexical fallback and this sentence intentionally exceeds ninety six characters";
    let expected_snippet = content.chars().take(96).collect::<String>();

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-embedding-only-snippet-truncation",
            source_label: "runtime config embedding only snippet truncation memo",
            content,
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:10:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::LexicalOnly,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(
        db.conn(),
        &config,
        Some(RetrievalMode::EmbeddingOnly),
    )
    .search(&SearchRequest::new("semantic second channel"))
    .expect("embedding_only search should succeed when the second channel is ready");

    assert_eq!(response.results.len(), 1);
    assert_eq!(response.results[0].snippet, expected_snippet);
}

#[test]
fn library_search_with_runtime_config_preserves_source_metadata_for_embedding_only_ready_path() {
    let path = fresh_db_path("runtime-config-embedding-only-source-metadata");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-embedding-only-source-metadata",
            source_label: "runtime config embedding_only source memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:12:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::EmbeddingOnly,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(db.conn(), &config, None)
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("configured embedding_only search should preserve source metadata when ready");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.kind,
        agent_memos::memory::record::SourceKind::Document
    );
    assert_eq!(
        response.results[0].record.source.label.as_deref(),
        Some("runtime config embedding_only source memo")
    );
    assert_eq!(
        response.results[0].citation.source_kind,
        agent_memos::memory::record::SourceKind::Document
    );
    assert_eq!(
        response.results[0].citation.source_label.as_deref(),
        Some("runtime config embedding_only source memo")
    );
}

#[test]
fn library_search_with_runtime_config_uses_configured_hybrid_mode_when_embedding_is_ready() {
    let path = fresh_db_path("runtime-config-configured-hybrid-ready-embedding");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-configured-hybrid-ready-embedding",
            source_label: "configured hybrid ready embedding memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T12:50:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::Hybrid,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(db.conn(), &config, None)
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("configured hybrid search should succeed when the embedding channel is ready");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::Hybrid,
        "configured hybrid mode should surface hybrid contribution when the embedding channel is ready"
    );
    assert!(
        response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Embedding),
        "configured hybrid mode should surface embedding strategies when the embedding channel is ready"
    );
}

#[test]
fn library_search_with_runtime_config_preserves_source_metadata_for_hybrid_ready_path() {
    let path = fresh_db_path("runtime-config-hybrid-source-metadata");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-hybrid-source-metadata",
            source_label: "runtime config hybrid source memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:13:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::Hybrid,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(db.conn(), &config, None)
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("configured hybrid search should preserve source metadata when ready");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.kind,
        agent_memos::memory::record::SourceKind::Document
    );
    assert_eq!(
        response.results[0].record.source.label.as_deref(),
        Some("runtime config hybrid source memo")
    );
    assert_eq!(
        response.results[0].citation.source_kind,
        agent_memos::memory::record::SourceKind::Document
    );
    assert_eq!(
        response.results[0].citation.source_label.as_deref(),
        Some("runtime config hybrid source memo")
    );
}

#[test]
fn library_search_preserves_record_and_citation_shape_for_hybrid_ready_path() {
    let path = fresh_db_path("library-hybrid-record-citation-shape");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-hybrid-record-citation-shape",
            source_label: "library hybrid record citation memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:35:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::Hybrid,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(db.conn(), &config, None)
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("hybrid ready-path search should succeed");

    assert_eq!(response.results.len(), 1);
    let result = &response.results[0];
    assert_eq!(result.record.scope, Scope::Project);
    assert_eq!(result.record.truth_layer, TruthLayer::T2);
    assert_eq!(result.record.record_type, RecordType::Decision);
    assert_eq!(result.citation.record_id, result.record.id);
    assert_eq!(result.citation.source_uri, result.record.source.uri);
    assert_eq!(result.citation.recorded_at, "2026-04-18T13:35:00Z");
    assert_eq!(result.citation.validity.valid_from.as_deref(), Some("2026-04-10T00:00:00Z"));
    assert_eq!(result.citation.validity.valid_to.as_deref(), Some("2026-04-20T00:00:00Z"));
    assert_eq!(result.citation.anchor.chunk_index, 0);
    assert_eq!(result.citation.anchor.chunk_count, 1);
}

#[test]
fn library_search_preserves_dsl_sidecar_for_hybrid_ready_path() {
    let path = fresh_db_path("library-hybrid-dsl-sidecar");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/library-hybrid-dsl-sidecar",
            source_label: "library hybrid dsl sidecar memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:36:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::Hybrid,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(db.conn(), &config, None)
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("hybrid ready-path search should preserve the dsl sidecar");

    assert_eq!(response.results.len(), 1);
    let dsl = response.results[0]
        .dsl
        .as_ref()
        .expect("hybrid ready-path search should attach the structured dsl sidecar");
    assert_eq!(dsl.domain, "project");
    assert_eq!(dsl.kind, "decision");
    assert_eq!(dsl.source_ref, "memo://project/library-hybrid-dsl-sidecar");
    assert!(!dsl.claim.is_empty());
}

#[test]
fn library_search_with_runtime_config_reports_exact_hybrid_strategies_when_embedding_is_ready() {
    let path = fresh_db_path("runtime-config-configured-hybrid-exact-strategies");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-configured-hybrid-exact-strategies",
            source_label: "configured hybrid exact strategies memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:00:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::Hybrid,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(db.conn(), &config, None)
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("configured hybrid search should succeed when the embedding channel is ready");

    assert_eq!(
        response.results[0].trace.query_strategies,
        vec![
            agent_memos::search::QueryStrategy::Jieba,
            agent_memos::search::QueryStrategy::Simple,
            agent_memos::search::QueryStrategy::Structured,
            agent_memos::search::QueryStrategy::Embedding,
        ],
        "configured hybrid mode should preserve the full ready-path strategy ordering"
    );
}

#[test]
fn library_search_with_runtime_config_embedding_only_returns_no_results_when_embedding_backend_is_reserved()
{
    let path = fresh_db_path("runtime-config-embedding-reserved");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-embedding-reserved",
            source_label: "runtime config embedding reserved memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T11:00:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::LexicalOnly,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Reserved,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(
        db.conn(),
        &config,
        Some(RetrievalMode::EmbeddingOnly),
    )
    .search(&SearchRequest::new("baseline"))
    .expect("embedding_only library search should succeed even when the embedding backend is reserved");

    assert!(
        response.results.is_empty(),
        "embedding_only service-level search should return no results when the embedding backend is reserved instead of falling back to lexical recall"
    );
}

#[test]
fn library_search_with_runtime_config_hybrid_falls_back_to_lexical_when_embedding_backend_is_reserved()
{
    let path = fresh_db_path("runtime-config-hybrid-reserved");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-hybrid-reserved",
            source_label: "runtime config hybrid reserved memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T11:05:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::LexicalOnly,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Reserved,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(db.conn(), &config, Some(RetrievalMode::Hybrid))
        .search(&SearchRequest::new("baseline"))
        .expect("hybrid library search should succeed when the embedding backend is reserved");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/runtime-config-hybrid-reserved"
    );
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::LexicalOnly,
        "hybrid service-level search should degrade to lexical-only channel contribution when the embedding backend is reserved"
    );
}

#[test]
fn library_search_with_runtime_config_uses_configured_lexical_only_mode_when_embedding_backend_is_reserved()
{
    let path = fresh_db_path("runtime-config-configured-lexical-only-reserved");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-configured-lexical-only-reserved",
            source_label: "configured lexical_only reserved memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T11:20:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::LexicalOnly,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Reserved,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(db.conn(), &config, None)
        .search(&SearchRequest::new("baseline"))
        .expect("lexical_only library search should honor the configured mode when the embedding backend is reserved");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/runtime-config-configured-lexical-only-reserved"
    );
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::LexicalOnly,
        "configured lexical_only mode should remain lexical-only when the embedding backend is reserved"
    );
}

#[test]
fn library_search_with_runtime_config_uses_configured_lexical_only_mode_when_vector_backend_is_none()
{
    let path = fresh_db_path("runtime-config-configured-lexical-only-no-vector");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-configured-lexical-only-no-vector",
            source_label: "configured lexical_only no vector memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T11:25:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::LexicalOnly,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::None,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(db.conn(), &config, None)
        .search(&SearchRequest::new("baseline"))
        .expect("lexical_only library search should honor the configured mode when the vector backend is missing");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/runtime-config-configured-lexical-only-no-vector"
    );
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::LexicalOnly,
        "configured lexical_only mode should remain lexical-only when the vector backend is missing"
    );
}

#[test]
fn library_search_with_variant_embedding_only_returns_no_results_when_embedding_sidecar_is_missing()
{
    let path = fresh_db_path("variant-embedding-missing-sidecar");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-embedding-missing-sidecar",
            source_label: "variant embedding missing sidecar memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T10:10:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let variant = RetrievalModeVariant {
        name: "embedding_only".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::EmbeddingOnly,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: None,
        vector: None,
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(&SearchRequest::new("baseline"))
        .expect("embedding_only variant search should succeed even when the embedding sidecar is missing");

    assert!(
        response.results.is_empty(),
        "embedding_only variant search should return no results when the embedding sidecar is unavailable instead of falling back to lexical recall"
    );
}

#[test]
fn library_search_with_variant_hybrid_falls_back_to_lexical_when_embedding_sidecar_is_missing() {
    let path = fresh_db_path("variant-hybrid-missing-sidecar");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-hybrid-missing-sidecar",
            source_label: "variant hybrid missing sidecar memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T10:15:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let variant = RetrievalModeVariant {
        name: "hybrid".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::Hybrid,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: None,
        vector: None,
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(&SearchRequest::new("baseline"))
        .expect("hybrid variant search should succeed when the embedding sidecar is missing");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/variant-hybrid-missing-sidecar"
    );
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::LexicalOnly,
        "hybrid variant search should degrade to lexical-only channel contribution when the embedding sidecar is unavailable"
    );
}

#[test]
fn library_search_with_variant_embedding_only_returns_no_results_when_embedding_model_is_missing()
{
    let path = fresh_db_path("variant-embedding-missing-model");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-embedding-missing-model",
            source_label: "variant embedding missing model memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T10:30:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let variant = RetrievalModeVariant {
        name: "embedding_only".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::EmbeddingOnly,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig::default()),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(&SearchRequest::new("baseline"))
        .expect("embedding_only variant search should succeed even when the embedding model is missing");

    assert!(
        response.results.is_empty(),
        "embedding_only variant search should return no results when the embedding model is missing instead of falling back to lexical recall"
    );
}

#[test]
fn library_search_with_variant_hybrid_falls_back_to_lexical_when_embedding_model_is_missing() {
    let path = fresh_db_path("variant-hybrid-missing-model");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-hybrid-missing-model",
            source_label: "variant hybrid missing model memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T10:35:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let variant = RetrievalModeVariant {
        name: "hybrid".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::Hybrid,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig::default()),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(&SearchRequest::new("baseline"))
        .expect("hybrid variant search should succeed when the embedding model is missing");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/variant-hybrid-missing-model"
    );
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::LexicalOnly,
        "hybrid variant search should degrade to lexical-only channel contribution when the embedding model is missing"
    );
}

#[test]
fn library_search_with_variant_embedding_only_returns_no_results_when_vector_backend_is_none() {
    let path = fresh_db_path("variant-embedding-missing-vector");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-embedding-missing-vector",
            source_label: "variant embedding missing vector memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T10:50:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let variant = RetrievalModeVariant {
        name: "embedding_only".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::EmbeddingOnly,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig {
            model: "builtin-16".to_string(),
            ..Default::default()
        }),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::None,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(&SearchRequest::new("baseline"))
        .expect("embedding_only variant search should succeed even when the vector backend is missing");

    assert!(
        response.results.is_empty(),
        "embedding_only variant search should return no results when the vector backend is missing instead of falling back to lexical recall"
    );
}

#[test]
fn library_search_with_variant_hybrid_falls_back_to_lexical_when_vector_backend_is_none() {
    let path = fresh_db_path("variant-hybrid-missing-vector");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-hybrid-missing-vector",
            source_label: "variant hybrid missing vector memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T10:55:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let variant = RetrievalModeVariant {
        name: "hybrid".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::Hybrid,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig {
            model: "builtin-16".to_string(),
            ..Default::default()
        }),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::None,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(&SearchRequest::new("baseline"))
        .expect("hybrid variant search should succeed when the vector backend is missing");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/variant-hybrid-missing-vector"
    );
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::LexicalOnly,
        "hybrid variant search should degrade to lexical-only channel contribution when the vector backend is missing"
    );
}

#[test]
fn library_search_with_variant_embedding_only_returns_no_results_when_embedding_backend_is_reserved()
{
    let path = fresh_db_path("variant-embedding-reserved");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-embedding-reserved",
            source_label: "variant embedding reserved memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T11:10:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let variant = RetrievalModeVariant {
        name: "embedding_only".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::EmbeddingOnly,
        embedding_backend: EmbeddingBackend::Reserved,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig {
            model: "builtin-16".to_string(),
            ..Default::default()
        }),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(&SearchRequest::new("baseline"))
        .expect("embedding_only variant search should succeed even when the embedding backend is reserved");

    assert!(
        response.results.is_empty(),
        "embedding_only variant search should return no results when the embedding backend is reserved instead of falling back to lexical recall"
    );
}

#[test]
fn library_search_with_variant_hybrid_falls_back_to_lexical_when_embedding_backend_is_reserved() {
    let path = fresh_db_path("variant-hybrid-reserved");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-hybrid-reserved",
            source_label: "variant hybrid reserved memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T11:15:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let variant = RetrievalModeVariant {
        name: "hybrid".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::Hybrid,
        embedding_backend: EmbeddingBackend::Reserved,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig {
            model: "builtin-16".to_string(),
            ..Default::default()
        }),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(&SearchRequest::new("baseline"))
        .expect("hybrid variant search should succeed when the embedding backend is reserved");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/variant-hybrid-reserved"
    );
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::LexicalOnly,
        "hybrid variant search should degrade to lexical-only channel contribution when the embedding backend is reserved"
    );
}

#[test]
fn library_search_with_variant_uses_lexical_only_mode_when_embedding_backend_is_reserved() {
    let path = fresh_db_path("variant-lexical-only-reserved");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-lexical-only-reserved",
            source_label: "variant lexical_only reserved memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T11:30:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let variant = RetrievalModeVariant {
        name: "lexical_only".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::LexicalOnly,
        embedding_backend: EmbeddingBackend::Reserved,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig {
            model: "builtin-16".to_string(),
            ..Default::default()
        }),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(&SearchRequest::new("baseline"))
        .expect("lexical_only variant search should still succeed when the embedding backend is reserved");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/variant-lexical-only-reserved"
    );
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::LexicalOnly,
        "lexical_only variant search should preserve lexical-only channel contribution when the embedding backend is reserved"
    );
}

#[test]
fn library_search_with_variant_uses_lexical_only_mode_when_vector_backend_is_none() {
    let path = fresh_db_path("variant-lexical-only-no-vector");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-lexical-only-no-vector",
            source_label: "variant lexical_only no vector memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T11:35:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let variant = RetrievalModeVariant {
        name: "lexical_only".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::LexicalOnly,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig {
            model: "builtin-16".to_string(),
            ..Default::default()
        }),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::None,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(&SearchRequest::new("baseline"))
        .expect("lexical_only variant search should still succeed when the vector backend is missing");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/variant-lexical-only-no-vector"
    );
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::LexicalOnly,
        "lexical_only variant search should preserve lexical-only channel contribution when the vector backend is missing"
    );
}

#[test]
fn library_search_with_runtime_config_keeps_lexical_only_when_embedding_channel_is_ready() {
    let path = fresh_db_path("runtime-config-lexical-only-ready-embedding");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-lexical-only-ready-embedding",
            source_label: "runtime config lexical only ready embedding memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T11:40:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::LexicalOnly,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(db.conn(), &config, None)
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("configured lexical_only search should still succeed when the embedding channel is ready");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::LexicalOnly,
        "configured lexical_only mode should keep lexical-only channel contribution even when the embedding channel is ready"
    );
    assert!(
        !response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Embedding),
        "configured lexical_only mode should not surface embedding strategies when the second channel is merely ready"
    );
}

#[test]
fn library_search_with_variant_keeps_lexical_only_when_embedding_channel_is_ready() {
    let path = fresh_db_path("variant-lexical-only-ready-embedding");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-lexical-only-ready-embedding",
            source_label: "variant lexical only ready embedding memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T11:45:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let variant = RetrievalModeVariant {
        name: "lexical_only".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::LexicalOnly,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig {
            model: "builtin-16".to_string(),
            ..Default::default()
        }),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("lexical_only variant search should still succeed when the embedding channel is ready");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::LexicalOnly,
        "lexical_only variant should keep lexical-only channel contribution even when the embedding channel is ready"
    );
    assert!(
        !response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Embedding),
        "lexical_only variant should not surface embedding strategies when the second channel is merely ready"
    );
}

#[test]
fn library_search_with_variant_uses_embedding_only_when_embedding_channel_is_ready() {
    let path = fresh_db_path("variant-embedding-only-ready-embedding");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-embedding-only-ready-embedding",
            source_label: "variant embedding_only ready embedding memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T12:45:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let variant = RetrievalModeVariant {
        name: "embedding_only".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::EmbeddingOnly,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig {
            model: "builtin-16".to_string(),
            ..Default::default()
        }),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("embedding_only variant search should succeed when the embedding channel is ready");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::EmbeddingOnly,
        "embedding_only variant should surface embedding-only contribution when the embedding channel is ready"
    );
    assert_eq!(
        response.results[0].trace.query_strategies,
        vec![agent_memos::search::QueryStrategy::Embedding],
        "embedding_only variant should only surface embedding strategies when the embedding channel is ready"
    );
}

#[test]
fn library_search_with_variant_preserves_source_metadata_for_embedding_only_ready_path() {
    let path = fresh_db_path("variant-embedding-only-source-metadata");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-embedding-only-source-metadata",
            source_label: "variant embedding_only source memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:17:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let variant = RetrievalModeVariant {
        name: "embedding_only".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::EmbeddingOnly,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig {
            model: "builtin-16".to_string(),
            ..Default::default()
        }),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("embedding_only variant search should preserve source metadata when ready");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.kind,
        agent_memos::memory::record::SourceKind::Document
    );
    assert_eq!(
        response.results[0].record.source.label.as_deref(),
        Some("variant embedding_only source memo")
    );
    assert_eq!(
        response.results[0].citation.source_kind,
        agent_memos::memory::record::SourceKind::Document
    );
    assert_eq!(
        response.results[0].citation.source_label.as_deref(),
        Some("variant embedding_only source memo")
    );
}

#[test]
fn library_search_with_variant_uses_unsuffixed_builtin_model_as_16_dimensions() {
    let path = fresh_db_path("variant-unsuffixed-builtin-model");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-unsuffixed-builtin-model",
            source_label: "variant unsuffixed builtin model memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:15:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let variant = RetrievalModeVariant {
        name: "embedding_only".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::EmbeddingOnly,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig {
            model: "builtin".to_string(),
            ..Default::default()
        }),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("unsuffixed builtin model should still work for embedding_only variant retrieval");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::EmbeddingOnly
    );
    assert_eq!(
        response.results[0].trace.query_strategies,
        vec![agent_memos::search::QueryStrategy::Embedding],
        "unsuffixed builtin model should fall back to the default 16-dimensional embedding path for variants"
    );
}

#[test]
fn library_search_with_variant_preserves_dsl_sidecar_for_embedding_only_ready_path() {
    let path = fresh_db_path("variant-embedding-only-dsl-sidecar");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-embedding-only-dsl-sidecar",
            source_label: "variant embedding only dsl sidecar memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:18:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let variant = RetrievalModeVariant {
        name: "embedding_only".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::EmbeddingOnly,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig {
            model: "builtin-16".to_string(),
            ..Default::default()
        }),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("embedding_only variant search should preserve the dsl sidecar");

    assert_eq!(response.results.len(), 1);
    let dsl = response.results[0]
        .dsl
        .as_ref()
        .expect("embedding_only variant search should attach the structured dsl sidecar");
    assert_eq!(dsl.domain, "project");
    assert_eq!(dsl.kind, "decision");
    assert_eq!(dsl.source_ref, "memo://project/variant-embedding-only-dsl-sidecar");
    assert!(!dsl.claim.is_empty());
}

#[test]
fn library_search_with_variant_uses_unsuffixed_builtin_model_for_hybrid_ready_path() {
    let path = fresh_db_path("variant-unsuffixed-builtin-model-hybrid");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-unsuffixed-builtin-model-hybrid",
            source_label: "variant unsuffixed builtin hybrid memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:25:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let variant = RetrievalModeVariant {
        name: "hybrid".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::Hybrid,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig {
            model: "builtin".to_string(),
            ..Default::default()
        }),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("unsuffixed builtin model should still work for hybrid variant retrieval");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::Hybrid
    );
    assert_eq!(
        response.results[0].trace.query_strategies,
        vec![
            agent_memos::search::QueryStrategy::Jieba,
            agent_memos::search::QueryStrategy::Simple,
            agent_memos::search::QueryStrategy::Structured,
            agent_memos::search::QueryStrategy::Embedding,
        ],
        "unsuffixed builtin model should fall back to the default 16-dimensional embedding path for hybrid variants"
    );
}

#[test]
fn library_search_with_variant_preserves_dsl_sidecar_for_hybrid_ready_path() {
    let path = fresh_db_path("variant-hybrid-dsl-sidecar");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-hybrid-dsl-sidecar",
            source_label: "variant hybrid dsl sidecar memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:28:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let variant = RetrievalModeVariant {
        name: "hybrid".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::Hybrid,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig {
            model: "builtin-16".to_string(),
            ..Default::default()
        }),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("hybrid variant search should preserve the dsl sidecar");

    assert_eq!(response.results.len(), 1);
    let dsl = response.results[0]
        .dsl
        .as_ref()
        .expect("hybrid variant search should attach the structured dsl sidecar");
    assert_eq!(dsl.domain, "project");
    assert_eq!(dsl.kind, "decision");
    assert_eq!(dsl.source_ref, "memo://project/variant-hybrid-dsl-sidecar");
    assert!(!dsl.claim.is_empty());
}

#[test]
fn library_search_with_variant_preserves_record_and_citation_shape_for_embedding_only_ready_path() {
    let path = fresh_db_path("variant-embedding-only-record-citation-shape");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-embedding-only-record-citation-shape",
            source_label: "variant embedding only record citation memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:19:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let variant = RetrievalModeVariant {
        name: "embedding_only".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::EmbeddingOnly,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig {
            model: "builtin-16".to_string(),
            ..Default::default()
        }),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("embedding_only variant ready-path search should preserve record and citation shape");

    assert_eq!(response.results.len(), 1);
    let result = &response.results[0];
    assert_eq!(result.record.scope, Scope::Project);
    assert_eq!(result.record.truth_layer, TruthLayer::T2);
    assert_eq!(result.record.record_type, RecordType::Decision);
    assert_eq!(result.citation.record_id, result.record.id);
    assert_eq!(result.citation.source_uri, result.record.source.uri);
    assert_eq!(result.citation.recorded_at, "2026-04-18T13:19:00Z");
    assert_eq!(
        result.citation.validity.valid_from.as_deref(),
        Some("2026-04-10T00:00:00Z")
    );
    assert_eq!(
        result.citation.validity.valid_to.as_deref(),
        Some("2026-04-20T00:00:00Z")
    );
    assert_eq!(result.citation.anchor.chunk_index, 0);
    assert_eq!(result.citation.anchor.chunk_count, 1);
}

#[test]
fn library_search_with_variant_preserves_record_and_citation_shape_for_hybrid_ready_path() {
    let path = fresh_db_path("variant-hybrid-record-citation-shape");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-hybrid-record-citation-shape",
            source_label: "variant hybrid record citation memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:29:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );

    let variant = RetrievalModeVariant {
        name: "hybrid".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::Hybrid,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig {
            model: "builtin-16".to_string(),
            ..Default::default()
        }),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("hybrid variant ready-path search should preserve record and citation shape");

    assert_eq!(response.results.len(), 1);
    let result = &response.results[0];
    assert_eq!(result.record.scope, Scope::Project);
    assert_eq!(result.record.truth_layer, TruthLayer::T2);
    assert_eq!(result.record.record_type, RecordType::Decision);
    assert_eq!(result.citation.record_id, result.record.id);
    assert_eq!(result.citation.source_uri, result.record.source.uri);
    assert_eq!(result.citation.recorded_at, "2026-04-18T13:29:00Z");
    assert_eq!(
        result.citation.validity.valid_from.as_deref(),
        Some("2026-04-10T00:00:00Z")
    );
    assert_eq!(
        result.citation.validity.valid_to.as_deref(),
        Some("2026-04-20T00:00:00Z")
    );
    assert_eq!(result.citation.anchor.chunk_index, 0);
    assert_eq!(result.citation.anchor.chunk_count, 1);
}

#[test]
fn library_search_with_variant_uses_hybrid_mode_when_embedding_channel_is_ready() {
    let path = fresh_db_path("variant-hybrid-ready-embedding");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-hybrid-ready-embedding",
            source_label: "variant hybrid ready embedding memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T12:55:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let variant = RetrievalModeVariant {
        name: "hybrid".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::Hybrid,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig {
            model: "builtin-16".to_string(),
            ..Default::default()
        }),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("hybrid variant search should succeed when the embedding channel is ready");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::Hybrid,
        "hybrid variant should surface hybrid contribution when the embedding channel is ready"
    );
    assert!(
        response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Embedding),
        "hybrid variant should surface embedding strategies when the embedding channel is ready"
    );
}

#[test]
fn library_search_with_variant_preserves_source_metadata_for_hybrid_ready_path() {
    let path = fresh_db_path("variant-hybrid-source-metadata");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-hybrid-source-metadata",
            source_label: "variant hybrid source memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:27:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let variant = RetrievalModeVariant {
        name: "hybrid".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::Hybrid,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig {
            model: "builtin-16".to_string(),
            ..Default::default()
        }),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("hybrid variant search should preserve source metadata when ready");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.kind,
        agent_memos::memory::record::SourceKind::Document
    );
    assert_eq!(
        response.results[0].record.source.label.as_deref(),
        Some("variant hybrid source memo")
    );
    assert_eq!(
        response.results[0].citation.source_kind,
        agent_memos::memory::record::SourceKind::Document
    );
    assert_eq!(
        response.results[0].citation.source_label.as_deref(),
        Some("variant hybrid source memo")
    );
}

#[test]
fn library_search_with_variant_reports_exact_hybrid_strategies_when_embedding_channel_is_ready() {
    let path = fresh_db_path("variant-hybrid-exact-strategies");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-hybrid-exact-strategies",
            source_label: "variant hybrid exact strategies memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:05:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let variant = RetrievalModeVariant {
        name: "hybrid".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::Hybrid,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig {
            model: "builtin-16".to_string(),
            ..Default::default()
        }),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("hybrid variant search should succeed when the embedding channel is ready");

    assert_eq!(
        response.results[0].trace.query_strategies,
        vec![
            agent_memos::search::QueryStrategy::Jieba,
            agent_memos::search::QueryStrategy::Simple,
            agent_memos::search::QueryStrategy::Structured,
            agent_memos::search::QueryStrategy::Embedding,
        ],
        "hybrid variant should preserve the full ready-path strategy ordering"
    );
}

#[test]
fn library_search_with_variant_embedding_only_applies_taxonomy_filters_before_top_k() {
    let path = fresh_db_path("variant-embedding-only-taxonomy-top-k");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-embedding-only-config-top-k",
            source_label: "variant embedding_only config top k memo",
            content: "baseline baseline keeps toml setting review stable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:44:00Z",
            valid_from: None,
            valid_to: None,
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-embedding-only-retrieval-top-k",
            source_label: "variant embedding_only retrieval top k memo",
            content: "baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:43:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let variant = RetrievalModeVariant {
        name: "embedding_only".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::EmbeddingOnly,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig {
            model: "builtin-16".to_string(),
            ..Default::default()
        }),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(
            &SearchRequest::new("baseline")
                .with_limit(1)
                .with_filters(SearchFilters {
                    topic: Some("retrieval".to_string()),
                    ..Default::default()
                }),
        )
        .expect("embedding_only variant search should apply taxonomy filters before top-k");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/variant-embedding-only-retrieval-top-k"
    );
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::EmbeddingOnly
    );
    assert_eq!(response.applied_filters.topic.as_deref(), Some("retrieval"));
}

#[test]
fn library_search_with_variant_hybrid_applies_taxonomy_filters_before_top_k() {
    let path = fresh_db_path("variant-hybrid-taxonomy-top-k");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-hybrid-config-top-k",
            source_label: "variant hybrid config top k memo",
            content: "baseline baseline keeps toml setting review stable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:46:00Z",
            valid_from: None,
            valid_to: None,
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-hybrid-retrieval-top-k",
            source_label: "variant hybrid retrieval top k memo",
            content: "baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:45:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let variant = RetrievalModeVariant {
        name: "hybrid".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::Hybrid,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig {
            model: "builtin-16".to_string(),
            ..Default::default()
        }),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(
            &SearchRequest::new("baseline")
                .with_limit(1)
                .with_filters(SearchFilters {
                    topic: Some("retrieval".to_string()),
                    ..Default::default()
                }),
        )
        .expect("hybrid variant search should apply taxonomy filters before top-k");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/variant-hybrid-retrieval-top-k"
    );
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::Hybrid
    );
    assert_eq!(response.applied_filters.topic.as_deref(), Some("retrieval"));
}

#[test]
fn library_search_with_variant_embedding_only_applies_temporal_filters_before_top_k() {
    let path = fresh_db_path("variant-embedding-only-temporal-top-k");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-embedding-only-current-temporal",
            source_label: "variant embedding_only current temporal memo",
            content: "baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:52:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-embedding-only-stale-temporal",
            source_label: "variant embedding_only stale temporal memo",
            content: "baseline baseline keeps stale review around",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-01T13:52:00Z",
            valid_from: Some("2026-03-01T00:00:00Z"),
            valid_to: Some("2026-04-05T00:00:00Z"),
        },
    );

    let variant = RetrievalModeVariant {
        name: "embedding_only".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::EmbeddingOnly,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig {
            model: "builtin-16".to_string(),
            ..Default::default()
        }),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(
            &SearchRequest::new("baseline")
                .with_limit(1)
                .with_filters(SearchFilters {
                    valid_at: Some("2026-04-18T14:00:00Z".to_string()),
                    recorded_from: Some("2026-04-10T00:00:00Z".to_string()),
                    recorded_to: Some("2026-04-19T00:00:00Z".to_string()),
                    ..Default::default()
                }),
        )
        .expect("embedding_only variant search should apply temporal filters before top-k");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/variant-embedding-only-current-temporal"
    );
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::EmbeddingOnly
    );
    assert_eq!(
        response.applied_filters.valid_at.as_deref(),
        Some("2026-04-18T14:00:00Z")
    );
}

#[test]
fn library_search_with_variant_hybrid_applies_temporal_filters_before_top_k() {
    let path = fresh_db_path("variant-hybrid-temporal-top-k");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-hybrid-current-temporal",
            source_label: "variant hybrid current temporal memo",
            content: "baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:54:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-hybrid-stale-temporal",
            source_label: "variant hybrid stale temporal memo",
            content: "baseline baseline keeps stale review around",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-01T13:54:00Z",
            valid_from: Some("2026-03-01T00:00:00Z"),
            valid_to: Some("2026-04-05T00:00:00Z"),
        },
    );

    let variant = RetrievalModeVariant {
        name: "hybrid".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::Hybrid,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig {
            model: "builtin-16".to_string(),
            ..Default::default()
        }),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(
            &SearchRequest::new("baseline")
                .with_limit(1)
                .with_filters(SearchFilters {
                    valid_at: Some("2026-04-18T14:00:00Z".to_string()),
                    recorded_from: Some("2026-04-10T00:00:00Z".to_string()),
                    recorded_to: Some("2026-04-19T00:00:00Z".to_string()),
                    ..Default::default()
                }),
        )
        .expect("hybrid variant search should apply temporal filters before top-k");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/variant-hybrid-current-temporal"
    );
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::Hybrid
    );
    assert_eq!(
        response.applied_filters.valid_at.as_deref(),
        Some("2026-04-18T14:00:00Z")
    );
}

#[test]
fn library_search_with_runtime_config_embedding_only_applies_temporal_filters_before_top_k() {
    let path = fresh_db_path("runtime-config-embedding-only-temporal-top-k");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-embedding-only-current-temporal",
            source_label: "runtime config embedding_only current temporal memo",
            content: "baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:48:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-embedding-only-stale-temporal",
            source_label: "runtime config embedding_only stale temporal memo",
            content: "baseline baseline keeps stale review around",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-01T13:48:00Z",
            valid_from: Some("2026-03-01T00:00:00Z"),
            valid_to: Some("2026-04-05T00:00:00Z"),
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::EmbeddingOnly,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(db.conn(), &config, None)
        .search(
            &SearchRequest::new("baseline")
                .with_limit(1)
                .with_filters(SearchFilters {
                    valid_at: Some("2026-04-18T14:00:00Z".to_string()),
                    recorded_from: Some("2026-04-10T00:00:00Z".to_string()),
                    recorded_to: Some("2026-04-19T00:00:00Z".to_string()),
                    ..Default::default()
                }),
        )
        .expect("configured embedding_only search should apply temporal filters before top-k");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/runtime-config-embedding-only-current-temporal"
    );
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::EmbeddingOnly
    );
    assert_eq!(
        response.applied_filters.valid_at.as_deref(),
        Some("2026-04-18T14:00:00Z")
    );
}

#[test]
fn library_search_with_runtime_config_hybrid_applies_temporal_filters_before_top_k() {
    let path = fresh_db_path("runtime-config-hybrid-temporal-top-k");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-hybrid-current-temporal",
            source_label: "runtime config hybrid current temporal memo",
            content: "baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T13:50:00Z",
            valid_from: Some("2026-04-10T00:00:00Z"),
            valid_to: Some("2026-04-20T00:00:00Z"),
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-hybrid-stale-temporal",
            source_label: "runtime config hybrid stale temporal memo",
            content: "baseline baseline keeps stale review around",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-01T13:50:00Z",
            valid_from: Some("2026-03-01T00:00:00Z"),
            valid_to: Some("2026-04-05T00:00:00Z"),
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::Hybrid,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(db.conn(), &config, None)
        .search(
            &SearchRequest::new("baseline")
                .with_limit(1)
                .with_filters(SearchFilters {
                    valid_at: Some("2026-04-18T14:00:00Z".to_string()),
                    recorded_from: Some("2026-04-10T00:00:00Z".to_string()),
                    recorded_to: Some("2026-04-19T00:00:00Z".to_string()),
                    ..Default::default()
                }),
        )
        .expect("configured hybrid search should apply temporal filters before top-k");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].record.source.uri,
        "memo://project/runtime-config-hybrid-current-temporal"
    );
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::Hybrid
    );
    assert_eq!(
        response.applied_filters.valid_at.as_deref(),
        Some("2026-04-18T14:00:00Z")
    );
}

#[test]
fn library_search_with_runtime_config_embedding_only_returns_no_results_when_embedding_model_mismatches_index()
{
    let path = fresh_db_path("runtime-config-embedding-model-mismatch");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-embedding-model-mismatch",
            source_label: "runtime config embedding model mismatch memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T12:10:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::LexicalOnly,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-32".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(
        db.conn(),
        &config,
        Some(RetrievalMode::EmbeddingOnly),
    )
    .search(&SearchRequest::new("retrieval fusion"))
    .expect("embedding_only library search should succeed when the configured model mismatches stored embeddings");

    assert!(
        response.results.is_empty(),
        "embedding_only service-level search should return no results when the configured model mismatches the stored embedding sidecar"
    );
}

#[test]
fn library_search_with_runtime_config_hybrid_falls_back_to_lexical_when_embedding_model_mismatches_index()
{
    let path = fresh_db_path("runtime-config-hybrid-model-mismatch");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/runtime-config-hybrid-model-mismatch",
            source_label: "runtime config hybrid model mismatch memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T12:15:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::LexicalOnly,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-32".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(db.conn(), &config, Some(RetrievalMode::Hybrid))
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("hybrid library search should succeed when the configured model mismatches stored embeddings");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::LexicalOnly,
        "hybrid service-level search should degrade to lexical-only channel contribution when the configured model mismatches the stored embedding sidecar"
    );
    assert!(
        !response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Embedding),
        "hybrid service-level search should not surface embedding strategies when the configured model mismatches stored embeddings"
    );
}

#[test]
fn library_search_with_variant_embedding_only_returns_no_results_when_embedding_model_mismatches_index()
{
    let path = fresh_db_path("variant-embedding-model-mismatch");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-embedding-model-mismatch",
            source_label: "variant embedding model mismatch memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T12:20:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let variant = RetrievalModeVariant {
        name: "embedding_only".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::EmbeddingOnly,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig {
            model: "builtin-32".to_string(),
            ..Default::default()
        }),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("embedding_only variant search should succeed when the configured model mismatches stored embeddings");

    assert!(
        response.results.is_empty(),
        "embedding_only variant search should return no results when the configured model mismatches the stored embedding sidecar"
    );
}

#[test]
fn library_search_with_variant_hybrid_falls_back_to_lexical_when_embedding_model_mismatches_index()
{
    let path = fresh_db_path("variant-hybrid-model-mismatch");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/variant-hybrid-model-mismatch",
            source_label: "variant hybrid model mismatch memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T12:25:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let variant = RetrievalModeVariant {
        name: "hybrid".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::Hybrid,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig {
            model: "builtin-32".to_string(),
            ..Default::default()
        }),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("hybrid variant search should succeed when the configured model mismatches stored embeddings");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::LexicalOnly,
        "hybrid variant search should degrade to lexical-only channel contribution when the configured model mismatches the stored embedding sidecar"
    );
    assert!(
        !response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Embedding),
        "hybrid variant search should not surface embedding strategies when the configured model mismatches stored embeddings"
    );
}

#[test]
fn library_search_with_runtime_config_embedding_only_returns_no_results_when_embedding_dimensions_mismatch()
{
    let path = fresh_db_path("runtime-config-embedding-dimension-mismatch");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    let report = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/runtime-config-embedding-dimension-mismatch".to_string(),
            source_label: Some("runtime config embedding dimension mismatch memo".to_string()),
            source_kind: None,
            content: "retrieval fusion semantic retrieval fusion citations".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T12:00:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("embedding dimension mismatch ingest should succeed");

    MemoryRepository::new(db.conn())
        .insert_record_embedding(&RecordEmbedding {
            record_id: report.record_ids[0].clone(),
            backend: EmbeddingBackend::Builtin,
            model: "builtin-16".to_string(),
            dimensions: 8,
            embedding: vec![0.25; 8],
            source_text_hash: "mismatched-dimensions".to_string(),
            created_at: "2026-04-18T12:00:00Z".to_string(),
            updated_at: "2026-04-18T12:00:00Z".to_string(),
        })
        .expect("mismatched embedding dimensions should overwrite the stored sidecar");

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::LexicalOnly,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(
        db.conn(),
        &config,
        Some(RetrievalMode::EmbeddingOnly),
    )
    .search(&SearchRequest::new("retrieval fusion"))
    .expect("embedding_only search should succeed when stored embedding dimensions mismatch the query embedding");

    assert!(
        response.results.is_empty(),
        "embedding_only search should return no results when stored embedding dimensions mismatch the query embedding"
    );
}

#[test]
fn library_search_with_runtime_config_hybrid_falls_back_to_lexical_when_embedding_dimensions_mismatch()
{
    let path = fresh_db_path("runtime-config-hybrid-dimension-mismatch");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    let report = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/runtime-config-hybrid-dimension-mismatch".to_string(),
            source_label: Some("runtime config hybrid dimension mismatch memo".to_string()),
            source_kind: None,
            content: "retrieval fusion semantic retrieval fusion citations".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T12:05:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("hybrid dimension mismatch ingest should succeed");

    MemoryRepository::new(db.conn())
        .insert_record_embedding(&RecordEmbedding {
            record_id: report.record_ids[0].clone(),
            backend: EmbeddingBackend::Builtin,
            model: "builtin-16".to_string(),
            dimensions: 8,
            embedding: vec![0.25; 8],
            source_text_hash: "hybrid-mismatched-dimensions".to_string(),
            created_at: "2026-04-18T12:05:00Z".to_string(),
            updated_at: "2026-04-18T12:05:00Z".to_string(),
        })
        .expect("mismatched hybrid embedding dimensions should overwrite the stored sidecar");

    let config = Config {
        retrieval: RetrievalConfig {
            mode: RetrievalMode::LexicalOnly,
        },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    };

    let response = SearchService::with_runtime_config(db.conn(), &config, Some(RetrievalMode::Hybrid))
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("hybrid search should succeed when stored embedding dimensions mismatch the query embedding");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::LexicalOnly,
        "hybrid search should degrade to lexical-only channel contribution when embedding dimensions mismatch"
    );
    assert!(
        !response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Embedding),
        "hybrid search should not surface embedding strategies when embedding dimensions mismatch"
    );
}

#[test]
fn library_search_with_variant_embedding_only_returns_no_results_when_embedding_dimensions_mismatch()
{
    let path = fresh_db_path("variant-embedding-dimension-mismatch");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    let report = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/variant-embedding-dimension-mismatch".to_string(),
            source_label: Some("variant embedding dimension mismatch memo".to_string()),
            source_kind: None,
            content: "retrieval fusion semantic retrieval fusion citations".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T12:10:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("variant embedding dimension mismatch ingest should succeed");

    MemoryRepository::new(db.conn())
        .insert_record_embedding(&RecordEmbedding {
            record_id: report.record_ids[0].clone(),
            backend: EmbeddingBackend::Builtin,
            model: "builtin-16".to_string(),
            dimensions: 8,
            embedding: vec![0.25; 8],
            source_text_hash: "variant-mismatched-dimensions".to_string(),
            created_at: "2026-04-18T12:10:00Z".to_string(),
            updated_at: "2026-04-18T12:10:00Z".to_string(),
        })
        .expect("mismatched variant embedding dimensions should overwrite the stored sidecar");

    let variant = RetrievalModeVariant {
        name: "embedding_only".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::EmbeddingOnly,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig {
            model: "builtin-16".to_string(),
            ..Default::default()
        }),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("embedding_only variant search should succeed when stored embedding dimensions mismatch the query embedding");

    assert!(
        response.results.is_empty(),
        "embedding_only variant search should return no results when stored embedding dimensions mismatch the query embedding"
    );
}

#[test]
fn library_search_with_variant_hybrid_falls_back_to_lexical_when_embedding_dimensions_mismatch() {
    let path = fresh_db_path("variant-hybrid-dimension-mismatch");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    let report = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/variant-hybrid-dimension-mismatch".to_string(),
            source_label: Some("variant hybrid dimension mismatch memo".to_string()),
            source_kind: None,
            content: "retrieval fusion semantic retrieval fusion citations".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T12:15:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("variant hybrid dimension mismatch ingest should succeed");

    MemoryRepository::new(db.conn())
        .insert_record_embedding(&RecordEmbedding {
            record_id: report.record_ids[0].clone(),
            backend: EmbeddingBackend::Builtin,
            model: "builtin-16".to_string(),
            dimensions: 8,
            embedding: vec![0.25; 8],
            source_text_hash: "variant-hybrid-mismatched-dimensions".to_string(),
            created_at: "2026-04-18T12:15:00Z".to_string(),
            updated_at: "2026-04-18T12:15:00Z".to_string(),
        })
        .expect("mismatched variant hybrid dimensions should overwrite the stored sidecar");

    let variant = RetrievalModeVariant {
        name: "hybrid".to_string(),
        db_path: path.display().to_string(),
        mode: RetrievalMode::Hybrid,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: Some(agent_memos::core::config::RootEmbeddingRuntimeConfig {
            model: "builtin-16".to_string(),
            ..Default::default()
        }),
        vector: Some(RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        }),
    };

    let response = SearchService::with_variant(db.conn(), &variant)
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("hybrid variant search should succeed when stored embedding dimensions mismatch the query embedding");

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::LexicalOnly,
        "hybrid variant search should degrade to lexical-only channel contribution when embedding dimensions mismatch"
    );
    assert!(
        !response.results[0]
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Embedding),
        "hybrid variant search should not surface embedding strategies when embedding dimensions mismatch"
    );
}

#[test]
fn cli_search_embedding_only_fails_closed_when_embedding_backend_is_disabled() {
    let dir = unique_temp_dir("embedding-only-disabled-backend");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before embedding_only readiness check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest = Database::open(&db_path).expect("database should bootstrap");
    let service = IngestService::new(ingest.conn());
    ingest_record(
        &service,
        FixtureRecord {
            source_uri: "memo://project/embedding-only-disabled",
            source_label: "embedding only disabled memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T09:00:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(&config_path, &["search", "baseline", "--mode", "embedding_only"]);
    let combined = format!(
        "{}\n{}",
        stdout(&search_output),
        stderr(&search_output)
    );

    assert!(
        !search_output.status.success(),
        "embedding_only search should fail closed when the embedding backend is disabled: {combined}"
    );
    assert!(
        combined.contains("ready: false"),
        "failure output should include readiness=false for embedding_only mode: {combined}"
    );
    assert!(
        combined.contains("embedding_only requires a non-disabled embedding backend"),
        "failure output should explain the missing embedding backend requirement: {combined}"
    );
    assert!(
        combined.contains("embedding backend is not ready for embedding_only retrieval"),
        "failure output should explain the missing embedding backend readiness: {combined}"
    );
}

#[test]
fn cli_search_hybrid_fails_closed_when_embedding_backend_is_disabled() {
    let dir = unique_temp_dir("hybrid-disabled-backend");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config(&config_path, &db_path);

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before hybrid readiness check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest = Database::open(&db_path).expect("database should bootstrap");
    let service = IngestService::new(ingest.conn());
    ingest_record(
        &service,
        FixtureRecord {
            source_uri: "memo://project/hybrid-disabled",
            source_label: "hybrid disabled memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T09:05:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(&config_path, &["search", "baseline", "--mode", "hybrid"]);
    let combined = format!(
        "{}\n{}",
        stdout(&search_output),
        stderr(&search_output)
    );

    assert!(
        !search_output.status.success(),
        "hybrid search should fail closed when the embedding backend is disabled: {combined}"
    );
    assert!(
        combined.contains("ready: false"),
        "failure output should include readiness=false for hybrid mode: {combined}"
    );
    assert!(
        combined.contains("hybrid requires an embedding backend for the secondary path"),
        "failure output should explain the missing hybrid embedding requirement: {combined}"
    );
    assert!(
        combined.contains("embedding backend is not ready for hybrid retrieval"),
        "failure output should explain the missing hybrid embedding backend readiness: {combined}"
    );
}

#[test]
fn cli_search_embedding_only_fails_closed_when_embedding_model_is_missing() {
    let dir = unique_temp_dir("embedding-only-missing-model");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "embedding_only",
        "builtin",
        None,
        Some("sqlite_vec"),
    );

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before embedding_only missing-model readiness check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest = Database::open(&db_path).expect("database should bootstrap");
    let service = IngestService::new(ingest.conn());
    ingest_record(
        &service,
        FixtureRecord {
            source_uri: "memo://project/embedding-only-missing-model",
            source_label: "embedding only missing model memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T09:10:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(&config_path, &["search", "baseline", "--mode", "embedding_only"]);
    let combined = format!(
        "{}\n{}",
        stdout(&search_output),
        stderr(&search_output)
    );

    assert!(
        !search_output.status.success(),
        "embedding_only search should fail closed when the embedding model is missing: {combined}"
    );
    assert!(
        combined.contains("ready: false"),
        "failure output should include readiness=false for embedding_only mode: {combined}"
    );
    assert!(
        combined.contains("embedding backend is not ready for embedding_only retrieval"),
        "failure output should explain the missing embedding backend readiness: {combined}"
    );
}

#[test]
fn cli_search_hybrid_fails_closed_when_embedding_model_is_missing() {
    let dir = unique_temp_dir("hybrid-missing-model");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "hybrid",
        "builtin",
        None,
        Some("sqlite_vec"),
    );

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before hybrid missing-model readiness check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest = Database::open(&db_path).expect("database should bootstrap");
    let service = IngestService::new(ingest.conn());
    ingest_record(
        &service,
        FixtureRecord {
            source_uri: "memo://project/hybrid-missing-model",
            source_label: "hybrid missing model memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T09:15:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(&config_path, &["search", "baseline", "--mode", "hybrid"]);
    let combined = format!(
        "{}\n{}",
        stdout(&search_output),
        stderr(&search_output)
    );

    assert!(
        !search_output.status.success(),
        "hybrid search should fail closed when the embedding model is missing: {combined}"
    );
    assert!(
        combined.contains("ready: false"),
        "failure output should include readiness=false for hybrid mode: {combined}"
    );
    assert!(
        combined.contains("embedding backend is not ready for hybrid retrieval"),
        "failure output should explain the missing hybrid embedding backend readiness: {combined}"
    );
}

#[test]
fn cli_search_embedding_only_returns_empty_results_when_vector_backend_is_none() {
    let dir = unique_temp_dir("embedding-only-missing-vector");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "embedding_only",
        "builtin",
        Some("builtin-16"),
        None,
    );

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before embedding_only missing-vector readiness check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest = Database::open(&db_path).expect("database should bootstrap");
    let service = IngestService::new(ingest.conn());
    ingest_record(
        &service,
        FixtureRecord {
            source_uri: "memo://project/embedding-only-missing-vector",
            source_label: "embedding only missing vector memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T09:20:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(&config_path, &["search", "baseline", "--mode", "embedding_only"]);
    let combined = format!(
        "{}\n{}",
        stdout(&search_output),
        stderr(&search_output)
    );

    assert!(
        search_output.status.success(),
        "embedding_only search should still succeed when the vector backend is missing: {combined}"
    );
    assert!(
        combined.contains("results: 0"),
        "embedding_only search should surface an empty result set when the vector backend is missing: {combined}"
    );
}

#[test]
fn cli_search_hybrid_falls_back_to_lexical_when_vector_backend_is_none() {
    let dir = unique_temp_dir("hybrid-missing-vector");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "hybrid",
        "builtin",
        Some("builtin-16"),
        None,
    );

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before hybrid missing-vector readiness check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest = Database::open(&db_path).expect("database should bootstrap");
    let service = IngestService::new(ingest.conn());
    ingest_record(
        &service,
        FixtureRecord {
            source_uri: "memo://project/hybrid-missing-vector",
            source_label: "hybrid missing vector memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T09:25:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(&config_path, &["search", "baseline", "--mode", "hybrid"]);
    let combined = format!(
        "{}\n{}",
        stdout(&search_output),
        stderr(&search_output)
    );

    assert!(
        search_output.status.success(),
        "hybrid search should still succeed when the vector backend is missing: {combined}"
    );
    assert!(
        combined.contains("memo://project/hybrid-missing-vector"),
        "hybrid search should still surface the lexical result when the vector backend is missing: {combined}"
    );
    assert!(
        combined.contains("channel: lexical_only"),
        "hybrid search should degrade to lexical_only channel contribution when the vector backend is missing: {combined}"
    );
}

#[test]
fn cli_search_embedding_only_fails_closed_when_embedding_backend_is_reserved() {
    let dir = unique_temp_dir("embedding-only-reserved-backend");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "embedding_only",
        "reserved",
        Some("builtin-16"),
        Some("sqlite_vec"),
    );

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before embedding_only reserved-backend readiness check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest = Database::open(&db_path).expect("database should bootstrap");
    let service = IngestService::new(ingest.conn());
    ingest_record(
        &service,
        FixtureRecord {
            source_uri: "memo://project/embedding-only-reserved-backend",
            source_label: "embedding only reserved backend memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T09:30:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(&config_path, &["search", "baseline", "--mode", "embedding_only"]);
    let combined = format!(
        "{}\n{}",
        stdout(&search_output),
        stderr(&search_output)
    );

    assert!(
        !search_output.status.success(),
        "embedding_only search should fail closed when the embedding backend is reserved: {combined}"
    );
    assert!(
        combined.contains("ready: false"),
        "failure output should include readiness=false for embedding_only reserved mode: {combined}"
    );
    assert!(
        combined.contains("embedding_only is reserved but not implemented in Phase 1"),
        "failure output should explain the reserved embedding_only mode: {combined}"
    );
    assert!(
        combined.contains("embedding backend is not ready for embedding_only retrieval"),
        "failure output should still report embedding readiness failure for reserved embedding_only mode: {combined}"
    );
}

#[test]
fn cli_search_hybrid_fails_closed_when_embedding_backend_is_reserved() {
    let dir = unique_temp_dir("hybrid-reserved-backend");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "hybrid",
        "reserved",
        Some("builtin-16"),
        Some("sqlite_vec"),
    );

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before hybrid reserved-backend readiness check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest = Database::open(&db_path).expect("database should bootstrap");
    let service = IngestService::new(ingest.conn());
    ingest_record(
        &service,
        FixtureRecord {
            source_uri: "memo://project/hybrid-reserved-backend",
            source_label: "hybrid reserved backend memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T09:35:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(&config_path, &["search", "baseline", "--mode", "hybrid"]);
    let combined = format!(
        "{}\n{}",
        stdout(&search_output),
        stderr(&search_output)
    );

    assert!(
        !search_output.status.success(),
        "hybrid search should fail closed when the embedding backend is reserved: {combined}"
    );
    assert!(
        combined.contains("ready: false"),
        "failure output should include readiness=false for hybrid reserved mode: {combined}"
    );
    assert!(
        combined.contains(
            "hybrid keeps lexical as the primary baseline, but the embedding secondary path is reserved in Phase 1"
        ),
        "failure output should explain the reserved hybrid mode: {combined}"
    );
    assert!(
        combined.contains("embedding backend is not ready for hybrid retrieval"),
        "failure output should still report embedding readiness failure for reserved hybrid mode: {combined}"
    );
}

#[test]
fn cli_search_succeeds_when_lexical_only_mode_uses_reserved_embedding_backend() {
    let dir = unique_temp_dir("lexical-only-reserved-backend");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "lexical_only",
        "reserved",
        Some("builtin-16"),
        Some("sqlite_vec"),
    );

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before lexical_only reserved-backend search check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest = Database::open(&db_path).expect("database should bootstrap");
    let service = IngestService::new(ingest.conn());
    ingest_record(
        &service,
        FixtureRecord {
            source_uri: "memo://project/lexical-only-reserved-backend",
            source_label: "lexical only reserved backend memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T09:40:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(&config_path, &["search", "baseline"]);
    let combined = format!("{}\n{}", stdout(&search_output), stderr(&search_output));

    assert!(
        search_output.status.success(),
        "lexical_only search should continue to work when the embedding backend is reserved: {combined}"
    );
    assert!(
        combined.contains("memo://project/lexical-only-reserved-backend"),
        "lexical_only search should still surface the lexical result when the embedding backend is reserved: {combined}"
    );
    assert!(
        combined.contains("channel: lexical_only"),
        "lexical_only search should preserve lexical-only channel output when the embedding backend is reserved: {combined}"
    );
}

#[test]
fn cli_search_succeeds_when_lexical_only_mode_has_no_vector_backend() {
    let dir = unique_temp_dir("lexical-only-no-vector-backend");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "lexical_only",
        "builtin",
        Some("builtin-16"),
        None,
    );

    let init_output = run_cli(&config_path, &["init"]);
    assert!(
        init_output.status.success(),
        "cli init should succeed before lexical_only no-vector search check: stdout={} stderr={}",
        stdout(&init_output),
        stderr(&init_output)
    );

    let ingest = Database::open(&db_path).expect("database should bootstrap");
    let service = IngestService::new(ingest.conn());
    ingest_record(
        &service,
        FixtureRecord {
            source_uri: "memo://project/lexical-only-no-vector-backend",
            source_label: "lexical only no vector backend memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-18T09:45:00Z",
            valid_from: None,
            valid_to: None,
        },
    );

    let search_output = run_cli(&config_path, &["search", "baseline"]);
    let combined = format!("{}\n{}", stdout(&search_output), stderr(&search_output));

    assert!(
        search_output.status.success(),
        "lexical_only search should continue to work when the vector backend is missing: {combined}"
    );
    assert!(
        combined.contains("memo://project/lexical-only-no-vector-backend"),
        "lexical_only search should still surface the lexical result when the vector backend is missing: {combined}"
    );
    assert!(
        combined.contains("channel: lexical_only"),
        "lexical_only search should preserve lexical-only channel output when the vector backend is missing: {combined}"
    );
}
