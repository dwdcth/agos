use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    cognition::{
        action::{ActionCandidate, ActionKind},
        assembly::{
            ActionSeed, MinimalSelfStateProvider, WorkingMemoryAssembler, WorkingMemoryRequest,
        },
        skill_memory::{
            ActionTemplate, Boundaries, ExpectedOutcome, Preconditions, SkillMemoryTemplate,
            SkillMemoryTemplateDecodeError, merge_runtime_skill_templates,
        },
    },
    core::db::Database,
    ingest::{IngestRequest, IngestService},
    memory::{
        record::{RecordType, Scope, TruthLayer},
        repository::{
            PersistedSkillMemoryTemplateCandidate, RuminationCandidate, RuminationCandidateKind,
            RuminationCandidateStatus, SKILL_TEMPLATE_PAYLOAD_VERSION,
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
        .join("agent-memos-skill-memory-tests")
        .join(format!("{name}-{unique}"))
        .join("skill-memory.sqlite")
}

fn ingest_record(
    service: &IngestService<'_>,
    source_uri: &str,
    source_label: &str,
    content: &str,
    recorded_at: &str,
) -> String {
    service
        .ingest(IngestRequest {
            source_uri: source_uri.to_string(),
            source_label: Some(source_label.to_string()),
            source_kind: None,
            content: content.to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: recorded_at.to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest should succeed")
        .record_ids[0]
        .clone()
}

#[test]
fn matching_skill_templates_project_into_branches_after_manual_seeds() {
    let path = fresh_db_path("matching-skill-template");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let decision_id = ingest_record(
        &ingest,
        "memo://project/skill-projection-decision",
        "skill-projection-decision",
        "skill projection should preserve cited decision evidence",
        "2026-04-28T10:00:00Z",
    );
    let guide_id = ingest_record(
        &ingest,
        "memo://project/skill-projection-guide",
        "skill-projection-guide",
        "skill projection guide keeps working memory explainable",
        "2026-04-28T10:05:00Z",
    );

    let mut skill_template = SkillMemoryTemplate::new(
        "clarify-before-acting",
        ActionTemplate::new(ActionKind::Regulative, "pause and request clarification")
            .with_intent("use a reusable regulating step before committing"),
    );
    skill_template.preconditions = Preconditions {
        required_goal_terms: vec!["clarify".to_string()],
        required_capability_flags: vec!["skill_projection_ready".to_string()],
        required_readiness_flags: vec!["citation_trace_ready".to_string()],
        ..Preconditions::default()
    };
    skill_template.expected_outcome = ExpectedOutcome {
        effects: vec![
            "ambiguity is reduced".to_string(),
            "citations remain attached".to_string(),
        ],
    };
    skill_template.boundaries = Boundaries {
        risk_markers: vec!["clarification_required".to_string()],
        supporting_record_ids: vec![decision_id.clone()],
        blocked_active_risks: Vec::new(),
    };
    skill_template.action.parameters = vec!["mode=safe".to_string()];

    let working_memory = WorkingMemoryAssembler::new(db.conn(), MinimalSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("skill projection")
                .with_active_goal("clarify the next step safely")
                .with_capability_flag("skill_projection_ready")
                .with_readiness_flag("citation_trace_ready")
                .with_action_seed(ActionSeed::new(ActionCandidate::new(
                    ActionKind::Epistemic,
                    "inspect the current evidence first",
                )))
                .with_skill_template(skill_template),
        )
        .expect("assembly should project matching skill templates");

    assert_eq!(working_memory.branches.len(), 2);
    assert_eq!(
        working_memory.branches[0].candidate.summary,
        "inspect the current evidence first"
    );
    assert_eq!(
        working_memory.branches[1].candidate.summary,
        "pause and request clarification"
    );
    assert_eq!(
        working_memory.branches[1].candidate.intent.as_deref(),
        Some("use a reusable regulating step before committing")
    );
    assert_eq!(
        working_memory.branches[1].candidate.parameters,
        vec!["mode=safe".to_string()]
    );
    assert_eq!(
        working_memory.branches[1].candidate.expected_effects,
        vec![
            "ambiguity is reduced".to_string(),
            "citations remain attached".to_string(),
        ]
    );
    assert_eq!(
        working_memory.branches[1].risk_markers,
        vec!["clarification_required".to_string()]
    );
    assert_eq!(
        working_memory.branches[1]
            .supporting_evidence
            .iter()
            .map(|fragment| fragment.record_id.as_str())
            .collect::<Vec<_>>(),
        vec![decision_id.as_str()]
    );
    assert!(
        working_memory.branches[0]
            .supporting_evidence
            .iter()
            .any(|fragment| fragment.record_id == guide_id),
        "manual branches should still use the ordinary materialization path"
    );
}

#[test]
fn unmet_skill_preconditions_skip_projection_without_failing_assembly() {
    let path = fresh_db_path("skill-preconditions");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let decision_id = ingest_record(
        &ingest,
        "memo://project/skill-preconditions",
        "skill-preconditions",
        "skill preconditions should fail safe when the request does not match",
        "2026-04-28T10:10:00Z",
    );

    let mut skill_template = SkillMemoryTemplate::new(
        "goal-mismatch",
        ActionTemplate::new(ActionKind::Instrumental, "apply the prepared action"),
    );
    skill_template.preconditions = Preconditions {
        required_goal_terms: vec!["clarify".to_string()],
        ..Preconditions::default()
    };
    skill_template.boundaries.supporting_record_ids = vec![decision_id];

    let working_memory = WorkingMemoryAssembler::new(db.conn(), MinimalSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("skill preconditions")
                .with_active_goal("ship the prepared action directly")
                .with_skill_template(skill_template),
        )
        .expect("assembly should skip unmet skill templates instead of failing");

    assert!(
        working_memory.branches.is_empty(),
        "unmet preconditions should skip projection without creating branches"
    );
}

#[test]
fn blocked_active_risks_suppress_skill_projection_but_keep_manual_seeds() {
    let path = fresh_db_path("skill-blocked-risk");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let decision_id = ingest_record(
        &ingest,
        "memo://project/skill-blocked-risk",
        "skill-blocked-risk",
        "blocked risks should suppress skill projection without disturbing manual seeds",
        "2026-04-28T10:20:00Z",
    );

    let mut skill_template = SkillMemoryTemplate::new(
        "blocked-by-risk",
        ActionTemplate::new(ActionKind::Instrumental, "apply the prepared action"),
    );
    skill_template.boundaries = Boundaries {
        risk_markers: vec!["unsafe_action".to_string()],
        supporting_record_ids: vec![decision_id],
        blocked_active_risks: vec!["deployment_frozen".to_string()],
    };

    let working_memory = WorkingMemoryAssembler::new(db.conn(), MinimalSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("blocked risks")
                .with_active_risk("deployment_frozen")
                .with_action_seed(ActionSeed::new(ActionCandidate::new(
                    ActionKind::Epistemic,
                    "collect more evidence",
                )))
                .with_skill_template(skill_template),
        )
        .expect("assembly should keep manual seeds when a skill template is blocked");

    assert_eq!(working_memory.branches.len(), 1);
    assert_eq!(
        working_memory.branches[0].candidate.summary,
        "collect more evidence"
    );
}

#[test]
fn persisted_skill_template_candidates_reconstruct_skill_memory_templates() {
    let mut original = SkillMemoryTemplate::new(
        "persisted-skill-template",
        ActionTemplate::new(ActionKind::Regulative, "pause and request clarification")
            .with_intent("keep the next step auditable"),
    );
    original.preconditions = Preconditions {
        required_goal_terms: vec!["clarify the next step safely".to_string()],
        required_task_context_terms: vec!["long-cycle learning".to_string()],
        required_capability_flags: vec!["skill_projection_ready".to_string()],
        required_readiness_flags: vec!["citation_trace_ready".to_string()],
        required_metacog_flag_codes: vec!["candidate_only".to_string()],
    };
    original.expected_outcome = ExpectedOutcome {
        effects: vec![
            "ambiguity is reduced".to_string(),
            "citations remain attached".to_string(),
        ],
    };
    original.boundaries = Boundaries {
        risk_markers: vec!["clarification_required".to_string()],
        supporting_record_ids: vec!["record-1".to_string(), "record-2".to_string()],
        blocked_active_risks: vec!["deployment_frozen".to_string()],
    };
    original.action.parameters = vec!["mode=safe".to_string()];

    let persisted = PersistedSkillMemoryTemplateCandidate {
        candidate: RuminationCandidate {
            candidate_id: "lpq:demo:skill_template".to_string(),
            source_queue_item_id: Some("lpq:demo".to_string()),
            candidate_kind: RuminationCandidateKind::SkillTemplate,
            subject_ref: "task://skill-memory/demo".to_string(),
            payload: json!({}),
            evidence_refs: vec!["record-1".to_string(), "record-2".to_string()],
            governance_ref_id: None,
            status: RuminationCandidateStatus::Pending,
            created_at: "2026-04-28T12:00:00Z".to_string(),
            updated_at: "2026-04-28T12:00:00Z".to_string(),
        },
        payload: original.to_candidate_payload(
            "evidence_accumulation",
            json!({
                "decision": {
                    "active_risks": ["deployment_frozen"],
                    "metacog_flags": [{"code": "candidate_only"}],
                }
            }),
            2,
        ),
    };

    let reconstructed = SkillMemoryTemplate::from_rumination_candidate(&persisted)
        .expect("persisted skill template candidate should reconstruct");

    assert_eq!(reconstructed, original);
    assert_eq!(
        persisted.payload.payload_version,
        SKILL_TEMPLATE_PAYLOAD_VERSION
    );
    assert_eq!(
        persisted.payload.template_summary,
        "pause and request clarification"
    );
}

#[test]
fn runtime_skill_template_merge_preserves_explicit_templates_and_adds_unique_persisted_ones() {
    let explicit = SkillMemoryTemplate::new(
        "shared-template",
        ActionTemplate::new(ActionKind::Regulative, "use the explicit template"),
    );
    let unique_persisted = SkillMemoryTemplate::new(
        "persisted-template",
        ActionTemplate::new(
            ActionKind::Instrumental,
            "add the consumed persisted template",
        ),
    );
    let duplicate_persisted = SkillMemoryTemplate::new(
        "shared-template",
        ActionTemplate::new(
            ActionKind::Regulative,
            "do not override the explicit template",
        ),
    );

    let merged = merge_runtime_skill_templates(
        std::slice::from_ref(&explicit),
        vec![duplicate_persisted, unique_persisted],
    );

    assert_eq!(merged.len(), 2);
    assert_eq!(merged[0], explicit);
    assert_eq!(
        merged[1].action.summary,
        "add the consumed persisted template"
    );
}

#[test]
fn persisted_skill_template_candidates_reject_unsupported_payload_versions() {
    let persisted = PersistedSkillMemoryTemplateCandidate {
        candidate: RuminationCandidate {
            candidate_id: "lpq:demo:skill_template".to_string(),
            source_queue_item_id: Some("lpq:demo".to_string()),
            candidate_kind: RuminationCandidateKind::SkillTemplate,
            subject_ref: "task://skill-memory/demo".to_string(),
            payload: json!({}),
            evidence_refs: vec!["record-1".to_string()],
            governance_ref_id: None,
            status: RuminationCandidateStatus::Pending,
            created_at: "2026-04-28T12:00:00Z".to_string(),
            updated_at: "2026-04-28T12:00:00Z".to_string(),
        },
        payload: SkillMemoryTemplate::new(
            "persisted-skill-template",
            ActionTemplate::new(ActionKind::Regulative, "pause and request clarification"),
        )
        .to_candidate_payload("evidence_accumulation", json!({}), 1),
    };
    let mut unsupported = persisted;
    unsupported.payload.payload_version = SKILL_TEMPLATE_PAYLOAD_VERSION + 1;

    let error = SkillMemoryTemplate::from_rumination_candidate(&unsupported)
        .expect_err("unsupported versions should fail deterministically");
    assert_eq!(
        error,
        SkillMemoryTemplateDecodeError::UnsupportedPayloadVersion {
            candidate_id: "lpq:demo:skill_template".to_string(),
            value: SKILL_TEMPLATE_PAYLOAD_VERSION + 1,
        }
    );
}
