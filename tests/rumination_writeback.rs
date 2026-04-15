use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    cognition::{
        assembly::{
            AdaptiveSelfStateProvider, MinimalSelfStateProvider, WorkingMemoryAssembler,
            WorkingMemoryRequest,
        },
        metacog::GateDecision,
        report::{DecisionReport, GateReport},
        rumination::{
            RuminationService, RuminationTriggerDecision, RuminationTriggerEvent,
            RuminationTriggerKind, ShortCycleWritebackReport,
        },
        working_memory::MetacognitiveFlag,
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

fn sample_veto_decision_report() -> DecisionReport {
    DecisionReport {
        scored_branches: Vec::new(),
        selected_branch: None,
        gate: GateReport {
            decision: GateDecision::HardVeto,
            diagnostics: vec!["unsafe_action".to_string()],
            rejected_branch: None,
            regulative_branch: None,
            safe_response: Some("pause execution".to_string()),
            autonomy_paused: false,
        },
        active_risks: vec!["unsafe_action".to_string()],
        metacog_flags: vec![MetacognitiveFlag {
            code: "human_review_required".to_string(),
            detail: Some("metacognition vetoed the last step".to_string()),
        }],
    }
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

#[test]
fn short_cycle_writeback_updates_local_state_without_mutating_shared_truth() {
    let path = fresh_db_path("short-cycle-writeback");
    let db = Database::open(&path).expect("database should open");
    let repository = MemoryRepository::new(db.conn());
    let service = RuminationService::new(db.conn());
    let subject_ref = "task://rumination/writeback";
    let decision_report = sample_veto_decision_report();

    let correction = RuminationTriggerEvent::from_user_correction(
        subject_ref,
        json!({
            "self_state": {
                "preferred_response_mode": "cite_before_answer"
            },
            "risk_boundary": {
                "unsafe_action": "blocked"
            },
            "private_t3": {
                "tentative_strategy": {
                    "summary": "pause and request confirmation"
                }
            }
        }),
        "2026-04-16T13:00:00Z",
        "2026-04-16",
        Some("user-correction-1".to_string()),
    );
    let duplicate_correction = RuminationTriggerEvent::from_user_correction(
        subject_ref,
        json!({
            "self_state": {
                "preferred_response_mode": "cite_before_answer"
            }
        }),
        "2026-04-16T13:00:05Z",
        "2026-04-16",
        Some("user-correction-1".to_string()),
    );
    let action_failure = RuminationTriggerEvent::from_action_failure(
        subject_ref,
        "tool_execution_failed",
        "command failed after unsafe preconditions",
        vec!["unsafe_action".to_string()],
        "2026-04-16T13:01:00Z",
        "2026-04-16",
        Some("failure-1".to_string()),
    );
    let veto = RuminationTriggerEvent::from_decision_report(
        RuminationTriggerKind::MetacogVeto,
        subject_ref,
        &decision_report,
        "2026-04-16T13:02:00Z",
        "2026-04-16",
        None,
        Some("veto-1".to_string()),
    )
    .expect("decision report should normalize");

    assert!(matches!(
        service.schedule(correction).expect("correction should schedule"),
        RuminationTriggerDecision::Enqueued { .. }
    ));
    assert!(matches!(
        service
            .schedule(duplicate_correction)
            .expect("duplicate should hit throttle"),
        RuminationTriggerDecision::Deduped { .. }
    ));
    assert!(matches!(
        service
            .schedule(action_failure)
            .expect("action failure should schedule"),
        RuminationTriggerDecision::Enqueued { .. }
    ));
    assert!(matches!(
        service.schedule(veto).expect("veto should schedule"),
        RuminationTriggerDecision::Enqueued { .. }
    ));

    let shared_truth_before = (
        table_count(&db, "memory_records"),
        table_count(&db, "truth_promotion_reviews"),
        table_count(&db, "truth_ontology_candidates"),
    );

    let mut reports: Vec<ShortCycleWritebackReport> = Vec::new();
    while let Some(report) = service
        .drain_short_cycle("2026-04-16T14:00:00Z")
        .expect("short-cycle drain should succeed")
    {
        reports.push(report);
        if reports.len() > 8 {
            panic!("short-cycle drain should not loop indefinitely");
        }
    }

    assert_eq!(reports.len(), 3, "deduped work must not drain twice");
    assert!(
        reports.iter().all(|report| report.entry_count > 0),
        "each drained short-cycle item should write at least one local adaptation entry: {reports:?}"
    );

    let entries = repository
        .list_local_adaptation_entries(subject_ref)
        .expect("local adaptations should load");
    assert!(
        entries.iter().any(|entry| entry.target_kind == LocalAdaptationTargetKind::SelfState),
        "short-cycle writeback should update self-state entries: {entries:?}"
    );
    assert!(
        entries
            .iter()
            .any(|entry| entry.target_kind == LocalAdaptationTargetKind::RiskBoundary),
        "short-cycle writeback should update risk-boundary entries: {entries:?}"
    );
    assert!(
        entries
            .iter()
            .any(|entry| entry.target_kind == LocalAdaptationTargetKind::PrivateT3),
        "short-cycle writeback should update private-t3-adjacent entries: {entries:?}"
    );

    let queue_items = repository
        .list_rumination_queue_items("spq")
        .expect("spq queue items should load");
    assert!(
        queue_items
            .iter()
            .all(|item| item.status.as_str() == "completed"),
        "all drained spq items should complete successfully: {queue_items:?}"
    );

    let shared_truth_after = (
        table_count(&db, "memory_records"),
        table_count(&db, "truth_promotion_reviews"),
        table_count(&db, "truth_ontology_candidates"),
    );
    assert_eq!(
        shared_truth_before, shared_truth_after,
        "short-cycle writeback must not mutate shared truth or governance tables"
    );
}
