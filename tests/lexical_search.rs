use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    core::db::Database,
    ingest::{IngestRequest, IngestService},
    memory::record::{RecordType, Scope, TruthLayer},
    search::{QueryStrategy, SearchFilters, SearchRequest, SearchService},
};

fn fresh_db_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-search-tests")
        .join(format!("{name}-{unique}"))
        .join("lexical.sqlite")
}

fn ingest_record(
    service: &IngestService<'_>,
    source_uri: &str,
    source_label: &str,
    content: &str,
    record_type: RecordType,
    recorded_at: &str,
) {
    service
        .ingest(IngestRequest {
            source_uri: source_uri.to_string(),
            source_label: Some(source_label.to_string()),
            source_kind: None,
            content: content.to_string(),
            scope: Scope::Project,
            record_type,
            truth_layer: TruthLayer::T2,
            recorded_at: recorded_at.to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest should succeed");
}

#[test]
fn lexical_search_recalls_chinese_and_pinyin_queries() {
    let path = fresh_db_path("mixed-script");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        "memo://project/briefing",
        "项目备忘录",
        "团队确认普通检索继续走 lexical first 路线，并保留出处和时间性说明。",
        RecordType::Decision,
        "2026-04-15T09:00:00Z",
    );
    ingest_record(
        &ingest,
        "memo://project/ops",
        "operations note",
        "The deployment checklist stays local and does not mention memo retrieval.",
        RecordType::Observation,
        "2026-04-15T09:05:00Z",
    );

    let search = SearchService::new(db.conn());

    let chinese_results = search
        .search(&SearchRequest::new("检索 路线"))
        .expect("chinese lexical search should succeed");
    assert!(
        !chinese_results.results.is_empty(),
        "chinese query should recall lexical candidate"
    );
    assert_eq!(
        chinese_results.results[0].record.source.uri, "memo://project/briefing",
        "chinese query should recall the authority-backed chunk"
    );
    assert!(
        chinese_results.results[0]
            .trace
            .query_strategies
            .contains(&QueryStrategy::Jieba),
        "chinese query should record jieba-based matching"
    );
    assert!(
        chinese_results.results[0].snippet.contains("lexical")
            || chinese_results.results[0].snippet.contains("检索"),
        "snippet should explain the match: {:?}",
        chinese_results.results[0].snippet
    );
    assert!(
        chinese_results.results[0].dsl.is_some(),
        "ordinary retrieval should carry the structured DSL sidecar alongside the authority row"
    );
    assert!(
        !chinese_results.results[0]
            .dsl
            .as_ref()
            .expect("dsl sidecar should exist")
            .claim
            .is_empty(),
        "structured sidecar should expose a non-empty compressed claim"
    );

    let pinyin_results = search
        .search(&SearchRequest::new("beiwanglu"))
        .expect("pinyin lexical search should succeed");
    assert!(
        !pinyin_results.results.is_empty(),
        "pinyin query should recall indexed chinese content"
    );
    assert_eq!(
        pinyin_results.results[0].record.source.uri,
        "memo://project/briefing"
    );
    assert!(
        pinyin_results.results[0]
            .trace
            .query_strategies
            .contains(&QueryStrategy::Simple),
        "pinyin query should record simple-query matching"
    );
    assert!(
        pinyin_results.results[0].dsl.is_some(),
        "pinyin recall should preserve the same structured sidecar payload"
    );
}

#[test]
fn lexical_search_score_breakdown_is_deterministic() {
    let path = fresh_db_path("score-breakdown");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        "memo://project/search-decision",
        "search decision memo",
        "lexical ranking should stay inspectable and deterministic for ordinary retrieval",
        RecordType::Decision,
        "2026-04-15T10:00:00Z",
    );
    ingest_record(
        &ingest,
        "memo://project/search-observation",
        "search note",
        "lexical ranking decision logs should stay inspectable and deterministic for ordinary retrieval",
        RecordType::Observation,
        "2026-04-10T10:00:00Z",
    );

    let search = SearchService::new(db.conn());
    let results = search
        .search(&SearchRequest::new("lexical ranking decision"))
        .expect("lexical scoring should succeed");

    assert_eq!(
        results.results.len(),
        2,
        "both comparable candidates should be recalled"
    );
    assert_eq!(
        results.results[0].record.source.uri, "memo://project/search-decision",
        "importance and keyword bonus should perturb the final ranking deterministically"
    );
    assert!(
        results.results[0].score.lexical_base >= results.results[0].score.keyword_bonus,
        "lexical score should remain the dominant term: {:?}",
        results.results[0].score
    );
    assert!(
        results.results[0].score.keyword_bonus > results.results[1].score.keyword_bonus,
        "the decision memo should get the stronger keyword overlap bonus"
    );
    assert!(
        results.results[0].score.importance_bonus > results.results[1].score.importance_bonus,
        "record-type defaults should contribute a deterministic importance bonus"
    );
    assert_eq!(
        results.results[0].score.emotion_bonus, 0.0,
        "emotion defaults should stay explicit when no signal exists"
    );
    assert!(
        results.results[0].score.final_score > results.results[1].score.final_score,
        "bonus composition should produce a stable final ranking"
    );
}

#[test]
fn lexical_search_dedupes_repeated_query_terms_before_keyword_scoring() {
    let path = fresh_db_path("dedupe-repeated-query-terms");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        "memo://project/repeated-query-terms",
        "repeated query terms memo",
        "lexical ranking decision logs should stay inspectable and deterministic",
        RecordType::Decision,
        "2026-04-15T10:30:00Z",
    );

    let search = SearchService::new(db.conn());
    let single = search
        .search(&SearchRequest::new("lexical ranking decision"))
        .expect("single-term lexical search should succeed");
    let repeated = search
        .search(&SearchRequest::new(
            "lexical lexical ranking decision decision",
        ))
        .expect("repeated-term lexical search should succeed");

    assert_eq!(single.results.len(), 1);
    assert_eq!(repeated.results.len(), 1);
    assert_eq!(
        repeated.results[0].score.keyword_bonus,
        single.results[0].score.keyword_bonus,
        "repeating the same query tokens should not inflate keyword bonus"
    );
}

#[test]
fn lexical_search_orders_recency_by_parsed_rfc3339_instant() {
    let path = fresh_db_path("parsed-recency-order");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        "memo://project/older-offset",
        "older offset memo",
        "recency ordering should use parsed instants across offsets",
        RecordType::Observation,
        "2026-04-16T08:00:00+02:00",
    );
    ingest_record(
        &ingest,
        "memo://project/newer-zulu",
        "newer zulu memo",
        "recency ordering should use parsed instants across offsets",
        RecordType::Observation,
        "2026-04-16T06:30:00Z",
    );

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("parsed instants offsets"))
        .expect("lexical search should succeed");

    assert_eq!(results.results.len(), 2);
    assert_eq!(
        results.results[0].record.source.uri,
        "memo://project/newer-zulu",
        "recency bonus should rank the truly newer instant ahead of the lexically larger offset timestamp"
    );
    assert!(
        results.results[0].score.recency_bonus > results.results[1].score.recency_bonus,
        "parsed recency should award the newer instant the stronger recency bonus"
    );
}

#[test]
fn lexical_search_applies_recorded_from_filter_by_parsed_rfc3339_instant() {
    let path = fresh_db_path("parsed-recorded-from-filter");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        "memo://project/older-offset-filter",
        "older offset filter memo",
        "temporal filtering should use parsed instants across offsets",
        RecordType::Observation,
        "2026-04-16T08:00:00+02:00",
    );
    ingest_record(
        &ingest,
        "memo://project/newer-zulu-filter",
        "newer zulu filter memo",
        "temporal filtering should use parsed instants across offsets",
        RecordType::Observation,
        "2026-04-16T06:30:00Z",
    );

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("parsed instants offsets").with_filters(SearchFilters {
            recorded_from: Some("2026-04-16T06:15:00Z".to_string()),
            ..Default::default()
        }))
        .expect("lexical search should respect parsed recorded_from filters");

    assert_eq!(results.results.len(), 1);
    assert_eq!(
        results.results[0].record.source.uri,
        "memo://project/newer-zulu-filter",
        "recorded_from should exclude the older offset-formatted instant even when its raw string sorts later"
    );
}

#[test]
fn lexical_search_preserves_structured_only_provenance_under_lexical_first_channel() {
    let path = fresh_db_path("structured-only-provenance");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        "memo://project/structured-only",
        "structured only memo",
        "use lexical-first as baseline because explainability matters",
        RecordType::Decision,
        "2026-04-15T12:00:00Z",
    );

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision"))
        .expect("structured-only lexical-first search should succeed");

    assert_eq!(results.results.len(), 1);
    assert_eq!(
        results.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::LexicalOnly
    );
    assert!(
        results.results[0]
            .trace
            .query_strategies
            .contains(&QueryStrategy::Structured),
        "structured-only library recall should still preserve structured provenance"
    );
    assert!(
        results.results[0].snippet.contains("WHY:"),
        "structured-only recall should already use the structured snippet surface: {:?}",
        results.results[0].snippet
    );
}

#[test]
fn lexical_search_respects_validity_window_filters() {
    let path = fresh_db_path("validity-window");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/current".to_string(),
            source_label: Some("current memo".to_string()),
            source_kind: None,
            content: "retrieval baseline is currently valid and explainable".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T12:30:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("current ingest should succeed");
    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/expired".to_string(),
            source_label: Some("expired memo".to_string()),
            source_kind: None,
            content: "retrieval baseline was valid before but is now expired".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-01T12:30:00Z".to_string(),
            valid_from: Some("2026-03-01T00:00:00Z".to_string()),
            valid_to: Some("2026-04-05T00:00:00Z".to_string()),
        })
        .expect("expired ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(
            &SearchRequest::new("retrieval baseline").with_filters(SearchFilters {
                valid_at: Some("2026-04-15T12:45:00Z".to_string()),
                ..Default::default()
            }),
        )
        .expect("validity-filtered lexical search should succeed");

    assert_eq!(results.results.len(), 1);
    assert_eq!(
        results.results[0].record.source.uri, "memo://project/current",
        "valid_at should exclude expired authority records from ordinary lexical recall"
    );
    assert_eq!(
        results.results[0].citation.validity.valid_to.as_deref(),
        Some("2026-04-20T00:00:00Z")
    );
}

#[test]
fn lexical_search_applies_validity_filters_to_structured_only_queries() {
    let path = fresh_db_path("structured-only-validity");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/current-structured".to_string(),
            source_label: Some("current structured memo".to_string()),
            source_kind: None,
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T12:40:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("current structured ingest should succeed");
    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/expired-structured".to_string(),
            source_label: Some("expired structured memo".to_string()),
            source_kind: None,
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-01T12:40:00Z".to_string(),
            valid_from: Some("2026-03-01T00:00:00Z".to_string()),
            valid_to: Some("2026-04-05T00:00:00Z".to_string()),
        })
        .expect("expired structured ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision").with_filters(SearchFilters {
            valid_at: Some("2026-04-15T12:45:00Z".to_string()),
            ..Default::default()
        }))
        .expect("structured-only validity-filtered search should succeed");

    assert_eq!(results.results.len(), 1);
    assert_eq!(
        results.results[0].record.source.uri, "memo://project/current-structured",
        "valid_at should still exclude expired records even when recall is driven by structured taxonomy fields"
    );
    assert_eq!(
        results.results[0].citation.validity.valid_from.as_deref(),
        Some("2026-04-10T00:00:00Z")
    );
    assert_eq!(
        results.results[0].citation.validity.valid_to.as_deref(),
        Some("2026-04-20T00:00:00Z")
    );
    assert!(
        results.results[0].snippet.contains("WHY:"),
        "structured-only validity-filtered recall should keep the structured snippet surface: {:?}",
        results.results[0].snippet
    );
}

#[test]
fn lexical_search_preserves_mixed_raw_and_structured_provenance_under_lexical_first_channel() {
    let path = fresh_db_path("mixed-structured-provenance");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        "memo://project/mixed-structured",
        "mixed structured memo",
        "use lexical-first as baseline because explainability matters",
        RecordType::Decision,
        "2026-04-15T12:10:00Z",
    );

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline"))
        .expect("mixed lexical + structured search should succeed");

    assert_eq!(results.results.len(), 1);
    assert_eq!(
        results.results[0].trace.channel_contribution,
        agent_memos::search::ChannelContribution::LexicalOnly
    );
    assert!(
        results.results[0]
            .trace
            .query_strategies
            .contains(&QueryStrategy::Simple),
        "mixed recall should preserve raw lexical provenance"
    );
    assert!(
        results.results[0]
            .trace
            .query_strategies
            .contains(&QueryStrategy::Structured),
        "mixed recall should preserve structured provenance"
    );
}

#[test]
fn lexical_search_prefers_structured_snippet_when_mixed_recall_occurs() {
    let path = fresh_db_path("mixed-structured-snippet");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest_record(
        &ingest,
        "memo://project/mixed-structured-snippet",
        "mixed structured snippet memo",
        "use lexical-first as baseline because explainability matters",
        RecordType::Decision,
        "2026-04-15T12:20:00Z",
    );

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline"))
        .expect("mixed lexical + structured search should succeed");

    assert_eq!(results.results.len(), 1);
    assert!(
        results.results[0].snippet.contains("WHY:"),
        "mixed recall should prefer the structured snippet over the raw lexical fragment: {:?}",
        results.results[0].snippet
    );
}

#[test]
fn lexical_search_applies_validity_filters_to_mixed_recall_queries() {
    let path = fresh_db_path("mixed-validity");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/current-mixed".to_string(),
            source_label: Some("current mixed memo".to_string()),
            source_kind: None,
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T12:50:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("current mixed ingest should succeed");
    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/expired-mixed".to_string(),
            source_label: Some("expired mixed memo".to_string()),
            source_kind: None,
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-01T12:50:00Z".to_string(),
            valid_from: Some("2026-03-01T00:00:00Z".to_string()),
            valid_to: Some("2026-04-05T00:00:00Z".to_string()),
        })
        .expect("expired mixed ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(
            &SearchRequest::new("lexical-first baseline").with_filters(SearchFilters {
                valid_at: Some("2026-04-15T12:55:00Z".to_string()),
                ..Default::default()
            }),
        )
        .expect("mixed validity-filtered search should succeed");

    assert_eq!(results.results.len(), 1);
    assert_eq!(
        results.results[0].record.source.uri, "memo://project/current-mixed",
        "valid_at should exclude expired records even when recall uses both lexical and structured signals"
    );
    assert!(
        results.results[0]
            .trace
            .query_strategies
            .contains(&QueryStrategy::Simple)
            && results.results[0]
                .trace
                .query_strategies
                .contains(&QueryStrategy::Structured),
        "mixed validity-filtered recall should preserve both provenance branches"
    );
}

#[test]
fn lexical_search_preserves_source_metadata_for_structured_only_recall() {
    let path = fresh_db_path("structured-only-source-metadata");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/structured-only-source-metadata".to_string(),
            source_label: Some("structured source metadata memo".to_string()),
            source_kind: Some(agent_memos::memory::record::SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T13:00:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("structured-only source-metadata ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision"))
        .expect("structured-only lexical-first search should succeed");

    assert_eq!(results.results.len(), 1);
    assert_eq!(
        results.results[0].record.source.kind,
        agent_memos::memory::record::SourceKind::Note
    );
    assert_eq!(
        results.results[0].record.source.label.as_deref(),
        Some("structured source metadata memo")
    );
    assert_eq!(
        results.results[0].citation.source_kind,
        agent_memos::memory::record::SourceKind::Note
    );
    assert_eq!(
        results.results[0].citation.source_label.as_deref(),
        Some("structured source metadata memo")
    );
    assert_eq!(
        results.results[0].record.source.uri,
        "memo://project/structured-only-source-metadata"
    );
    assert_eq!(
        results.results[0].citation.source_uri,
        "memo://project/structured-only-source-metadata"
    );
}

#[test]
fn lexical_search_preserves_source_uri_for_structured_only_recall() {
    let path = fresh_db_path("structured-only-source-uri");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/structured-only-source-uri".to_string(),
            source_label: Some("structured only source uri memo".to_string()),
            source_kind: Some(agent_memos::memory::record::SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T13:05:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("structured-only source-uri ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision"))
        .expect("structured-only lexical-first search should succeed");

    assert_eq!(results.results.len(), 1);
    assert_eq!(
        results.results[0].record.source.uri,
        "memo://project/structured-only-source-uri"
    );
    assert_eq!(
        results.results[0].citation.source_uri,
        "memo://project/structured-only-source-uri"
    );
    assert!(
        results.results[0]
            .trace
            .query_strategies
            .contains(&QueryStrategy::Structured),
        "structured-only source-uri recall should preserve structured provenance"
    );
    assert_eq!(results.results[0].trace.matched_query, "decision");
    assert_eq!(
        results.results[0].citation.record_id,
        results.results[0].record.id
    );
}

#[test]
fn lexical_search_preserves_recorded_at_for_structured_only_recall() {
    let path = fresh_db_path("structured-only-recorded-at");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/structured-only-recorded-at".to_string(),
            source_label: Some("structured only recorded-at memo".to_string()),
            source_kind: Some(agent_memos::memory::record::SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T13:07:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("structured-only recorded-at ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision"))
        .expect("structured-only lexical-first search should succeed");

    assert_eq!(results.results.len(), 1);
    assert_eq!(
        results.results[0].citation.recorded_at,
        "2026-04-15T13:07:00Z"
    );
    assert_eq!(
        results.results[0].record.timestamp.recorded_at,
        "2026-04-15T13:07:00Z"
    );
    assert_eq!(
        results.results[0].record.timestamp.created_at,
        "2026-04-15T13:07:00Z"
    );
    assert_eq!(
        results.results[0].record.timestamp.updated_at,
        "2026-04-15T13:07:00Z"
    );
    assert!(
        results.results[0]
            .trace
            .query_strategies
            .contains(&QueryStrategy::Structured),
        "structured-only recorded-at recall should preserve structured provenance"
    );
}

#[test]
fn lexical_search_preserves_line_range_anchor_for_structured_only_recall() {
    let path = fresh_db_path("structured-only-line-range-anchor");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/structured-only-line-range-anchor".to_string(),
            source_label: Some("structured only line-range anchor memo".to_string()),
            source_kind: Some(agent_memos::memory::record::SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T13:08:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("structured-only line-range anchor ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision"))
        .expect("structured-only lexical-first search should succeed");

    assert_eq!(results.results.len(), 1);
    assert!(matches!(
        results.results[0].citation.anchor.anchor,
        agent_memos::memory::record::ChunkAnchor::LineRange {
            start_line: 1,
            end_line: 1
        }
    ));
    assert!(
        results.results[0]
            .trace
            .query_strategies
            .contains(&QueryStrategy::Structured),
        "structured-only line-range anchor recall should preserve structured provenance"
    );
}

#[test]
fn lexical_search_preserves_chunk_anchor_for_structured_only_recall() {
    let path = fresh_db_path("structured-only-chunk-anchor");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/structured-only-chunk-anchor".to_string(),
            source_label: Some("structured only chunk-anchor memo".to_string()),
            source_kind: Some(agent_memos::memory::record::SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T13:09:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("structured-only chunk-anchor ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision"))
        .expect("structured-only lexical-first search should succeed");

    assert_eq!(results.results.len(), 1);
    assert_eq!(results.results[0].citation.anchor.chunk_index, 0);
    assert_eq!(results.results[0].citation.anchor.chunk_count, 1);
    assert!(
        results.results[0]
            .trace
            .query_strategies
            .contains(&QueryStrategy::Structured),
        "structured-only chunk-anchor recall should preserve structured provenance"
    );
}

#[test]
fn lexical_search_preserves_record_scope_and_truth_layer_for_structured_only_recall() {
    let path = fresh_db_path("structured-only-record-shape");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/structured-only-record-shape".to_string(),
            source_label: Some("structured only record shape memo".to_string()),
            source_kind: Some(agent_memos::memory::record::SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T13:09:30Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("structured-only record-shape ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision"))
        .expect("structured-only lexical-first search should succeed");

    assert_eq!(results.results.len(), 1);
    assert_eq!(results.results[0].record.scope, Scope::Project);
    assert_eq!(results.results[0].record.truth_layer, TruthLayer::T2);
    assert_eq!(results.results[0].record.record_type, RecordType::Decision);
    assert!(
        results.results[0]
            .trace
            .query_strategies
            .contains(&QueryStrategy::Structured),
        "structured-only record-shape recall should preserve structured provenance"
    );
}

#[test]
fn lexical_search_preserves_source_metadata_for_mixed_recall() {
    let path = fresh_db_path("mixed-source-metadata");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-source-metadata".to_string(),
            source_label: Some("mixed source metadata memo".to_string()),
            source_kind: Some(agent_memos::memory::record::SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T13:10:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("mixed source-metadata ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline"))
        .expect("mixed lexical + structured search should succeed");

    assert_eq!(results.results.len(), 1);
    assert_eq!(
        results.results[0].record.source.kind,
        agent_memos::memory::record::SourceKind::Note
    );
    assert_eq!(
        results.results[0].record.source.label.as_deref(),
        Some("mixed source metadata memo")
    );
    assert_eq!(
        results.results[0].citation.source_kind,
        agent_memos::memory::record::SourceKind::Note
    );
    assert_eq!(
        results.results[0].citation.source_label.as_deref(),
        Some("mixed source metadata memo")
    );
    assert!(
        results.results[0]
            .trace
            .query_strategies
            .contains(&QueryStrategy::Simple)
            && results.results[0]
                .trace
                .query_strategies
                .contains(&QueryStrategy::Structured),
        "mixed source-metadata recall should preserve both provenance branches"
    );
}

#[test]
fn lexical_search_preserves_record_shape_for_mixed_recall() {
    let path = fresh_db_path("mixed-record-shape");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-record-shape".to_string(),
            source_label: Some("mixed record shape memo".to_string()),
            source_kind: Some(agent_memos::memory::record::SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T13:15:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("mixed record-shape ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline"))
        .expect("mixed lexical + structured search should succeed");

    assert_eq!(results.results.len(), 1);
    assert_eq!(results.results[0].record.scope, Scope::Project);
    assert_eq!(results.results[0].record.truth_layer, TruthLayer::T2);
    assert_eq!(results.results[0].record.record_type, RecordType::Decision);
    assert!(
        results.results[0]
            .trace
            .query_strategies
            .contains(&QueryStrategy::Simple)
            && results.results[0]
                .trace
                .query_strategies
                .contains(&QueryStrategy::Structured),
        "mixed record-shape recall should preserve both provenance branches"
    );
}

#[test]
fn lexical_search_preserves_source_uri_for_mixed_recall() {
    let path = fresh_db_path("mixed-source-uri");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-source-uri".to_string(),
            source_label: Some("mixed source uri memo".to_string()),
            source_kind: Some(agent_memos::memory::record::SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T13:20:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("mixed source-uri ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline"))
        .expect("mixed lexical + structured search should succeed");

    assert_eq!(results.results.len(), 1);
    assert_eq!(
        results.results[0].record.source.uri,
        "memo://project/mixed-source-uri"
    );
    assert_eq!(
        results.results[0].citation.source_uri,
        "memo://project/mixed-source-uri"
    );
    assert!(
        results.results[0]
            .trace
            .query_strategies
            .contains(&QueryStrategy::Simple)
            && results.results[0]
                .trace
                .query_strategies
                .contains(&QueryStrategy::Structured),
        "mixed source-uri recall should preserve both provenance branches"
    );
}

#[test]
fn lexical_search_preserves_matched_query_for_mixed_recall() {
    let path = fresh_db_path("mixed-matched-query");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-matched-query".to_string(),
            source_label: Some("mixed matched query memo".to_string()),
            source_kind: Some(agent_memos::memory::record::SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T13:25:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("mixed matched-query ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline"))
        .expect("mixed lexical + structured search should succeed");

    assert_eq!(results.results.len(), 1);
    assert_eq!(
        results.results[0].trace.matched_query,
        "lexical-first baseline"
    );
    assert!(
        results.results[0]
            .trace
            .query_strategies
            .contains(&QueryStrategy::Simple)
            && results.results[0]
                .trace
                .query_strategies
                .contains(&QueryStrategy::Structured),
        "mixed matched-query recall should preserve both provenance branches"
    );
}

#[test]
fn lexical_search_preserves_record_id_for_mixed_recall() {
    let path = fresh_db_path("mixed-record-id");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-record-id".to_string(),
            source_label: Some("mixed record id memo".to_string()),
            source_kind: Some(agent_memos::memory::record::SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T13:27:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("mixed record-id ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline"))
        .expect("mixed lexical + structured search should succeed");

    assert_eq!(results.results.len(), 1);
    assert_eq!(results.results[0].citation.record_id, results.results[0].record.id);
    assert!(
        results.results[0]
            .trace
            .query_strategies
            .contains(&QueryStrategy::Simple)
            && results.results[0]
                .trace
                .query_strategies
                .contains(&QueryStrategy::Structured),
        "mixed record-id recall should preserve both provenance branches"
    );
}

#[test]
fn lexical_search_preserves_recorded_at_for_mixed_recall() {
    let path = fresh_db_path("mixed-recorded-at");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-recorded-at".to_string(),
            source_label: Some("mixed recorded-at memo".to_string()),
            source_kind: Some(agent_memos::memory::record::SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T13:30:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("mixed recorded-at ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline"))
        .expect("mixed lexical + structured search should succeed");

    assert_eq!(results.results.len(), 1);
    assert_eq!(
        results.results[0].citation.recorded_at,
        "2026-04-15T13:30:00Z"
    );
    assert_eq!(
        results.results[0].record.timestamp.recorded_at,
        "2026-04-15T13:30:00Z"
    );
    assert_eq!(
        results.results[0].record.timestamp.created_at,
        "2026-04-15T13:30:00Z"
    );
    assert_eq!(
        results.results[0].record.timestamp.updated_at,
        "2026-04-15T13:30:00Z"
    );
    assert!(
        results.results[0]
            .trace
            .query_strategies
            .contains(&QueryStrategy::Simple)
            && results.results[0]
                .trace
                .query_strategies
                .contains(&QueryStrategy::Structured),
        "mixed recorded-at recall should preserve both provenance branches"
    );
}

#[test]
fn lexical_search_preserves_chunk_anchor_for_mixed_recall() {
    let path = fresh_db_path("mixed-chunk-anchor");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-chunk-anchor".to_string(),
            source_label: Some("mixed chunk-anchor memo".to_string()),
            source_kind: Some(agent_memos::memory::record::SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T13:35:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("mixed chunk-anchor ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline"))
        .expect("mixed lexical + structured search should succeed");

    assert_eq!(results.results.len(), 1);
    assert_eq!(results.results[0].citation.anchor.chunk_index, 0);
    assert_eq!(results.results[0].citation.anchor.chunk_count, 1);
    assert!(
        results.results[0]
            .trace
            .query_strategies
            .contains(&QueryStrategy::Simple)
            && results.results[0]
                .trace
                .query_strategies
                .contains(&QueryStrategy::Structured),
        "mixed chunk-anchor recall should preserve both provenance branches"
    );
}

#[test]
fn lexical_search_preserves_line_range_anchor_for_mixed_recall() {
    let path = fresh_db_path("mixed-line-range-anchor");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-line-range-anchor".to_string(),
            source_label: Some("mixed line-range anchor memo".to_string()),
            source_kind: Some(agent_memos::memory::record::SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-15T13:37:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("mixed line-range anchor ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline"))
        .expect("mixed lexical + structured search should succeed");

    assert_eq!(results.results.len(), 1);
    assert!(matches!(
        results.results[0].citation.anchor.anchor,
        agent_memos::memory::record::ChunkAnchor::LineRange {
            start_line: 1,
            end_line: 1
        }
    ));
    assert!(
        results.results[0]
            .trace
            .query_strategies
            .contains(&QueryStrategy::Simple)
            && results.results[0]
                .trace
                .query_strategies
                .contains(&QueryStrategy::Structured),
        "mixed line-range anchor recall should preserve both provenance branches"
    );
}
