use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    cognition::assembly::{
        AdaptiveSelfStateProvider, MinimalSelfStateProvider, WorkingMemoryAssembler,
        WorkingMemoryRequest,
    },
    core::db::Database,
    ingest::{IngestRequest, IngestService},
    memory::{
        record::{RecordType, Scope, TruthLayer},
        repository::{
            LocalAdaptationEntry, LocalAdaptationPayload, LocalAdaptationTargetKind,
            MemoryRepository,
        },
    },
};
use serde_json::json;

fn fresh_db_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-rumination-writeback-tests")
        .join(format!("{name}-{unique}"))
        .join("rumination-writeback.sqlite")
}

fn table_count(db: &Database, table: &str) -> i64 {
    db.conn()
        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| row.get(0))
        .expect("table count should load")
}

#[test]
fn short_cycle_overlay_provider_reads_local_adaptations() {
    let path = fresh_db_path("overlay-provider");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    let repository = MemoryRepository::new(db.conn());
    let subject_ref = "task://rumination/overlay";

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/rumination".to_string(),
            source_label: Some("rumination".to_string()),
            source_kind: None,
            content: "adaptive local state should remain explainable and queryable".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T12:00:00Z".to_string(),
            valid_from: Some("2026-04-16T00:00:00Z".to_string()),
            valid_to: None,
        })
        .expect("ingest should seed a searchable record");

    repository
        .insert_local_adaptation_entry(&LocalAdaptationEntry {
            entry_id: "adapt-self".to_string(),
            subject_ref: subject_ref.to_string(),
            target_kind: LocalAdaptationTargetKind::SelfState,
            key: "last_user_correction".to_string(),
            payload: LocalAdaptationPayload {
                value: json!("prefer cited answers"),
                trigger_kind: "user_correction".to_string(),
                evidence_refs: vec!["record://evidence/1".to_string()],
            },
            source_queue_item_id: Some("spq:item-1".to_string()),
            created_at: "2026-04-16T12:01:00Z".to_string(),
            updated_at: "2026-04-16T12:01:00Z".to_string(),
        })
        .expect("self-state adaptation should persist");
    repository
        .insert_local_adaptation_entry(&LocalAdaptationEntry {
            entry_id: "adapt-risk".to_string(),
            subject_ref: subject_ref.to_string(),
            target_kind: LocalAdaptationTargetKind::RiskBoundary,
            key: "unsafe_action".to_string(),
            payload: LocalAdaptationPayload {
                value: json!("blocked"),
                trigger_kind: "metacog_veto".to_string(),
                evidence_refs: vec!["record://evidence/2".to_string()],
            },
            source_queue_item_id: Some("spq:item-2".to_string()),
            created_at: "2026-04-16T12:02:00Z".to_string(),
            updated_at: "2026-04-16T12:02:00Z".to_string(),
        })
        .expect("risk-boundary adaptation should persist");
    repository
        .insert_local_adaptation_entry(&LocalAdaptationEntry {
            entry_id: "adapt-private-t3".to_string(),
            subject_ref: subject_ref.to_string(),
            target_kind: LocalAdaptationTargetKind::PrivateT3,
            key: "tentative_strategy".to_string(),
            payload: LocalAdaptationPayload {
                value: json!({"summary": "pause and request confirmation"}),
                trigger_kind: "action_failure".to_string(),
                evidence_refs: vec!["record://evidence/3".to_string()],
            },
            source_queue_item_id: Some("spq:item-3".to_string()),
            created_at: "2026-04-16T12:03:00Z".to_string(),
            updated_at: "2026-04-16T12:03:00Z".to_string(),
        })
        .expect("private-t3 adaptation should persist");

    let shared_truth_before = (
        table_count(&db, "memory_records"),
        table_count(&db, "truth_promotion_reviews"),
        table_count(&db, "truth_ontology_candidates"),
    );

    let assembler = WorkingMemoryAssembler::new(
        db.conn(),
        AdaptiveSelfStateProvider::new(MinimalSelfStateProvider),
    );
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("adaptive local state")
                .with_subject_ref(subject_ref)
                .with_task_context("overlay local adaptations for the next step"),
        )
        .expect("overlay-backed working-memory assembly should succeed");

    let facts = &working_memory.present.self_state.facts;
    assert!(
        facts.iter()
            .any(|fact| fact.key == "self_state:last_user_correction"
                && fact.value == "prefer cited answers"),
        "self-state adaptations should surface through the provider seam: {facts:?}"
    );
    assert!(
        facts.iter()
            .any(|fact| fact.key == "risk_boundary:unsafe_action" && fact.value == "blocked"),
        "risk-boundary adaptations should surface through the provider seam: {facts:?}"
    );
    assert!(
        facts.iter().any(|fact| {
            fact.key == "private_t3:tentative_strategy"
                && fact.value.contains("pause and request confirmation")
        }),
        "private-t3 adaptations should remain visible as local overlay facts: {facts:?}"
    );

    let shared_truth_after = (
        table_count(&db, "memory_records"),
        table_count(&db, "truth_promotion_reviews"),
        table_count(&db, "truth_ontology_candidates"),
    );
    assert_eq!(
        shared_truth_before, shared_truth_after,
        "assembly overlay must not mutate shared truth tables"
    );
}
