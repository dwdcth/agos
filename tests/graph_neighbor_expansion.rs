//! 检索融合（图关系层邻居扩展）集成测试：默认关、builder 开、config 开。

use std::path::PathBuf;

use agent_memos::{
    core::config::{Config, GraphConfig},
    core::db::Database,
    ingest::{IngestRequest, IngestService},
    memory::links::{MemoryLink, MemoryLinkStore, RELATION_RELATES_TO},
    memory::record::{RecordType, Scope, TruthLayer},
    search::{QueryStrategy, SearchRequest, SearchService},
};

fn fresh_db_path(name: &str) -> PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-graph-search-tests")
        .join(format!("{name}-{unique}"))
        .join("memory.sqlite")
}

fn ingest(conn: &rusqlite::Connection, uri: &str, content: &str) -> String {
    IngestService::new(conn)
        .ingest(IngestRequest {
            source_uri: uri.to_string(),
            source_label: None,
            source_kind: None,
            content: content.to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-11-12T10:00:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest")
        .record_ids[0]
        .clone()
}

#[test]
fn neighbor_expansion_default_off_builder_on() {
    let path = fresh_db_path("expansion");
    std::fs::create_dir_all(path.parent().expect("parent dir")).expect("create temp dir");
    let db = Database::open(&path).expect("open db");

    let hit_id = ingest(
        db.conn(),
        "memo://k8s/incident",
        "kubernetes cluster rollout incident pager duty",
    );
    let neighbor_id = ingest(
        db.conn(),
        "memo://cooking/dinner",
        "completely unrelated note about pasta dinner plans",
    );
    // 命中记录 ↔ 无关记录的手工关联边（图关系层存在性）
    MemoryLinkStore::new(db.conn())
        .put_link(&MemoryLink::new(
            &hit_id,
            &neighbor_id,
            RELATION_RELATES_TO,
            0.9,
            "manual",
            "2026-11-12T10:00:00Z",
        ))
        .expect("link");

    let request = SearchRequest::new("kubernetes rollout incident");

    // 默认关：只有词法命中
    let plain = SearchService::new(db.conn())
        .search(&request)
        .expect("plain search");
    assert_eq!(plain.results.len(), 1);
    assert_eq!(plain.results[0].record.id, hit_id);

    // builder 开启：一跳邻居追加为 supporting 结果，trace 标 graph_neighbor
    let expanded = SearchService::new(db.conn())
        .with_link_expansion(Some(agent_memos::search::LinkExpansion::new(2)))
        .search(&request)
        .expect("expanded search");
    assert_eq!(expanded.results.len(), 2, "邻居追加在主结果之后");
    assert_eq!(expanded.results[0].record.id, hit_id);
    let neighbor_result = &expanded.results[1];
    assert_eq!(neighbor_result.record.id, neighbor_id);
    assert_eq!(
        neighbor_result.trace.query_strategies,
        vec![QueryStrategy::GraphNeighbor]
    );
    assert_eq!(
        neighbor_result.trace.matched_query,
        format!("link-neighbor:{hit_id}")
    );
    assert!(
        neighbor_result.score.final_score < expanded.results[0].score.final_score,
        "邻居分数相对父结果衰减"
    );

    let _ = std::fs::remove_dir_all(path.parent().expect("parent dir"));
}

#[test]
fn neighbor_expansion_respects_config_gate() {
    let path = fresh_db_path("config-gate");
    std::fs::create_dir_all(path.parent().expect("parent dir")).expect("create temp dir");
    let db = Database::open(&path).expect("open db");

    let hit_id = ingest(
        db.conn(),
        "memo://k8s/incident",
        "kubernetes cluster rollout incident pager duty",
    );
    let neighbor_id = ingest(
        db.conn(),
        "memo://k8s/related",
        "postmortem follow up action items quarterly review",
    );
    MemoryLinkStore::new(db.conn())
        .put_link(&MemoryLink::new(
            &hit_id,
            &neighbor_id,
            RELATION_RELATES_TO,
            0.9,
            "manual",
            "2026-11-12T10:00:00Z",
        ))
        .expect("link");

    let request = SearchRequest::new("kubernetes rollout incident");

    // config 默认（[graph] 未配置）→ 关
    let default_config = Config::default();
    assert!(!default_config.graph.neighbor_expansion);
    let plain = SearchService::with_runtime_config(db.conn(), &default_config, None)
        .search(&request)
        .expect("default config search");
    assert_eq!(plain.results.len(), 1);

    // config 显式开启 → 扩
    let graph_config = Config {
        graph: GraphConfig {
            neighbor_expansion: true,
            neighbor_per_result: 1,
        },
        ..Default::default()
    };
    let expanded = SearchService::with_runtime_config(db.conn(), &graph_config, None)
        .search(&request)
        .expect("graph config search");
    assert_eq!(expanded.results.len(), 2);
    assert_eq!(expanded.results[1].record.id, neighbor_id);

    let _ = std::fs::remove_dir_all(path.parent().expect("parent dir"));
}
