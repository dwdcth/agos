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
            PersistedSkillMemoryTemplateAction, PersistedSkillMemoryTemplateBoundaries,
            PersistedSkillMemoryTemplateExpectedOutcome, PersistedSkillMemoryTemplatePayload,
            PersistedSkillMemoryTemplatePreconditions, PersistedWorldModelAppliedFilters,
            PersistedWorldModelChannelContribution, PersistedWorldModelCitation,
            PersistedWorldModelCitationAnchor, PersistedWorldModelQueryStrategy,
            PersistedWorldModelScore, PersistedWorldModelSnapshot,
            PersistedWorldModelSnapshotFragment, PersistedWorldModelTrace,
            PersistedWorldModelTruthContext, RepositoryError, RuminationCandidate,
            RuminationCandidateKind, RuminationCandidateStatus, SKILL_TEMPLATE_PAYLOAD_VERSION,
            SelfModelGovernanceMetadata, SelfModelResolutionState,
            SkillTemplateCandidateLifecycleError,
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

fn sample_skill_template_candidate(
    candidate_id: &str,
    subject_ref: &str,
    created_at: &str,
) -> RuminationCandidate {
    let payload = PersistedSkillMemoryTemplatePayload {
        payload_version: SKILL_TEMPLATE_PAYLOAD_VERSION,
        template_id: format!("template:{candidate_id}"),
        template_summary: "pause and request clarification".to_string(),
        preconditions: PersistedSkillMemoryTemplatePreconditions {
            required_goal_terms: vec!["clarify the next step safely".to_string()],
            required_task_context_terms: vec!["long-cycle learning".to_string()],
            required_capability_flags: vec!["skill_projection_ready".to_string()],
            required_readiness_flags: vec!["citation_trace_ready".to_string()],
            required_metacog_flag_codes: vec!["candidate_only".to_string()],
        },
        action: PersistedSkillMemoryTemplateAction {
            kind: "regulative".to_string(),
            summary: "pause and request clarification".to_string(),
            intent: Some("keep the next step auditable".to_string()),
            parameters: vec!["mode=safe".to_string()],
        },
        expected_outcome: PersistedSkillMemoryTemplateExpectedOutcome {
            effects: vec!["ambiguity is reduced".to_string()],
        },
        boundaries: PersistedSkillMemoryTemplateBoundaries {
            risk_markers: vec!["clarification_required".to_string()],
            supporting_record_ids: vec!["mem-1".to_string()],
            blocked_active_risks: vec!["deployment_frozen".to_string()],
        },
        trigger_kind: "evidence_accumulation".to_string(),
        source_report: json!({
            "decision": {
                "active_risks": ["deployment_frozen"],
                "metacog_flags": [{"code": "candidate_only"}],
            }
        }),
        evidence_count: 1,
    };

    RuminationCandidate {
        candidate_id: candidate_id.to_string(),
        source_queue_item_id: Some(format!("lpq:{candidate_id}")),
        candidate_kind: RuminationCandidateKind::SkillTemplate,
        subject_ref: subject_ref.to_string(),
        payload: serde_json::to_value(payload).expect("skill template payload should serialize"),
        evidence_refs: vec!["mem-1".to_string()],
        governance_ref_id: None,
        status: RuminationCandidateStatus::Pending,
        created_at: created_at.to_string(),
        updated_at: created_at.to_string(),
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
fn sqlite_repository_lists_only_skill_template_candidates_and_filters_by_subject() {
    let path = fresh_db_path("skill-template-candidates");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());

    let first = sample_skill_template_candidate(
        "lpq:subject-a:skill_template",
        "task://subject-a",
        "2026-04-28T12:00:00Z",
    );
    let second = sample_skill_template_candidate(
        "lpq:subject-b:skill_template",
        "task://subject-b",
        "2026-04-28T12:05:00Z",
    );
    let promotion = RuminationCandidate {
        candidate_id: "lpq:subject-a:promotion_candidate".to_string(),
        source_queue_item_id: Some("lpq:subject-a".to_string()),
        candidate_kind: RuminationCandidateKind::PromotionCandidate,
        subject_ref: "task://subject-a".to_string(),
        payload: json!({
            "promotion_path": "pending_governance_bridge",
            "source_record_id": "mem-1",
            "basis_record_ids": ["mem-1"],
        }),
        evidence_refs: vec!["mem-1".to_string()],
        governance_ref_id: Some("review:lpq:subject-a".to_string()),
        status: RuminationCandidateStatus::Pending,
        created_at: "2026-04-28T12:10:00Z".to_string(),
        updated_at: "2026-04-28T12:10:00Z".to_string(),
    };

    for candidate in [&first, &second, &promotion] {
        repo.insert_rumination_candidate(candidate)
            .expect("rumination candidate should insert");
    }

    let all_skill_candidates = repo
        .list_skill_template_candidates()
        .expect("skill template candidates should load");
    assert_eq!(all_skill_candidates.len(), 2);
    assert_eq!(
        all_skill_candidates
            .iter()
            .map(|candidate| candidate.candidate.subject_ref.as_str())
            .collect::<Vec<_>>(),
        vec!["task://subject-a", "task://subject-b"]
    );
    assert!(
        all_skill_candidates
            .iter()
            .all(|candidate| candidate.candidate.candidate_kind
                == RuminationCandidateKind::SkillTemplate)
    );
    assert_eq!(
        all_skill_candidates[0].payload.action.summary,
        "pause and request clarification"
    );

    let filtered = repo
        .list_skill_template_candidates_for_subject("task://subject-a")
        .expect("subject-scoped skill template candidates should load");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].candidate.candidate_id, first.candidate_id);
    assert_eq!(
        filtered[0].payload.template_id,
        "template:lpq:subject-a:skill_template"
    );
}

#[test]
fn sqlite_repository_lists_subject_skill_template_candidates_across_lifecycle_statuses() {
    let path = fresh_db_path("skill-template-candidates-by-subject-statuses");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());
    let subject_ref = "task://subject-a";

    let mut pending = sample_skill_template_candidate(
        "lpq:subject-a:pending",
        subject_ref,
        "2026-04-28T12:20:00Z",
    );
    pending.status = RuminationCandidateStatus::Pending;

    let mut consumed = sample_skill_template_candidate(
        "lpq:subject-a:consumed",
        subject_ref,
        "2026-04-28T12:21:00Z",
    );
    consumed.status = RuminationCandidateStatus::Consumed;

    let mut rejected = sample_skill_template_candidate(
        "lpq:subject-a:rejected",
        subject_ref,
        "2026-04-28T12:22:00Z",
    );
    rejected.status = RuminationCandidateStatus::Rejected;

    let mut archived = sample_skill_template_candidate(
        "lpq:subject-a:archived",
        subject_ref,
        "2026-04-28T12:23:00Z",
    );
    archived.status = RuminationCandidateStatus::Archived;

    for candidate in [&pending, &consumed, &rejected, &archived] {
        repo.insert_rumination_candidate(candidate)
            .expect("rumination candidate should insert");
    }

    let filtered = repo
        .list_skill_template_candidates_for_subject(subject_ref)
        .expect("subject-scoped skill template candidates should load");

    assert_eq!(filtered.len(), 4);
    assert_eq!(
        filtered
            .iter()
            .map(|candidate| candidate.candidate.status)
            .collect::<Vec<_>>(),
        vec![
            RuminationCandidateStatus::Pending,
            RuminationCandidateStatus::Consumed,
            RuminationCandidateStatus::Rejected,
            RuminationCandidateStatus::Archived,
        ]
    );
}

#[test]
fn sqlite_repository_rejects_legacy_placeholder_skill_template_candidates() {
    let path = fresh_db_path("legacy-skill-template-candidates");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());

    repo.insert_rumination_candidate(&RuminationCandidate {
        candidate_id: "lpq:legacy:skill_template".to_string(),
        source_queue_item_id: Some("lpq:legacy".to_string()),
        candidate_kind: RuminationCandidateKind::SkillTemplate,
        subject_ref: "task://legacy".to_string(),
        payload: json!({
            "template_summary": "pause and request clarification",
            "trigger_kind": "evidence_accumulation",
            "source_report": {"decision": {"selected_branch": null}},
            "evidence_count": 1,
        }),
        evidence_refs: vec!["mem-legacy".to_string()],
        governance_ref_id: None,
        status: RuminationCandidateStatus::Pending,
        created_at: "2026-04-28T12:00:00Z".to_string(),
        updated_at: "2026-04-28T12:00:00Z".to_string(),
    })
    .expect("legacy placeholder candidate should insert for compatibility checks");

    let error = repo
        .list_skill_template_candidates()
        .expect_err("legacy placeholder rows should fail with a typed boundary error");
    assert!(matches!(
        error,
        RepositoryError::LegacySkillTemplatePayload { ref candidate_id }
            if candidate_id == "lpq:legacy:skill_template"
    ));
}

#[test]
fn sqlite_repository_skill_template_lifecycle_bridge_does_not_mutate_invalid_payload_rows() {
    let path = fresh_db_path("legacy-skill-template-lifecycle-bridge");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());

    repo.insert_rumination_candidate(&RuminationCandidate {
        candidate_id: "lpq:legacy:lifecycle".to_string(),
        source_queue_item_id: Some("lpq:legacy".to_string()),
        candidate_kind: RuminationCandidateKind::SkillTemplate,
        subject_ref: "task://legacy".to_string(),
        payload: json!({
            "template_summary": "pause and request clarification",
            "trigger_kind": "evidence_accumulation",
            "source_report": {"decision": {"selected_branch": null}},
            "evidence_count": 1,
        }),
        evidence_refs: vec!["mem-legacy".to_string()],
        governance_ref_id: Some("review:legacy".to_string()),
        status: RuminationCandidateStatus::Pending,
        created_at: "2026-04-29T10:00:00Z".to_string(),
        updated_at: "2026-04-29T10:00:00Z".to_string(),
    })
    .expect("legacy placeholder candidate should insert for lifecycle checks");

    let error = repo
        .consume_skill_template_candidate("lpq:legacy:lifecycle", "2026-04-29T10:05:00Z")
        .expect_err("invalid payloads should fail before any status mutation");
    assert!(matches!(
        error,
        SkillTemplateCandidateLifecycleError::Repository(
            RepositoryError::LegacySkillTemplatePayload { ref candidate_id }
        ) if candidate_id == "lpq:legacy:lifecycle"
    ));

    let stored = repo
        .get_rumination_candidate("lpq:legacy:lifecycle")
        .expect("lookup should succeed")
        .expect("candidate should still exist after a failed transition");
    assert_eq!(stored.status, RuminationCandidateStatus::Pending);
    assert_eq!(stored.updated_at, "2026-04-29T10:00:00Z");
    assert_eq!(stored.governance_ref_id.as_deref(), Some("review:legacy"));
}

#[test]
fn sqlite_repository_filters_runtime_skill_template_candidates_by_status_and_subject() {
    let path = fresh_db_path("runtime-skill-template-candidates");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());

    let mut consumed = sample_skill_template_candidate(
        "lpq:subject-a:consumed",
        "task://subject-a",
        "2026-04-28T12:00:00Z",
    );
    consumed.status = RuminationCandidateStatus::Consumed;

    let mut pending = sample_skill_template_candidate(
        "lpq:subject-a:pending",
        "task://subject-a",
        "2026-04-28T12:01:00Z",
    );
    pending.status = RuminationCandidateStatus::Pending;

    let mut rejected = sample_skill_template_candidate(
        "lpq:subject-a:rejected",
        "task://subject-a",
        "2026-04-28T12:02:00Z",
    );
    rejected.status = RuminationCandidateStatus::Rejected;

    let mut archived = sample_skill_template_candidate(
        "lpq:subject-a:archived",
        "task://subject-a",
        "2026-04-28T12:03:00Z",
    );
    archived.status = RuminationCandidateStatus::Archived;

    let mut other_subject = sample_skill_template_candidate(
        "lpq:subject-b:consumed",
        "task://subject-b",
        "2026-04-28T12:04:00Z",
    );
    other_subject.status = RuminationCandidateStatus::Consumed;

    for candidate in [&consumed, &pending, &rejected, &archived, &other_subject] {
        repo.insert_rumination_candidate(candidate)
            .expect("rumination candidate should insert");
    }

    let runtime_candidates = repo
        .list_consumed_skill_template_candidates_for_subject("task://subject-a")
        .expect("runtime skill template candidates should load");

    assert_eq!(runtime_candidates.len(), 1);
    assert_eq!(
        runtime_candidates[0].candidate.candidate_id,
        "lpq:subject-a:consumed"
    );
    assert_eq!(
        runtime_candidates[0].candidate.status,
        RuminationCandidateStatus::Consumed
    );
    assert_eq!(
        runtime_candidates[0].candidate.subject_ref,
        "task://subject-a"
    );
}

#[test]
fn sqlite_repository_consumes_skill_template_candidates_and_preserves_metadata() {
    let path = fresh_db_path("skill-template-candidate-consume");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());

    let mut pending = sample_skill_template_candidate(
        "lpq:subject-a:lifecycle",
        "task://subject-a",
        "2026-04-29T08:00:00Z",
    );
    pending.governance_ref_id = Some("review:lpq:subject-a:lifecycle".to_string());
    let original_payload = pending.payload.clone();
    let original_evidence_refs = pending.evidence_refs.clone();
    let original_source_queue_item_id = pending.source_queue_item_id.clone();
    let original_governance_ref_id = pending.governance_ref_id.clone();

    repo.insert_rumination_candidate(&pending)
        .expect("pending skill template candidate should insert");

    let consumed = repo
        .consume_skill_template_candidate(&pending.candidate_id, "2026-04-29T08:05:00Z")
        .expect("consume helper should transition the candidate");

    assert_eq!(
        consumed.candidate.status,
        RuminationCandidateStatus::Consumed
    );
    assert_eq!(consumed.candidate.created_at, pending.created_at);
    assert_eq!(consumed.candidate.updated_at, "2026-04-29T08:05:00Z");
    assert_eq!(consumed.candidate.payload, original_payload);
    assert_eq!(consumed.candidate.evidence_refs, original_evidence_refs);
    assert_eq!(
        consumed.candidate.source_queue_item_id,
        original_source_queue_item_id
    );
    assert_eq!(consumed.candidate.subject_ref, pending.subject_ref);
    assert_eq!(
        consumed.candidate.governance_ref_id,
        original_governance_ref_id
    );

    let stored = repo
        .get_rumination_candidate(&pending.candidate_id)
        .expect("lookup should succeed")
        .expect("candidate should still exist after transition");
    assert_eq!(stored.status, RuminationCandidateStatus::Consumed);
    assert_eq!(stored.updated_at, "2026-04-29T08:05:00Z");
    assert_eq!(stored.payload, pending.payload);
    assert_eq!(stored.evidence_refs, pending.evidence_refs);
    assert_eq!(stored.source_queue_item_id, pending.source_queue_item_id);
    assert_eq!(stored.governance_ref_id, pending.governance_ref_id);
}

#[test]
fn sqlite_repository_rejects_and_archives_skill_template_candidates() {
    let path = fresh_db_path("skill-template-candidate-reject-archive");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());

    let pending = sample_skill_template_candidate(
        "lpq:subject-a:reject",
        "task://subject-a",
        "2026-04-29T08:10:00Z",
    );
    let mut consumed = sample_skill_template_candidate(
        "lpq:subject-a:archive",
        "task://subject-a",
        "2026-04-29T08:11:00Z",
    );
    consumed.status = RuminationCandidateStatus::Consumed;

    repo.insert_rumination_candidate(&pending)
        .expect("pending skill template candidate should insert");
    repo.insert_rumination_candidate(&consumed)
        .expect("consumed skill template candidate should insert");

    let rejected = repo
        .reject_skill_template_candidate(&pending.candidate_id, "2026-04-29T08:12:00Z")
        .expect("reject helper should transition the candidate");
    let archived = repo
        .archive_skill_template_candidate(&consumed.candidate_id, "2026-04-29T08:13:00Z")
        .expect("archive helper should transition the candidate");

    assert_eq!(
        rejected.candidate.status,
        RuminationCandidateStatus::Rejected
    );
    assert_eq!(rejected.candidate.updated_at, "2026-04-29T08:12:00Z");
    assert_eq!(rejected.candidate.payload, pending.payload);
    assert_eq!(
        archived.candidate.status,
        RuminationCandidateStatus::Archived
    );
    assert_eq!(archived.candidate.updated_at, "2026-04-29T08:13:00Z");
    assert_eq!(archived.candidate.payload, consumed.payload);
}

#[test]
fn sqlite_repository_skill_template_lifecycle_bridge_rejects_invalid_transitions_without_mutation()
{
    let path = fresh_db_path("skill-template-candidate-invalid-transitions");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());

    let pending = sample_skill_template_candidate(
        "lpq:subject-a:archive-from-pending",
        "task://subject-a",
        "2026-04-29T08:14:00Z",
    );
    let mut consumed = sample_skill_template_candidate(
        "lpq:subject-a:reject-from-consumed",
        "task://subject-a",
        "2026-04-29T08:15:00Z",
    );
    consumed.status = RuminationCandidateStatus::Consumed;
    let mut rejected = sample_skill_template_candidate(
        "lpq:subject-a:consume-from-rejected",
        "task://subject-a",
        "2026-04-29T08:16:00Z",
    );
    rejected.status = RuminationCandidateStatus::Rejected;
    let mut archived = sample_skill_template_candidate(
        "lpq:subject-a:consume-from-archived",
        "task://subject-a",
        "2026-04-29T08:17:00Z",
    );
    archived.status = RuminationCandidateStatus::Archived;

    for candidate in [&pending, &consumed, &rejected, &archived] {
        repo.insert_rumination_candidate(candidate)
            .expect("candidate should insert");
    }

    let archive_pending = repo
        .archive_skill_template_candidate(&pending.candidate_id, "2026-04-29T08:18:00Z")
        .expect_err("pending candidates must not archive directly");
    assert!(matches!(
        archive_pending,
        SkillTemplateCandidateLifecycleError::InvalidTransition {
            ref candidate_id,
            ref current,
            ref next,
        } if candidate_id == "lpq:subject-a:archive-from-pending"
            && current == "pending"
            && next == "archived"
    ));

    let reject_consumed = repo
        .reject_skill_template_candidate(&consumed.candidate_id, "2026-04-29T08:19:00Z")
        .expect_err("consumed candidates must not reject after activation");
    assert!(matches!(
        reject_consumed,
        SkillTemplateCandidateLifecycleError::InvalidTransition {
            ref candidate_id,
            ref current,
            ref next,
        } if candidate_id == "lpq:subject-a:reject-from-consumed"
            && current == "consumed"
            && next == "rejected"
    ));

    let consume_rejected = repo
        .consume_skill_template_candidate(&rejected.candidate_id, "2026-04-29T08:20:00Z")
        .expect_err("rejected candidates must not reactivate");
    assert!(matches!(
        consume_rejected,
        SkillTemplateCandidateLifecycleError::InvalidTransition {
            ref candidate_id,
            ref current,
            ref next,
        } if candidate_id == "lpq:subject-a:consume-from-rejected"
            && current == "rejected"
            && next == "consumed"
    ));

    let consume_archived = repo
        .consume_skill_template_candidate(&archived.candidate_id, "2026-04-29T08:21:00Z")
        .expect_err("archived candidates must stay terminal");
    assert!(matches!(
        consume_archived,
        SkillTemplateCandidateLifecycleError::InvalidTransition {
            ref candidate_id,
            ref current,
            ref next,
        } if candidate_id == "lpq:subject-a:consume-from-archived"
            && current == "archived"
            && next == "consumed"
    ));

    for (candidate_id, expected_status, expected_updated_at) in [
        (
            "lpq:subject-a:archive-from-pending",
            RuminationCandidateStatus::Pending,
            "2026-04-29T08:14:00Z",
        ),
        (
            "lpq:subject-a:reject-from-consumed",
            RuminationCandidateStatus::Consumed,
            "2026-04-29T08:15:00Z",
        ),
        (
            "lpq:subject-a:consume-from-rejected",
            RuminationCandidateStatus::Rejected,
            "2026-04-29T08:16:00Z",
        ),
        (
            "lpq:subject-a:consume-from-archived",
            RuminationCandidateStatus::Archived,
            "2026-04-29T08:17:00Z",
        ),
    ] {
        let stored = repo
            .get_rumination_candidate(candidate_id)
            .expect("lookup should succeed")
            .expect("candidate should remain stored");
        assert_eq!(stored.status, expected_status);
        assert_eq!(stored.updated_at, expected_updated_at);
    }
}

#[test]
fn sqlite_repository_skill_template_lifecycle_bridge_rejects_wrong_kind_and_missing_candidate() {
    let path = fresh_db_path("skill-template-candidate-lifecycle-errors");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());

    repo.insert_rumination_candidate(&RuminationCandidate {
        candidate_id: "lpq:subject-a:promotion".to_string(),
        source_queue_item_id: Some("lpq:subject-a".to_string()),
        candidate_kind: RuminationCandidateKind::PromotionCandidate,
        subject_ref: "task://subject-a".to_string(),
        payload: json!({
            "promotion_path": "pending_governance_bridge",
            "source_record_id": "mem-1",
            "basis_record_ids": ["mem-1"],
        }),
        evidence_refs: vec!["mem-1".to_string()],
        governance_ref_id: Some("review:lpq:subject-a".to_string()),
        status: RuminationCandidateStatus::Pending,
        created_at: "2026-04-29T08:20:00Z".to_string(),
        updated_at: "2026-04-29T08:20:00Z".to_string(),
    })
    .expect("promotion candidate should insert");

    let wrong_kind = repo
        .consume_skill_template_candidate("lpq:subject-a:promotion", "2026-04-29T08:21:00Z")
        .expect_err("wrong candidate kinds should be rejected explicitly");
    assert!(matches!(
        wrong_kind,
        SkillTemplateCandidateLifecycleError::WrongCandidateKind {
            ref candidate_id,
            ref actual,
        } if candidate_id == "lpq:subject-a:promotion" && actual == "promotion_candidate"
    ));

    let missing = repo
        .archive_skill_template_candidate("lpq:missing:skill_template", "2026-04-29T08:22:00Z")
        .expect_err("missing candidates should fail explicitly");
    assert!(matches!(
        missing,
        SkillTemplateCandidateLifecycleError::CandidateNotFound { ref candidate_id }
            if candidate_id == "lpq:missing:skill_template"
    ));
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
