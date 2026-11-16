use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    cognition::self_model::{
        DEFAULT_USER_SUBJECT, compact_self_model_subject, list_active_subject_entries,
        remove_subject_entry, upsert_subject_entry,
    },
    core::{
        config::{
            EmbeddingBackend, EmbeddingConfig, RetrievalMode, RetrievalModeVariant,
            RootEmbeddingRuntimeConfig, RootLlmConfig,
        },
        db::Database,
    },
    ingest::{IngestRequest, IngestService},
    memory::{
        links::{LinkDirection, LinkQuery, MemoryLink, MemoryLinkStore, RELATION_RELATES_TO, RELATION_SUPERSEDES},
        record::{RecordType, Scope, TruthLayer},
        repository::{LocalAdaptationTargetKind, MemoryRepository},
    },
    search::{SearchFilters, SearchRequest, SearchService},
};

fn fresh_db_dir(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir()
        .join(format!("agos-m2-{name}-{}-{unique}", std::process::id()))
}

#[test]
fn only_open_filters_closed_and_superseded_records_across_lexical_and_embedding() {
    let dir = fresh_db_dir("only-open");
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let db = Database::open(&dir.join("memory.sqlite")).expect("open db");

    let emb_cfg = EmbeddingConfig {
        backend: EmbeddingBackend::Builtin,
        model: Some("builtin-hash-v1".to_string()),
        endpoint: None,
        api_key: None,
        ..Default::default()
    };
    let ingest = IngestService::with_embedding_config(db.conn(), Default::default(), emb_cfg.clone());

    let r_old = ingest
        .ingest(IngestRequest {
            source_uri: "chat://director/old-npm".into(),
            source_label: None,
            source_kind: None,
            content: "前端构建统一使用 npm run build 命令。".into(),
            scope: Scope::Project,
            record_type: RecordType::Fact,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-11-16T10:00:00Z".into(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest old");
    let r_other = ingest
        .ingest(IngestRequest {
            source_uri: "chat://director/peer".into(),
            source_label: None,
            source_kind: None,
            content: "仓库结构说明文档。".into(),
            scope: Scope::Project,
            record_type: RecordType::Fact,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-11-16T10:01:00Z".into(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest peer");
    let r_new = ingest
        .ingest(IngestRequest {
            source_uri: "chat://director/new-pnpm".into(),
            source_label: None,
            source_kind: None,
            content: "纠正：前端构建统一改用 pnpm build，不用 npm。".into(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-11-16T10:05:00Z".into(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest new");

    let old_id = r_old.record_ids[0].clone();
    let peer_id = r_other.record_ids[0].clone();
    let new_id = r_new.record_ids[0].clone();

    // Seed an open link touching old_id to verify supersede_records closes it.
    let link_store = MemoryLinkStore::new(db.conn());
    link_store
        .put_link(&MemoryLink::new(
            &old_id,
            &peer_id,
            RELATION_RELATES_TO,
            0.8,
            "rumination",
            "2026-11-16T10:02:00Z",
        ))
        .expect("put link");

    let repo = MemoryRepository::new(db.conn());
    let closed = repo
        .supersede_records(
            std::slice::from_ref(&old_id),
            &new_id,
            "2026-11-16T10:05:00Z",
            "user_correction",
        )
        .expect("supersede");
    assert_eq!(closed, 1);

    // Idempotent second supersede returns 0 newly closed.
    let closed_again = repo
        .supersede_records(
            std::slice::from_ref(&old_id),
            &new_id,
            "2026-11-16T10:06:00Z",
            "user_correction",
        )
        .expect("supersede idempotent");
    assert_eq!(closed_again, 0);

    // Check directed supersedes link & closed relates_to link.
    let outgoing = link_store
        .neighbors(
            &new_id,
            1,
            &LinkQuery::default()
                .with_direction(LinkDirection::Outgoing)
                .with_relation(RELATION_SUPERSEDES),
        )
        .expect("supersedes neighbor");
    assert_eq!(outgoing.len(), 1);
    assert_eq!(outgoing[0].record.id, old_id);
    let old_relates = link_store
        .neighbors(
            &old_id,
            1,
            &LinkQuery::default().with_relation(RELATION_RELATES_TO),
        )
        .expect("open relates_to");
    assert!(old_relates.is_empty(), "open link touching superseded record must be closed");

    // 1. Lexical search: only_open=true hides old_id; only_open=false still retrieves old_id.
    let lexical = SearchService::new(db.conn());
    let res_open = lexical
        .search(
            &SearchRequest::new("前端 构建 npm pnpm").with_filters(SearchFilters {
                only_open: true,
                ..Default::default()
            }),
        )
        .expect("lexical open");
    assert!(res_open.results.iter().all(|r| r.record.id != old_id));
    assert!(res_open.results.iter().any(|r| r.record.id == new_id));

    let res_all = lexical
        .search(&SearchRequest::new("前端 构建 npm"))
        .expect("lexical all");
    assert!(res_all.results.iter().any(|r| r.record.id == old_id));

    // 2. Hybrid embedding search: only_open=true hides old_id from embedding leg too.
    let variant = RetrievalModeVariant {
        name: "test-hybrid".into(),
        db_path: dir.join("memory.sqlite").to_string_lossy().to_string(),
        mode: RetrievalMode::Hybrid,
        embedding_backend: EmbeddingBackend::Builtin,
        llm: RootLlmConfig::default(),
        embedding: Some(RootEmbeddingRuntimeConfig {
            provider: "builtin".into(),
            model: "builtin-hash-v1".into(),
            api_base: None,
            api_key: None,
            dimensions: None,
            batch_size: None,
            timeout_seconds: None,
        }),
        vector: None,
    };
    let hybrid = SearchService::with_variant(db.conn(), &variant);
    let h_open = hybrid
        .search(
            &SearchRequest::new("前端构建统一使用 npm run build 命令").with_filters(SearchFilters {
                only_open: true,
                ..Default::default()
            }),
        )
        .expect("hybrid open");
    assert!(h_open.results.iter().all(|r| r.record.id != old_id));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn subject_profile_helpers_upsert_remove_revive_and_compact() {
    let dir = fresh_db_dir("profile");
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let db = Database::open(&dir.join("memory.sqlite")).expect("open db");
    let repo = MemoryRepository::new(db.conn());

    let ts = "2026-11-16T12:00:00Z";
    upsert_subject_entry(
        &repo,
        DEFAULT_USER_SUBJECT,
        "stack.pkg_mgr",
        "pnpm",
        vec!["chat://1".into()],
        ts,
    )
    .expect("upsert stack");
    upsert_subject_entry(
        &repo,
        DEFAULT_USER_SUBJECT,
        "taboo.force_push",
        "禁止对 main 分支强推",
        vec![],
        ts,
    )
    .expect("upsert taboo");

    let active = list_active_subject_entries(&repo, DEFAULT_USER_SUBJECT).expect("list");
    assert_eq!(active.len(), 2);
    assert_eq!(active[0].key, "stack.pkg_mgr");
    assert_eq!(active[0].value, "pnpm");
    assert_eq!(active[0].target_kind, LocalAdaptationTargetKind::SelfState);
    assert_eq!(active[1].key, "taboo.force_push");
    assert_eq!(active[1].target_kind, LocalAdaptationTargetKind::RiskBoundary);

    // Same timestamp remove must deactivate the key via monotonic entry_id ordering.
    assert!(remove_subject_entry(&repo, DEFAULT_USER_SUBJECT, "stack.pkg_mgr", ts).expect("remove"));
    assert!(!remove_subject_entry(&repo, DEFAULT_USER_SUBJECT, "stack.pkg_mgr", ts).expect("remove missing"));
    let after_rm = list_active_subject_entries(&repo, DEFAULT_USER_SUBJECT).expect("list after rm");
    assert_eq!(after_rm.len(), 1);
    assert_eq!(after_rm[0].key, "taboo.force_push");

    // Compact to snapshot, then revive removed key at same timestamp.
    compact_self_model_subject(&repo, DEFAULT_USER_SUBJECT, "snap-1", ts).expect("compact");
    upsert_subject_entry(&repo, DEFAULT_USER_SUBJECT, "stack.pkg_mgr", "uv", vec![], ts)
        .expect("revive");
    let revived = list_active_subject_entries(&repo, DEFAULT_USER_SUBJECT).expect("list revived");
    assert_eq!(revived.len(), 2);
    assert_eq!(
        revived.iter().find(|e| e.key == "stack.pkg_mgr").unwrap().value,
        "uv"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
