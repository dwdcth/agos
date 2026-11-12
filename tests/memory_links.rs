//! 图关系层（memory_links）集成测试：边往返、邻居 BFS、时间窗/衰减
//! 语义、provenance 回填与 BM25 互相命中发现。

use std::path::PathBuf;

use agent_memos::{
    cognition::rumination::{MaintenancePassOptions, run_maintenance_pass_with_options},
    core::db::Database,
    memory::{
        links::{
            LinkDirection, LinkDiscoveryOptions, LinkQuery, MemoryLink, MemoryLinkStore,
            PROVENANCE_PROVENANCE_BACKFILL, PROVENANCE_RUMINATION, RELATION_DERIVED_FROM,
            RELATION_RELATES_TO,
        },
        record::{
            ChunkAnchor, ChunkMetadata, MemoryRecord, Provenance, RecordTimestamp, RecordType,
            Scope, SourceKind, SourceRef, TruthLayer, ValidityWindow,
        },
        repository::MemoryRepository,
    },
};

fn fresh_db_path(name: &str) -> PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-links-tests")
        .join(format!("{name}-{unique}"))
        .join("memory.sqlite")
}

/// 带 chunk 元数据的记录（chunk 供 Citation 引用；chunk_index 可控以测
/// 回填链顺序）。
fn chunked_record(
    id: &str,
    uri: &str,
    chunk_index: u32,
    content: &str,
    derived_from: &[&str],
    recorded_at: &str,
) -> MemoryRecord {
    MemoryRecord {
        id: id.to_string(),
        source: SourceRef {
            uri: uri.to_string(),
            kind: SourceKind::Note,
            label: None,
        },
        timestamp: RecordTimestamp {
            recorded_at: recorded_at.to_string(),
            created_at: recorded_at.to_string(),
            updated_at: recorded_at.to_string(),
        },
        scope: Scope::Project,
        record_type: RecordType::Observation,
        truth_layer: TruthLayer::T2,
        provenance: Provenance {
            origin: "test".to_string(),
            imported_via: None,
            derived_from: derived_from.iter().map(|value| value.to_string()).collect(),
        },
        content_text: content.to_string(),
        chunk: Some(ChunkMetadata {
            chunk_index,
            chunk_count: 3,
            anchor: ChunkAnchor::LineRange {
                start_line: chunk_index * 10 + 1,
                end_line: chunk_index * 10 + 10,
            },
            content_hash: format!("hash-{id}"),
        }),
        validity: ValidityWindow {
            valid_from: Some(recorded_at.to_string()),
            valid_to: None,
        },
    }
}

fn open_db(name: &str) -> (Database, PathBuf) {
    let path = fresh_db_path(name);
    std::fs::create_dir_all(path.parent().expect("parent dir")).expect("create temp dir");
    let db = Database::open(&path).expect("open db");
    (db, path)
}

#[test]
fn link_roundtrip_upsert_and_direction_filters() {
    let (db, path) = open_db("roundtrip");
    let repo = MemoryRepository::new(db.conn());
    let store = MemoryLinkStore::new(db.conn());

    repo.insert_record(&chunked_record(
        "rec-a",
        "memo://a",
        0,
        "alpha",
        &[],
        "2026-11-12T10:00:00Z",
    ))
    .expect("insert a");
    repo.insert_record(&chunked_record(
        "rec-b",
        "memo://b",
        0,
        "beta",
        &[],
        "2026-11-12T10:00:00Z",
    ))
    .expect("insert b");

    let link = MemoryLink::new(
        "rec-a",
        "rec-b",
        RELATION_DERIVED_FROM,
        0.8,
        "manual",
        "2026-11-12T10:00:00Z",
    );
    store.put_link(&link).expect("put link");

    let fetched = store
        .get_link("rec-a", "rec-b", RELATION_DERIVED_FROM, "manual")
        .expect("get link")
        .expect("link exists");
    assert_eq!(fetched.link_id, link.link_id);
    assert_eq!(fetched.weight, 0.8);
    assert!(fetched.validity.valid_to.is_none(), "新边开放");

    // 同键 upsert：weight 更新，不重复建行
    let updated = MemoryLink::new(
        "rec-a",
        "rec-b",
        RELATION_DERIVED_FROM,
        0.95,
        "manual",
        "2026-11-12T11:00:00Z",
    );
    store.put_link(&updated).expect("upsert link");
    let fetched = store
        .get_link("rec-a", "rec-b", RELATION_DERIVED_FROM, "manual")
        .expect("get link")
        .expect("link exists");
    assert_eq!(fetched.weight, 0.95);

    // 出边/入边方向过滤
    let outgoing = store
        .links_for_record(
            "rec-a",
            &LinkQuery::default().with_direction(LinkDirection::Outgoing),
        )
        .expect("outgoing");
    assert_eq!(outgoing.len(), 1);
    let incoming = store
        .links_for_record(
            "rec-a",
            &LinkQuery::default().with_direction(LinkDirection::Incoming),
        )
        .expect("incoming");
    assert!(incoming.is_empty(), "rec-a 无入边");

    let _ = std::fs::remove_dir_all(path.parent().expect("parent dir"));
}

#[test]
fn symmetric_links_are_canonicalized_for_dedup() {
    let (db, path) = open_db("symmetric");
    let repo = MemoryRepository::new(db.conn());
    let store = MemoryLinkStore::new(db.conn());
    let now = "2026-11-12T10:00:00Z";

    for (id, uri) in [("rec-a", "memo://a"), ("rec-z", "memo://z")] {
        repo.insert_record(&chunked_record(id, uri, 0, id, &[], now))
            .expect("insert record");
    }

    // a→b 与 b→a 的 relates_to 应规范化为同一行
    store
        .put_link(&MemoryLink::new(
            "rec-z",
            "rec-a",
            RELATION_RELATES_TO,
            0.5,
            "manual",
            now,
        ))
        .expect("put z-a");
    store
        .put_link(&MemoryLink::new(
            "rec-a",
            "rec-z",
            RELATION_RELATES_TO,
            0.7,
            "manual",
            "2026-11-12T10:05:00Z",
        ))
        .expect("put a-z");

    let links = store
        .links_for_record("rec-a", &LinkQuery::default())
        .expect("links");
    assert_eq!(links.len(), 1, "对称关系双向写只落一行");
    assert_eq!(links[0].source_record_id, "rec-a", "端点按字典序规范化");

    let _ = std::fs::remove_dir_all(path.parent().expect("parent dir"));
}

#[test]
fn neighbors_bfs_depth_and_relation_filter() {
    let (db, path) = open_db("neighbors");
    let repo = MemoryRepository::new(db.conn());
    let store = MemoryLinkStore::new(db.conn());
    let now = "2026-11-12T10:00:00Z";

    for (id, uri) in [
        ("rec-a", "memo://a"),
        ("rec-b", "memo://b"),
        ("rec-c", "memo://c"),
    ] {
        repo.insert_record(&chunked_record(id, uri, 0, id, &[], now))
            .expect("insert record");
    }
    // a -relates_to- b -relates_to- c 链 + 一条 a -supports- b 方向边
    store
        .put_link(&MemoryLink::new(
            "rec-a",
            "rec-b",
            RELATION_RELATES_TO,
            0.9,
            "manual",
            now,
        ))
        .expect("link ab");
    store
        .put_link(&MemoryLink::new(
            "rec-b",
            "rec-c",
            RELATION_RELATES_TO,
            0.8,
            "manual",
            now,
        ))
        .expect("link bc");
    store
        .put_link(&MemoryLink::new(
            "rec-a", "rec-b", "supports", 0.7, "manual", now,
        ))
        .expect("link ab supports");

    // 一跳：rec-b 是唯一邻居（首达边保留，多条边只记一条路径）
    let one_hop = store
        .neighbors("rec-a", 1, &LinkQuery::default())
        .expect("one hop");
    assert_eq!(one_hop.len(), 1);
    assert_eq!(one_hop[0].record.id, "rec-b");
    assert_eq!(
        one_hop[0].link.relation, RELATION_RELATES_TO,
        "高权重边为首达边"
    );
    assert_eq!(one_hop[0].depth, 1);

    // 两跳 BFS：能到 c
    let two_hop = store
        .neighbors("rec-a", 2, &LinkQuery::default())
        .expect("two hop");
    assert!(
        two_hop
            .iter()
            .any(|neighbor| neighbor.record.id == "rec-c" && neighbor.depth == 2)
    );

    // 关系过滤：只走 relates_to
    let related = store
        .neighbors(
            "rec-a",
            2,
            &LinkQuery::default().with_relation(RELATION_RELATES_TO),
        )
        .expect("relation filtered");
    assert!(
        related
            .iter()
            .all(|neighbor| neighbor.link.relation == RELATION_RELATES_TO)
    );

    let _ = std::fs::remove_dir_all(path.parent().expect("parent dir"));
}

#[test]
fn link_validity_window_and_close_semantics() {
    let (db, path) = open_db("validity");
    let repo = MemoryRepository::new(db.conn());
    let store = MemoryLinkStore::new(db.conn());
    let now = "2026-11-12T10:00:00Z";

    for (id, uri) in [("rec-a", "memo://a"), ("rec-b", "memo://b")] {
        repo.insert_record(&chunked_record(id, uri, 0, id, &[], now))
            .expect("insert record");
    }
    store
        .put_link(&MemoryLink::new(
            "rec-a",
            "rec-b",
            RELATION_RELATES_TO,
            0.9,
            "manual",
            now,
        ))
        .expect("link");

    // 关闭 b 触及的开放边（版本化下线钩子）
    let closed = store
        .close_links_touching_record("rec-b", "2026-11-12T12:00:00Z")
        .expect("close links");
    assert_eq!(closed, 1);

    // 默认查询只看开放边
    let open_links = store
        .links_for_record("rec-b", &LinkQuery::default())
        .expect("open links");
    assert!(open_links.is_empty(), "关闭后不再命中");

    // include_closed 取回历史
    let all_links = store
        .links_for_record("rec-b", &LinkQuery::default().with_include_closed(true))
        .expect("all links");
    assert_eq!(all_links.len(), 1);

    // as_of 时间窗：关闭前包含，关闭后不包含
    let before = store
        .links_for_record(
            "rec-b",
            &LinkQuery::default().with_as_of("2026-11-12T11:00:00Z"),
        )
        .expect("as_of before");
    assert_eq!(before.len(), 1, "关闭时点前该边有效");
    let after = store
        .links_for_record(
            "rec-b",
            &LinkQuery::default().with_as_of("2026-11-12T13:00:00Z"),
        )
        .expect("as_of after");
    assert!(after.is_empty(), "关闭时点后无效");

    // 邻居查询同样尊重开放语义
    let neighbors = store
        .neighbors("rec-a", 1, &LinkQuery::default())
        .expect("neighbors");
    assert!(neighbors.is_empty());

    let _ = std::fs::remove_dir_all(path.parent().expect("parent dir"));
}

#[test]
fn backfill_from_provenance_builds_chain_and_direct_edges() {
    let (db, path) = open_db("backfill");
    let repo = MemoryRepository::new(db.conn());
    let store = MemoryLinkStore::new(db.conn());
    let now = "2026-11-12T10:00:00Z";

    // 同源三块 chunk（ingest 派生锚形态：uri#anchor）
    repo.insert_record(&chunked_record(
        "doc-chunk-0",
        "memo://doc",
        0,
        "first",
        &["memo://doc#line-1-10"],
        now,
    ))
    .expect("insert chunk 0");
    repo.insert_record(&chunked_record(
        "doc-chunk-1",
        "memo://doc",
        1,
        "second",
        &["memo://doc#line-1-10"],
        now,
    ))
    .expect("insert chunk 1");
    repo.insert_record(&chunked_record(
        "doc-chunk-2",
        "memo://doc",
        2,
        "third",
        &["memo://doc#line-1-10"],
        now,
    ))
    .expect("insert chunk 2");
    // 直接引用库内 record id 的派生
    repo.insert_record(&chunked_record(
        "derived-note",
        "memo://note",
        0,
        "note derived from chunk 0",
        &["doc-chunk-0"],
        now,
    ))
    .expect("insert derived note");

    let inserted = store.backfill_from_provenance(now).expect("backfill run 1");
    // 3 块 chunk 相邻链 = 2 条 relates_to + 1 条 direct derived_from
    assert_eq!(inserted, 3);

    let chain = store
        .links_for_record(
            "doc-chunk-0",
            &LinkQuery::default()
                .with_relation(RELATION_RELATES_TO)
                .with_include_closed(true),
        )
        .expect("chain links");
    assert_eq!(chain.len(), 1, "chunk0 只连 chunk1（链不造完全图）");
    assert_eq!(chain[0].provenance, PROVENANCE_PROVENANCE_BACKFILL);
    assert!(
        chain[0].source_record_id == "doc-chunk-0" && chain[0].target_record_id == "doc-chunk-1"
    );

    let direct = store
        .get_link(
            "derived-note",
            "doc-chunk-0",
            RELATION_DERIVED_FROM,
            PROVENANCE_PROVENANCE_BACKFILL,
        )
        .expect("direct link")
        .expect("derived_from edge exists");
    assert_eq!(
        direct.source_record_id, "derived-note",
        "方向保持：派生方为源"
    );

    // 幂等：重跑不再插入
    let again = store
        .backfill_from_provenance("2026-11-12T11:00:00Z")
        .expect("backfill run 2");
    assert_eq!(again, 0, "回填幂等");

    let _ = std::fs::remove_dir_all(path.parent().expect("parent dir"));
}

#[test]
fn close_valid_records_by_source_closes_touching_links() {
    let (db, path) = open_db("decay-hook");
    let repo = MemoryRepository::new(db.conn());
    let store = MemoryLinkStore::new(db.conn());
    let now = "2026-11-12T10:00:00Z";

    // 同一 source_uri 的 v1 与另一条记录相连（版本化 upsert 场景）
    repo.insert_record(&chunked_record(
        "state-v1",
        "plan-skeleton:family",
        0,
        "v1",
        &[],
        now,
    ))
    .expect("insert v1");
    repo.insert_record(&chunked_record(
        "other-rec",
        "memo://other",
        0,
        "other",
        &[],
        now,
    ))
    .expect("insert other");
    store
        .put_link(&MemoryLink::new(
            "state-v1",
            "other-rec",
            RELATION_RELATES_TO,
            0.9,
            "manual",
            now,
        ))
        .expect("link");

    // 版本化 upsert 的关旧版半步 → 链接联动关闭（宿主零改动拿衰减语义）
    let closed = repo
        .close_valid_records_by_source("plan-skeleton:family", "2026-11-12T11:00:00Z")
        .expect("close records");
    assert_eq!(closed, 1);

    let open_links = store
        .links_for_record("state-v1", &LinkQuery::default())
        .expect("open links");
    assert!(open_links.is_empty(), "触及被关记录的边同步关闭");
    let other_side = store
        .links_for_record("other-rec", &LinkQuery::default())
        .expect("other side");
    assert!(other_side.is_empty());

    let _ = std::fs::remove_dir_all(path.parent().expect("parent dir"));
}

#[test]
fn discover_related_links_mutual_bm25_hits() {
    let (db, path) = open_db("discovery");
    let repo = MemoryRepository::new(db.conn());
    let store = MemoryLinkStore::new(db.conn());
    let now = "2026-11-12T10:00:00Z";

    // 两个不同来源、共享独特词表的记录（BM25 互相命中）
    repo.insert_record(&chunked_record(
        "k8s-incident",
        "memo://k8s/incident",
        0,
        "kubernetes cluster rollout pager duty incident quota",
        &[],
        "2026-11-12T10:00:00Z",
    ))
    .expect("insert k8s incident");
    repo.insert_record(&chunked_record(
        "k8s-cost",
        "memo://k8s/cost",
        0,
        "kubernetes cluster vendor quota cost budget rollout",
        &[],
        "2026-11-12T10:01:00Z",
    ))
    .expect("insert k8s cost");
    // 无关词表：不应成边
    repo.insert_record(&chunked_record(
        "pasta-recipe",
        "memo://cooking/pasta",
        0,
        "boil pasta salt water olive oil parmesan minutes",
        &[],
        "2026-11-12T10:02:00Z",
    ))
    .expect("insert pasta");
    // 同源 chunk：词表相似但同 source_uri，必须排除
    repo.insert_record(&chunked_record(
        "same-doc-0",
        "memo://same-doc",
        0,
        "kubernetes cluster rollout quota",
        &[],
        "2026-11-12T10:03:00Z",
    ))
    .expect("insert same doc 0");
    repo.insert_record(&chunked_record(
        "same-doc-1",
        "memo://same-doc",
        1,
        "kubernetes cluster rollout quota second half",
        &[],
        "2026-11-12T10:03:00Z",
    ))
    .expect("insert same doc 1");

    let options = LinkDiscoveryOptions {
        max_records: 32,
        top_k: 4,
        min_score: 0.15,
    };
    let inserted = store.discover_related(now, &options).expect("discover");
    assert!(inserted >= 1, "k8s 两记录应互相命中成边（实际 {inserted}）");

    let k8s_link = store
        .get_link(
            "k8s-cost",
            "k8s-incident",
            RELATION_RELATES_TO,
            PROVENANCE_RUMINATION,
        )
        .expect("get k8s link")
        .expect("k8s relates_to edge exists");
    assert!(k8s_link.weight > 0.0 && k8s_link.weight <= 1.0);

    // pasta 无边；同源 chunk 无边
    let pasta_links = store
        .links_for_record(
            "pasta-recipe",
            &LinkQuery::default().with_include_closed(true),
        )
        .expect("pasta links");
    assert!(pasta_links.is_empty(), "无关词表不成边");
    let same_doc_links = store
        .links_for_record(
            "same-doc-0",
            &LinkQuery::default().with_include_closed(true),
        )
        .expect("same doc links");
    assert!(
        same_doc_links
            .iter()
            .all(|link| link.target_record_id != "same-doc-1"),
        "同源 chunk 排除"
    );

    // 重复发现幂等（开放边去重）
    let again = store
        .discover_related(now, &options)
        .expect("discover again");
    assert_eq!(again, 0, "已有开放 relates_to 不重复建边");

    let _ = std::fs::remove_dir_all(path.parent().expect("parent dir"));
}

#[test]
fn maintenance_pass_reports_link_work() {
    let (db, path) = open_db("maintenance");
    let repo = MemoryRepository::new(db.conn());
    let now = "2026-11-12T10:00:00Z";

    repo.insert_record(&chunked_record(
        "k8s-incident",
        "memo://k8s/incident",
        0,
        "kubernetes cluster rollout pager duty incident quota",
        &["memo://k8s/incident#line-1-10"],
        now,
    ))
    .expect("insert k8s incident");
    repo.insert_record(&chunked_record(
        "k8s-cost",
        "memo://k8s/cost",
        0,
        "kubernetes cluster vendor quota cost budget rollout",
        &[],
        "2026-11-12T10:01:00Z",
    ))
    .expect("insert k8s cost");

    let report = run_maintenance_pass_with_options(
        db.conn(),
        None,
        now,
        &MaintenancePassOptions {
            link_discovery: LinkDiscoveryOptions {
                max_records: 32,
                top_k: 4,
                min_score: 0.15,
            },
        },
    );
    assert_eq!(report.links_backfilled, 0, "无同锚组、无 record-id 引用");
    assert!(report.links_discovered >= 1, "互相命中应入报告");
    assert!(report.errors.is_empty(), "无错误：{report:?}");

    let _ = std::fs::remove_dir_all(path.parent().expect("parent dir"));
}
