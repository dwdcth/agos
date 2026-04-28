use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    core::db::Database,
    memory::{
        dsl::{FactDslDraft, FactDslRecord},
        record::{
            ChunkAnchor, MemoryRecord, Provenance, RecordTimestamp, RecordType, Scope, SourceKind,
            SourceRef, TruthLayer, ValidityWindow,
        },
        repository::{
            LocalAdaptationEntry, LocalAdaptationPayload, LocalAdaptationTargetKind,
            MemoryRepository, PersistedSelfModelSnapshot, PersistedSelfModelSnapshotEntry,
            PersistedWorldModelAppliedFilters, PersistedWorldModelChannelContribution,
            PersistedWorldModelCitation, PersistedWorldModelCitationAnchor,
            PersistedWorldModelQueryStrategy, PersistedWorldModelScore,
            PersistedWorldModelSnapshot, PersistedWorldModelSnapshotFragment,
            PersistedWorldModelTrace, PersistedWorldModelTruthContext, SelfModelGovernanceMetadata,
            SelfModelResolutionState,
        },
        store::{FactDslStore, PersistedFactDslRecordV1},
        taxonomy::{AspectV1, DomainV1, KindV1, TaxonomyPathV1, TopicV1},
    },
};
use serde_json::json;

fn fresh_db_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-memory-store-tests")
        .join(format!("{name}-{unique}"))
        .join("memory.sqlite")
}

fn sample_memory_record() -> MemoryRecord {
    MemoryRecord {
        id: "mem-1".to_string(),
        source: SourceRef {
            uri: "memo://project/retrieval".to_string(),
            kind: SourceKind::Note,
            label: Some("retrieval note".to_string()),
        },
        timestamp: RecordTimestamp {
            recorded_at: "2026-04-19T00:00:00Z".to_string(),
            created_at: "2026-04-19T00:00:00Z".to_string(),
            updated_at: "2026-04-19T00:00:00Z".to_string(),
        },
        scope: Scope::Project,
        record_type: RecordType::Observation,
        truth_layer: TruthLayer::T2,
        provenance: Provenance {
            origin: "test".to_string(),
            imported_via: None,
            derived_from: Vec::new(),
        },
        content_text: "Use lexical-first as baseline because explainability matters.".to_string(),
        chunk: None,
        validity: ValidityWindow::default(),
    }
}

fn sample_dsl_record() -> FactDslRecord {
    FactDslRecord {
        taxonomy: TaxonomyPathV1::new(
            DomainV1::Project,
            TopicV1::Retrieval,
            AspectV1::Behavior,
            KindV1::Decision,
        )
        .expect("taxonomy path should be valid"),
        draft: FactDslDraft {
            claim: "use lexical-first as baseline".to_string(),
            why: Some("explainability matters".to_string()),
            time: Some("2026-04".to_string()),
            ..Default::default()
        },
        truth_layer: TruthLayer::T2,
        source_ref: "roadmap#phase9".to_string(),
    }
}

#[test]
fn sqlite_repository_implements_fact_dsl_store_contract() {
    let path = fresh_db_path("dsl-store");
    let db = Database::open(&path).expect("database should open");
    assert_eq!(db.schema_version().expect("schema version"), 10);

    let repo = MemoryRepository::new(db.conn());
    repo.insert_record(&sample_memory_record())
        .expect("authority record should insert");

    let persisted = PersistedFactDslRecordV1::from_fact_dsl_record("mem-1", &sample_dsl_record())
        .expect("persisted wrapper should build");

    repo.put_fact_dsl(&persisted)
        .expect("repository should persist fact DSL");

    let loaded = repo
        .get_fact_dsl("mem-1")
        .expect("lookup should succeed")
        .expect("row should exist");
    assert_eq!(loaded, persisted);
    assert_eq!(loaded.classification_confidence, None);
    assert!(!loaded.needs_review);

    let listed = repo.list_fact_dsls().expect("listing should succeed");
    assert_eq!(listed, vec![persisted.clone()]);

    let by_topic = repo
        .list_fact_dsls_by_topic(TopicV1::Retrieval)
        .expect("topic filter should succeed");
    assert_eq!(by_topic, vec![persisted.clone()]);

    let by_path = repo
        .list_fact_dsls_by_path(&sample_dsl_record().taxonomy)
        .expect("path filter should succeed");
    assert_eq!(by_path, vec![persisted.clone()]);

    let removed = repo
        .delete_fact_dsl("mem-1")
        .expect("delete should succeed")
        .expect("row should exist");
    assert_eq!(removed, persisted);
    assert!(
        repo.get_fact_dsl("mem-1")
            .expect("lookup should succeed")
            .is_none()
    );
}

#[test]
fn sqlite_repository_fact_dsl_rows_follow_memory_record_lifecycle() {
    let path = fresh_db_path("dsl-cascade");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());
    let record = sample_memory_record();
    repo.insert_record(&record)
        .expect("authority record should insert");

    let persisted = PersistedFactDslRecordV1::from_fact_dsl_record("mem-1", &sample_dsl_record())
        .expect("persisted wrapper should build");
    repo.put_fact_dsl(&persisted)
        .expect("repository should persist fact DSL");

    db.conn()
        .execute("DELETE FROM memory_records WHERE id = ?1", [&record.id])
        .expect("authority row delete should succeed");

    assert!(
        repo.get_fact_dsl("mem-1")
            .expect("lookup should succeed")
            .is_none(),
        "fact DSL rows should cascade when the authority record is deleted"
    );
}

#[test]
fn sqlite_repository_fact_dsl_store_upserts_existing_rows() {
    let path = fresh_db_path("dsl-upsert");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());
    repo.insert_record(&sample_memory_record())
        .expect("authority record should insert");

    let mut first = PersistedFactDslRecordV1::from_fact_dsl_record("mem-1", &sample_dsl_record())
        .expect("persisted wrapper should build");
    first.classification_confidence = Some(0.82);
    first.needs_review = true;
    repo.put_fact_dsl(&first)
        .expect("initial persist should succeed");

    first.payload.why = Some("updated rationale".to_string());
    first.payload.impact = Some("updated impact".to_string());
    repo.put_fact_dsl(&first).expect("upsert should succeed");

    let loaded = repo
        .get_fact_dsl("mem-1")
        .expect("lookup should succeed")
        .expect("row should exist");
    assert_eq!(loaded.payload.why.as_deref(), Some("updated rationale"));
    assert_eq!(loaded.payload.impact.as_deref(), Some("updated impact"));
    assert_eq!(loaded.classification_confidence, Some(0.82));
    assert!(loaded.needs_review);
    assert_eq!(
        repo.list_fact_dsls().expect("listing should succeed").len(),
        1
    );
}

#[test]
fn sqlite_repository_loads_self_model_snapshot_with_tail_and_prunes_through_cursor() {
    let path = fresh_db_path("self-model-snapshot-tail");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());
    let subject_ref = "subject://agent/demo";
    let cursor_updated_at = "2026-04-19T00:01:00Z";
    let cursor_entry_id = "local-adaptation:self-state:preferred_mode:v2";

    for entry in [
        LocalAdaptationEntry {
            entry_id: "local-adaptation:self-state:preferred_mode:v1".to_string(),
            subject_ref: subject_ref.to_string(),
            target_kind: LocalAdaptationTargetKind::SelfState,
            key: "preferred_mode".to_string(),
            payload: LocalAdaptationPayload {
                value: json!("conservative"),
                trigger_kind: "user_preference".to_string(),
                evidence_refs: vec!["memo://project/self-model-repository".to_string()],
            },
            source_queue_item_id: None,
            created_at: "2026-04-19T00:00:00Z".to_string(),
            updated_at: "2026-04-19T00:00:00Z".to_string(),
        },
        LocalAdaptationEntry {
            entry_id: cursor_entry_id.to_string(),
            subject_ref: subject_ref.to_string(),
            target_kind: LocalAdaptationTargetKind::SelfState,
            key: "preferred_mode".to_string(),
            payload: LocalAdaptationPayload {
                value: json!("balanced"),
                trigger_kind: "user_preference".to_string(),
                evidence_refs: vec!["memo://project/self-model-repository".to_string()],
            },
            source_queue_item_id: None,
            created_at: cursor_updated_at.to_string(),
            updated_at: cursor_updated_at.to_string(),
        },
        LocalAdaptationEntry {
            entry_id: "local-adaptation:risk-boundary:deploy:v1".to_string(),
            subject_ref: subject_ref.to_string(),
            target_kind: LocalAdaptationTargetKind::RiskBoundary,
            key: "deploy".to_string(),
            payload: LocalAdaptationPayload {
                value: json!("requires_human_review"),
                trigger_kind: "risk_rule".to_string(),
                evidence_refs: vec!["memo://project/self-model-repository".to_string()],
            },
            source_queue_item_id: None,
            created_at: "2026-04-19T00:02:00Z".to_string(),
            updated_at: "2026-04-19T00:02:00Z".to_string(),
        },
    ] {
        repo.insert_local_adaptation_entry(&entry)
            .expect("local adaptation entry should insert");
    }

    let snapshot = PersistedSelfModelSnapshot {
        subject_ref: subject_ref.to_string(),
        snapshot_id: "self-model-snapshot-001".to_string(),
        entries: vec![PersistedSelfModelSnapshotEntry {
            target_kind: LocalAdaptationTargetKind::SelfState,
            key: "preferred_mode".to_string(),
            value: "balanced".to_string(),
            active: true,
            governance: None,
            source_queue_item_id: None,
            updated_at: cursor_updated_at.to_string(),
            entry_id: cursor_entry_id.to_string(),
        }],
        compacted_through_updated_at: cursor_updated_at.to_string(),
        compacted_through_entry_id: cursor_entry_id.to_string(),
        created_at: "2026-04-19T00:03:00Z".to_string(),
        updated_at: "2026-04-19T00:03:00Z".to_string(),
    };
    repo.replace_self_model_snapshot(&snapshot)
        .expect("snapshot should upsert");

    let persisted = repo
        .load_self_model_state(subject_ref)
        .expect("persisted self-model state should load");
    assert_eq!(persisted.snapshot, Some(snapshot.clone()));
    assert_eq!(persisted.tail_entries.len(), 1);
    assert_eq!(
        persisted.tail_entries[0].entry_id,
        "local-adaptation:risk-boundary:deploy:v1"
    );

    let pruned = repo
        .prune_local_adaptation_entries_through(
            subject_ref,
            &snapshot.compacted_through_updated_at,
            &snapshot.compacted_through_entry_id,
        )
        .expect("prune should succeed");
    assert_eq!(pruned, 2);

    let remaining = repo
        .list_local_adaptation_entries(subject_ref)
        .expect("remaining ledger rows should load");
    assert_eq!(remaining.len(), 1);
    assert_eq!(
        remaining[0].entry_id,
        "local-adaptation:risk-boundary:deploy:v1"
    );
}

#[test]
fn sqlite_repository_round_trips_self_model_snapshot_governance_metadata() {
    let path = fresh_db_path("self-model-snapshot-governance");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());

    let snapshot = PersistedSelfModelSnapshot {
        subject_ref: "subject://agent/demo".to_string(),
        snapshot_id: "self-model-snapshot-governance-001".to_string(),
        entries: vec![PersistedSelfModelSnapshotEntry {
            target_kind: LocalAdaptationTargetKind::SelfState,
            key: "preferred_mode".to_string(),
            value: "aggressive".to_string(),
            active: true,
            governance: Some(SelfModelGovernanceMetadata {
                resolution: SelfModelResolutionState::Accepted,
                conflicting_entry_ids: vec![
                    "local-adaptation:self-state:preferred_mode:v1".to_string(),
                ],
                review_reason: Some("approved after conflict review".to_string()),
            }),
            source_queue_item_id: Some("rumination-queue-1".to_string()),
            updated_at: "2026-04-19T00:01:00Z".to_string(),
            entry_id: "self-model-snapshot-entry:preferred_mode:v2".to_string(),
        }],
        compacted_through_updated_at: "2026-04-19T00:01:00Z".to_string(),
        compacted_through_entry_id: "self-model-snapshot-entry:preferred_mode:v2".to_string(),
        created_at: "2026-04-19T00:02:00Z".to_string(),
        updated_at: "2026-04-19T00:02:00Z".to_string(),
    };

    repo.replace_self_model_snapshot(&snapshot)
        .expect("snapshot should upsert");

    let loaded = repo
        .get_self_model_snapshot("subject://agent/demo")
        .expect("snapshot lookup should succeed")
        .expect("snapshot should exist");
    assert_eq!(loaded, snapshot);
}

#[test]
fn sqlite_repository_round_trips_world_model_snapshot_for_subject_and_scope() {
    let path = fresh_db_path("world-model-snapshot-round-trip");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());

    let snapshot = PersistedWorldModelSnapshot {
        subject_ref: "subject://agent/demo".to_string(),
        world_key: "current".to_string(),
        snapshot_id: "world-model-snapshot-001".to_string(),
        fragments: vec![PersistedWorldModelSnapshotFragment {
            record_id: "record-world".to_string(),
            snippet: "explicit world-model seam keeps persistence explainable".to_string(),
            citation: PersistedWorldModelCitation {
                record_id: "record-world".to_string(),
                source_uri: "memo://project/world-model".to_string(),
                source_kind: SourceKind::Note,
                source_label: Some("world-model".to_string()),
                recorded_at: "2026-04-20T09:00:00Z".to_string(),
                validity: ValidityWindow {
                    valid_from: Some("2026-04-20T00:00:00Z".to_string()),
                    valid_to: None,
                },
                anchor: PersistedWorldModelCitationAnchor {
                    chunk_index: 0,
                    chunk_count: 1,
                    anchor: ChunkAnchor::LineRange {
                        start_line: 1,
                        end_line: 4,
                    },
                },
            },
            provenance: Provenance {
                origin: "test".to_string(),
                imported_via: Some("fixture".to_string()),
                derived_from: vec!["memo://project/world-model".to_string()],
            },
            truth_context: PersistedWorldModelTruthContext {
                truth_layer: TruthLayer::T3,
                t3_state: None,
                open_review_ids: vec!["review-1".to_string()],
                open_candidate_ids: Vec::new(),
            },
            dsl: Some(
                PersistedFactDslRecordV1::from_fact_dsl_record("mem-1", &sample_dsl_record())
                    .expect("persisted wrapper should build")
                    .payload,
            ),
            trace: PersistedWorldModelTrace {
                matched_query: "world model snapshot".to_string(),
                query_strategies: vec![
                    PersistedWorldModelQueryStrategy::Jieba,
                    PersistedWorldModelQueryStrategy::Structured,
                ],
                channel_contribution: PersistedWorldModelChannelContribution::Hybrid,
                applied_filters: PersistedWorldModelAppliedFilters {
                    topic: Some("world_model".to_string()),
                    kind: Some("decision".to_string()),
                    ..Default::default()
                },
            },
            score: PersistedWorldModelScore {
                lexical_raw: -1.2,
                lexical_base: 0.45,
                keyword_bonus: 0.08,
                importance_bonus: 0.08,
                recency_bonus: 0.03,
                emotion_bonus: 0.0,
                final_score: 0.64,
            },
        }],
        created_at: "2026-04-20T10:00:00Z".to_string(),
        updated_at: "2026-04-20T10:00:00Z".to_string(),
    };

    repo.replace_world_model_snapshot(&snapshot)
        .expect("world-model snapshot should upsert");

    let loaded = repo
        .load_world_model_snapshot("subject://agent/demo", "current")
        .expect("world-model snapshot lookup should succeed")
        .expect("world-model snapshot should exist");
    assert_eq!(loaded, snapshot);
}
