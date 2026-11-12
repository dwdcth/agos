//! 世界模型时间演化查询（0415-世界模型.md §8.2）集成测试：
//! `facts_at` 时间窗语义、`fact_timeline` 版本时间线、agent-search 装配的
//! 演化查询门控（默认关）。

use std::path::PathBuf;

use agent_memos::{
    cognition::{
        assembly::{MinimalSelfStateProvider, WorkingMemoryAssembler, WorkingMemoryRequest},
        world_model::{WorldFactQuery, fact_timeline, facts_at},
    },
    core::db::Database,
    memory::{
        dsl::FlatFactDslRecordV1,
        record::{
            ChunkAnchor, ChunkMetadata, MemoryRecord, Provenance, RecordTimestamp, RecordType,
            Scope, SourceKind, SourceRef, TruthLayer, ValidityWindow,
        },
        repository::MemoryRepository,
        store::{FactDslStore, PersistedFactDslRecordV1},
    },
    search::SearchFilters,
};

fn fresh_db_path(name: &str) -> PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-evolution-tests")
        .join(format!("{name}-{unique}"))
        .join("memory.sqlite")
}

fn world_fact_record(
    id: &str,
    claim: &str,
    valid_from: &str,
    valid_to: Option<&str>,
    recorded_at: &str,
) -> MemoryRecord {
    MemoryRecord {
        id: id.to_string(),
        source: SourceRef {
            uri: format!("memo://world/{id}"),
            kind: SourceKind::Note,
            label: None,
        },
        timestamp: RecordTimestamp {
            recorded_at: recorded_at.to_string(),
            created_at: recorded_at.to_string(),
            updated_at: recorded_at.to_string(),
        },
        scope: Scope::Project,
        record_type: RecordType::Fact,
        truth_layer: TruthLayer::T2,
        provenance: Provenance {
            origin: "test".to_string(),
            imported_via: None,
            derived_from: Vec::new(),
        },
        content_text: format!("world fact: {claim}"),
        chunk: Some(ChunkMetadata {
            chunk_index: 0,
            chunk_count: 1,
            anchor: ChunkAnchor::LineRange {
                start_line: 1,
                end_line: 3,
            },
            content_hash: format!("hash-{id}"),
        }),
        validity: ValidityWindow {
            valid_from: Some(valid_from.to_string()),
            valid_to: valid_to.map(ToString::to_string),
        },
    }
}

fn world_fact_dsl(record: &MemoryRecord, claim: &str) -> PersistedFactDslRecordV1 {
    PersistedFactDslRecordV1 {
        record_id: record.id.clone(),
        payload: FlatFactDslRecordV1 {
            domain: "project".to_string(),
            topic: "retrieval".to_string(),
            aspect: "state".to_string(),
            kind: "fact".to_string(),
            claim: claim.to_string(),
            truth_layer: "t2".to_string(),
            source_ref: record.source.uri.clone(),
            why: None,
            time: None,
            cond: None,
            impact: None,
            conf: Some(0.9),
            rel: None,
        },
        classification_confidence: Some(0.9),
        needs_review: false,
    }
}

/// 同一世界主题的两个版本：v1（2026-01-01 ~ 2026-06-01 关闭）、
/// v2（2026-06-01 起开放）。
fn seeded_repository<'db>(conn: &'db rusqlite::Connection) -> MemoryRepository<'db> {
    let repository = MemoryRepository::new(conn);
    let v1 = world_fact_record(
        "retrieval-v1",
        "lexical baseline active",
        "2026-01-01T00:00:00Z",
        Some("2026-06-01T00:00:00Z"),
        "2026-01-01T00:00:00Z",
    );
    let v2 = world_fact_record(
        "retrieval-v2",
        "hybrid retrieval active",
        "2026-06-01T00:00:00Z",
        None,
        "2026-06-01T00:00:00Z",
    );
    repository.insert_record(&v1).expect("insert v1");
    repository.insert_record(&v2).expect("insert v2");
    repository
        .put_fact_dsl(&world_fact_dsl(&v1, "lexical baseline active"))
        .expect("dsl v1");
    repository
        .put_fact_dsl(&world_fact_dsl(&v2, "hybrid retrieval active"))
        .expect("dsl v2");
    repository
}

#[test]
fn facts_at_returns_time_window_view() {
    let path = fresh_db_path("facts-at");
    std::fs::create_dir_all(path.parent().expect("parent dir")).expect("create temp dir");
    let db = Database::open(&path).expect("open db");
    let repository = seeded_repository(db.conn());

    // v1 有效期内
    let march = facts_at(
        &repository,
        &WorldFactQuery::new()
            .with_domain("project")
            .with_topic("retrieval")
            .with_as_of("2026-03-01T00:00:00Z"),
    )
    .expect("facts at march");
    assert_eq!(march.len(), 1);
    assert_eq!(march[0].record.id, "retrieval-v1");
    assert_eq!(march[0].claim(), "lexical baseline active");
    assert_eq!(march[0].truth_layer(), TruthLayer::T2);

    // v1 关闭、v2 开放后
    let november = facts_at(
        &repository,
        &WorldFactQuery::new().with_as_of("2026-11-01T00:00:00Z"),
    )
    .expect("facts at november");
    assert_eq!(november.len(), 1);
    assert_eq!(november[0].record.id, "retrieval-v2");

    // 无 as_of：当前开放版本
    let current = facts_at(&repository, &WorldFactQuery::new()).expect("current facts");
    assert_eq!(current.len(), 1);
    assert_eq!(current[0].record.id, "retrieval-v2");

    // domain/topic 过滤：别的主题不进视图
    let other = facts_at(&repository, &WorldFactQuery::new().with_domain("external"))
        .expect("domain filtered");
    assert!(other.is_empty());

    let _ = std::fs::remove_dir_all(path.parent().expect("parent dir"));
}

#[test]
fn fact_timeline_lists_versions_in_order() {
    let path = fresh_db_path("timeline");
    std::fs::create_dir_all(path.parent().expect("parent dir")).expect("create temp dir");
    let db = Database::open(&path).expect("open db");
    let repository = seeded_repository(db.conn());

    let timeline = fact_timeline(&repository, &WorldFactQuery::new()).expect("timeline");
    assert_eq!(timeline.len(), 2, "历史版本也在时间线上");
    // 同 claim 分组内按 valid_from 升序：v1 在前
    assert_eq!(timeline[0].record_id, "retrieval-v1");
    assert!(!timeline[0].open, "v1 已被 v2 取代");
    assert_eq!(timeline[1].record_id, "retrieval-v2");
    assert!(timeline[1].open);
    assert_eq!(
        timeline[0].valid_to.as_deref(),
        Some("2026-06-01T00:00:00Z")
    );
    assert_eq!(
        timeline[1].valid_from.as_deref(),
        Some("2026-06-01T00:00:00Z")
    );

    let _ = std::fs::remove_dir_all(path.parent().expect("parent dir"));
}

#[test]
fn world_evolution_query_gate_augments_assembly() {
    let path = fresh_db_path("assembly-gate");
    std::fs::create_dir_all(path.parent().expect("parent dir")).expect("create temp dir");
    let db = Database::open(&path).expect("open db");
    seeded_repository(db.conn());

    let filters = SearchFilters {
        domain: Some("project".to_string()),
        topic: Some("retrieval".to_string()),
        valid_at: Some("2026-03-01T00:00:00Z".to_string()),
        ..Default::default()
    };
    // 查询词不命中任何 FTS 词条：词法主路径为空，只能靠演化查询补片段
    let request = WorkingMemoryRequest::new("zzzqqq wxyz").with_filters(filters.clone());

    // 默认关：世界片段为空
    let off = WorkingMemoryAssembler::new(db.conn(), MinimalSelfStateProvider)
        .assemble(&request)
        .expect("assemble off");
    assert!(
        off.present.world_fragments.is_empty(),
        "演化查询关闭时词法无命中 → 无世界片段"
    );

    // 开启：facts_at(as_of=valid_at) 的 v1 事实并入世界片段
    let on = WorkingMemoryAssembler::new(db.conn(), MinimalSelfStateProvider)
        .with_world_evolution_query(true)
        .assemble(&request)
        .expect("assemble on");
    assert_eq!(on.present.world_fragments.len(), 1);
    assert_eq!(on.present.world_fragments[0].record_id, "retrieval-v1");
    assert!(
        on.present.world_fragments[0]
            .dsl
            .as_ref()
            .is_some_and(|dsl| dsl.claim == "lexical baseline active"),
        "DSL 事实视图随片段装配"
    );

    let _ = std::fs::remove_dir_all(path.parent().expect("parent dir"));
}
