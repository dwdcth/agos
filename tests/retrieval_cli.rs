use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    core::db::Database,
    ingest::{IngestRequest, IngestService},
    memory::record::{RecordType, Scope, TruthLayer},
    search::{SearchFilters, SearchRequest, SearchService},
};

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
