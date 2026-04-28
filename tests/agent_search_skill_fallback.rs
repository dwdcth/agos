use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    agent::orchestration::{
        AgentSearchBranchValue, AgentSearchOrchestrator, AgentSearchRequest, AgentSearchRunner,
    },
    cognition::{
        action::ActionKind,
        assembly::{MinimalSelfStateProvider, WorkingMemoryRequest},
        metacog::GateDecision,
        skill_memory::{
            ActionTemplate, Boundaries, ExpectedOutcome, Preconditions, SkillMemoryTemplate,
        },
        value::{ValueConfig, ValueVector},
    },
    core::db::Database,
    ingest::{IngestRequest, IngestService},
    memory::record::{RecordType, Scope, TruthLayer},
};

fn fresh_db_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-agent-search-tests")
        .join(format!("{name}-{unique}"))
        .join("agent-memos.sqlite")
}

#[test]
fn skill_generated_branch_uses_unique_same_kind_value_fallback_and_reaches_gating() {
    let path = fresh_db_path("skill-branch-value-fallback");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());
    let decision = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/skill-branch-value-fallback".to_string(),
            source_label: Some("skill-branch-value-fallback".to_string()),
            source_kind: None,
            content: "skill-generated branches should reuse the unique same-kind value template"
                .to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-28T11:00:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest should succeed");

    let mut skill_template = SkillMemoryTemplate::new(
        "safe-apply-template",
        ActionTemplate::new(ActionKind::Instrumental, "apply the prepared change safely")
            .with_intent("reuse the procedural template instead of a manual seed"),
    );
    skill_template.preconditions = Preconditions {
        required_goal_terms: vec!["apply".to_string()],
        ..Preconditions::default()
    };
    skill_template.expected_outcome = ExpectedOutcome {
        effects: vec!["the prepared change is applied".to_string()],
    };
    skill_template.boundaries = Boundaries {
        risk_markers: vec!["clarification_required".to_string()],
        supporting_record_ids: vec![decision.record_ids[0].clone()],
        blocked_active_risks: Vec::new(),
    };

    let request = AgentSearchRequest::new(
        WorkingMemoryRequest::new("same kind fallback")
            .with_active_goal("apply the prepared change safely")
            .with_skill_template(skill_template),
    )
    .with_max_steps(1)
    .with_working_memory_limit(1)
    .with_branch_value(AgentSearchBranchValue::new(
        ActionKind::Instrumental,
        "take the leading action",
        ValueVector {
            goal_progress: 0.90,
            information_gain: 0.35,
            risk_avoidance: 0.50,
            resource_efficiency: 0.85,
            agent_robustness: 0.65,
        },
    ));

    let report = AgentSearchOrchestrator::with_services(
        db.conn(),
        MinimalSelfStateProvider,
        ValueConfig::default(),
    )
    .run(&request)
    .expect("skill-generated branches should use the unique same-kind fallback");

    assert_eq!(report.decision.gate.decision, GateDecision::SoftVeto);
    assert_eq!(report.working_memory.branches.len(), 1);
    assert_eq!(
        report.working_memory.branches[0].candidate.summary,
        "apply the prepared change safely"
    );
    assert_eq!(
        report
            .decision
            .gate
            .rejected_branch
            .as_ref()
            .expect("soft veto should preserve the scored skill branch")
            .branch
            .candidate
            .summary,
        "apply the prepared change safely"
    );
}
