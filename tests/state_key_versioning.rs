//! 按来源 URI 的键值状态检索与版本化更新（`latest_valid_record_by_source` /
//! `close_valid_records_by_source`）：调度器状态类调用方（编排骨架
//! `plan-skeleton:<family>`、族缓存 `plan-family:<goal>`）的唯一键语义——
//! upsert = 关旧版 + 插新版（不物理删，历史可追溯），lookup 只看最新有效版。

use std::path::PathBuf;

use agent_memos::{
    core::db::Database,
    memory::{
        record::{
            MemoryRecord, Provenance, RecordTimestamp, RecordType, Scope, SourceKind, SourceRef,
            TruthLayer, ValidityWindow,
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
        .join("agent-memos-state-key-tests")
        .join(format!("{name}-{unique}"))
        .join("memory.sqlite")
}

/// 最小状态记录：单条、无 chunk、有效期开放（valid_from=now，valid_to=NULL）。
fn state_record(uri: &str, content: &str, at: &str) -> MemoryRecord {
    MemoryRecord {
        id: format!("state-{uri}-{at}"),
        source: SourceRef {
            uri: uri.to_string(),
            kind: SourceKind::System,
            label: None,
        },
        timestamp: RecordTimestamp {
            recorded_at: at.to_string(),
            created_at: at.to_string(),
            updated_at: at.to_string(),
        },
        scope: Scope::Project,
        record_type: RecordType::Decision,
        truth_layer: TruthLayer::T2,
        provenance: Provenance {
            origin: "state".to_string(),
            imported_via: None,
            derived_from: Vec::new(),
        },
        content_text: content.to_string(),
        chunk: None,
        validity: ValidityWindow {
            valid_from: Some(at.to_string()),
            valid_to: None,
        },
    }
}

#[test]
fn latest_valid_record_roundtrip_and_versioning() {
    let path = fresh_db_path("versioning");
    std::fs::create_dir_all(path.parent().expect("parent dir")).expect("create temp dir");
    let db = Database::open(&path).expect("open db");
    let repo = MemoryRepository::new(db.conn());

    // 无记录 → None（未命中语义）
    assert!(
        repo.latest_valid_record_by_source("plan-skeleton:naming-transform")
            .expect("query")
            .is_none()
    );

    // v1 写入 → lookup 取到 v1
    repo.insert_record(&state_record(
        "plan-skeleton:naming-transform",
        r#"{"wins":1}"#,
        "2026-11-12T10:00:00Z",
    ))
    .expect("insert v1");
    let got = repo
        .latest_valid_record_by_source("plan-skeleton:naming-transform")
        .expect("query")
        .expect("v1 valid");
    assert_eq!(got.content_text, r#"{"wins":1}"#);

    // upsert：关旧版 + 插新版 → lookup 取新，旧版不再命中
    let closed = repo
        .close_valid_records_by_source("plan-skeleton:naming-transform", "2026-11-12T11:00:00Z")
        .expect("close v1");
    assert_eq!(closed, 1, "恰关闭一条有效记录");
    repo.insert_record(&state_record(
        "plan-skeleton:naming-transform",
        r#"{"wins":2}"#,
        "2026-11-12T11:00:00Z",
    ))
    .expect("insert v2");
    let got = repo
        .latest_valid_record_by_source("plan-skeleton:naming-transform")
        .expect("query")
        .expect("v2 valid");
    assert_eq!(got.content_text, r#"{"wins":2}"#);

    // 删除语义：关当前有效版 → 无有效记录（未命中），历史仍在库
    let closed = repo
        .close_valid_records_by_source("plan-skeleton:naming-transform", "2026-11-12T12:00:00Z")
        .expect("close v2");
    assert_eq!(closed, 1);
    assert!(
        repo.latest_valid_record_by_source("plan-skeleton:naming-transform")
            .expect("query")
            .is_none(),
        "关闭后不再命中"
    );
    // 已关闭的旧版再关一次 = 无操作（幂等）
    let closed_again = repo
        .close_valid_records_by_source("plan-skeleton:naming-transform", "2026-11-12T13:00:00Z")
        .expect("close again");
    assert_eq!(closed_again, 0, "无有效版本可关");

    // 键隔离：其它来源 URI 不受影响
    repo.insert_record(&state_record(
        "plan-skeleton:visual-draw",
        "B",
        "2026-11-12T10:00:00Z",
    ))
    .expect("insert other key");
    assert!(
        repo.latest_valid_record_by_source("plan-skeleton:visual-draw")
            .expect("query")
            .is_some(),
        "别的族键照常命中"
    );
    assert!(
        repo.latest_valid_record_by_source("plan-skeleton:naming-transform")
            .expect("query")
            .is_none()
    );

    let _ = std::fs::remove_dir_all(path.parent().expect("parent dir"));
}
