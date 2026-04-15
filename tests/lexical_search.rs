use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    core::db::Database,
    ingest::{IngestRequest, IngestService},
    memory::record::{RecordType, Scope, TruthLayer},
    search::{QueryStrategy, SearchRequest, SearchService},
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
        !chinese_results.is_empty(),
        "chinese query should recall lexical candidate"
    );
    assert_eq!(
        chinese_results[0].record.source.uri,
        "memo://project/briefing",
        "chinese query should recall the authority-backed chunk"
    );
    assert!(
        chinese_results[0]
            .query_strategies
            .contains(&QueryStrategy::Jieba),
        "chinese query should record jieba-based matching"
    );
    assert!(
        chinese_results[0].snippet.contains("lexical")
            || chinese_results[0].snippet.contains("检索"),
        "snippet should explain the match: {:?}",
        chinese_results[0].snippet
    );

    let pinyin_results = search
        .search(&SearchRequest::new("beiwanglu"))
        .expect("pinyin lexical search should succeed");
    assert!(
        !pinyin_results.is_empty(),
        "pinyin query should recall indexed chinese content"
    );
    assert_eq!(pinyin_results[0].record.source.uri, "memo://project/briefing");
    assert!(
        pinyin_results[0]
            .query_strategies
            .contains(&QueryStrategy::Simple),
        "pinyin query should record simple-query matching"
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

    assert_eq!(results.len(), 2, "both comparable candidates should be recalled");
    assert_eq!(
        results[0].record.source.uri,
        "memo://project/search-decision",
        "importance and keyword bonus should perturb the final ranking deterministically"
    );
    assert!(
        results[0].score.lexical_base >= results[0].score.keyword_bonus,
        "lexical score should remain the dominant term: {:?}",
        results[0].score
    );
    assert!(
        results[0].score.keyword_bonus > results[1].score.keyword_bonus,
        "the decision memo should get the stronger keyword overlap bonus"
    );
    assert!(
        results[0].score.importance_bonus > results[1].score.importance_bonus,
        "record-type defaults should contribute a deterministic importance bonus"
    );
    assert_eq!(
        results[0].score.emotion_bonus, 0.0,
        "emotion defaults should stay explicit when no signal exists"
    );
    assert!(
        results[0].score.final_score > results[1].score.final_score,
        "bonus composition should produce a stable final ranking"
    );
}
