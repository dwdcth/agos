use agent_memos::cognition::{
    action::{ActionBranch, ActionCandidate, ActionKind},
    metacog::{GateDecision, MetacognitionService},
    report::DecisionReport,
    value::{BranchValueInput, ValueConfig, ValueScorer, ValueVector},
    working_memory::{MetacognitiveFlag, PresentFrame, SelfStateSnapshot, WorkingMemory},
};

fn sample_branch(kind: ActionKind, summary: &str) -> ActionBranch {
    ActionBranch::new(ActionCandidate::new(kind, summary))
}

fn sample_working_memory(
    branches: Vec<ActionBranch>,
    metacog_flags: Vec<MetacognitiveFlag>,
) -> WorkingMemory {
    WorkingMemory {
        present: PresentFrame {
            world_fragments: Vec::new(),
            self_state: SelfStateSnapshot {
                task_context: Some("evaluate candidate actions".to_string()),
                capability_flags: vec!["lexical_search_ready".to_string()],
                readiness_flags: vec!["truth_governance_ready".to_string()],
                facts: Vec::new(),
            },
            active_goal: None,
            active_risks: Vec::new(),
            metacog_flags,
        },
        branches,
    }
}

fn approx_eq(left: f32, right: f32) {
    assert!(
        (left - right).abs() < 0.0001,
        "expected {left} to be approximately {right}"
    );
}

#[test]
fn value_scorer_projects_five_dimensions_with_dynamic_weights() {
    let epistemic = BranchValueInput::new(
        sample_branch(ActionKind::Epistemic, "inspect the evidence gaps"),
        ValueVector {
            goal_progress: 0.45,
            information_gain: 0.95,
            risk_avoidance: 0.85,
            resource_efficiency: 0.60,
            agent_robustness: 0.80,
        },
    );
    let instrumental = BranchValueInput::new(
        sample_branch(ActionKind::Instrumental, "apply the code change now"),
        ValueVector {
            goal_progress: 0.95,
            information_gain: 0.35,
            risk_avoidance: 0.55,
            resource_efficiency: 0.80,
            agent_robustness: 0.65,
        },
    );
    let regulative = BranchValueInput::new(
        sample_branch(ActionKind::Regulative, "pause and request clarification"),
        ValueVector {
            goal_progress: 0.55,
            information_gain: 0.40,
            risk_avoidance: 0.98,
            resource_efficiency: 0.50,
            agent_robustness: 0.96,
        },
    );

    let info_weighted = ValueConfig {
        goal_progress: 0.20,
        information_gain: 0.35,
        risk_avoidance: 0.15,
        resource_efficiency: 0.10,
        agent_robustness: 0.20,
    };
    let info_scored =
        ValueScorer::new(info_weighted.clone()).score_branches(vec![
            epistemic.clone(),
            instrumental.clone(),
            regulative.clone(),
        ]);

    assert_eq!(info_scored.len(), 3);
    assert_eq!(
        info_scored[0].projected.weight_snapshot,
        info_weighted,
        "projected score should carry the exact runtime weight snapshot",
    );
    assert_eq!(info_scored[0].branch.candidate.kind, ActionKind::Epistemic);
    assert_eq!(
        info_scored[1].branch.candidate.kind,
        ActionKind::Instrumental
    );
    assert_eq!(info_scored[2].branch.candidate.kind, ActionKind::Regulative);

    approx_eq(
        info_scored[0].projected.final_score,
        (0.45 * 0.20) + (0.95 * 0.35) + (0.85 * 0.15) + (0.60 * 0.10) + (0.80 * 0.20),
    );
    approx_eq(
        info_scored[1].projected.final_score,
        (0.95 * 0.20) + (0.35 * 0.35) + (0.55 * 0.15) + (0.80 * 0.10) + (0.65 * 0.20),
    );
    approx_eq(
        info_scored[2].projected.final_score,
        (0.55 * 0.20) + (0.40 * 0.35) + (0.98 * 0.15) + (0.50 * 0.10) + (0.96 * 0.20),
    );

    let goal_weighted = ValueConfig {
        goal_progress: 0.50,
        information_gain: 0.10,
        risk_avoidance: 0.15,
        resource_efficiency: 0.15,
        agent_robustness: 0.10,
    };
    let goal_scored = ValueScorer::new(goal_weighted.clone()).score_branches(vec![
        epistemic,
        instrumental,
        regulative,
    ]);

    assert_eq!(goal_scored[0].projected.weight_snapshot, goal_weighted);

    assert!(
        info_scored[0].projected.final_score > info_scored[1].projected.final_score,
        "information-heavy weighting should favor epistemic branches",
    );
    assert!(
        goal_scored[1].projected.final_score > goal_scored[0].projected.final_score,
        "goal-heavy weighting should favor instrumental branches",
    );
    assert!(
        goal_scored[2].projected.final_score > goal_scored[0].projected.final_score,
        "regulative branches must stay comparable on the same scoring surface",
    );
}

#[test]
fn metacog_gates_warn_veto_and_escalate_with_typed_reports() {
    let scorer = ValueScorer::default();
    let service = MetacognitionService::default();

    let warning_branch = sample_branch(ActionKind::Instrumental, "apply the patch immediately");
    let warning_memory = sample_working_memory(vec![warning_branch.clone()], Vec::new());
    let warning_scored = scorer.score_branches(vec![BranchValueInput::new(
        warning_branch,
        ValueVector {
            goal_progress: 0.90,
            information_gain: 0.20,
            risk_avoidance: 0.55,
            resource_efficiency: 0.80,
            agent_robustness: 0.65,
        },
    )]);
    let warning_report: DecisionReport = service.evaluate(&warning_memory, warning_scored);

    assert_eq!(warning_report.gate.decision, GateDecision::Warning);
    assert_eq!(
        warning_report
            .selected_branch
            .as_ref()
            .expect("warnings should preserve a selected branch")
            .branch
            .candidate
            .kind,
        ActionKind::Instrumental
    );
    assert!(
        warning_report
            .active_risks
            .iter()
            .any(|risk| risk.contains("under-supported")),
        "warning should enrich active risks instead of blocking output",
    );
    assert!(
        warning_report
            .metacog_flags
            .iter()
            .any(|flag| flag.code == "warning_under_supported"),
        "warning should inject a typed metacognitive flag",
    );

    let soft_branch = sample_branch(ActionKind::Instrumental, "ship the irreversible change")
        .with_risk_marker("clarification_required");
    let regulate_branch = sample_branch(ActionKind::Regulative, "pause and request clarification");
    let soft_memory = sample_working_memory(
        vec![soft_branch.clone(), regulate_branch.clone()],
        Vec::new(),
    );
    let soft_scored = scorer.score_branches(vec![
        BranchValueInput::new(
            soft_branch,
            ValueVector {
                goal_progress: 0.95,
                information_gain: 0.25,
                risk_avoidance: 0.20,
                resource_efficiency: 0.85,
                agent_robustness: 0.45,
            },
        ),
        BranchValueInput::new(
            regulate_branch,
            ValueVector {
                goal_progress: 0.35,
                information_gain: 0.35,
                risk_avoidance: 0.95,
                resource_efficiency: 0.45,
                agent_robustness: 0.98,
            },
        ),
    ]);
    let soft_report = service.evaluate(&soft_memory, soft_scored);

    assert_eq!(soft_report.gate.decision, GateDecision::SoftVeto);
    assert_eq!(
        soft_report
            .gate
            .rejected_branch
            .as_ref()
            .expect("soft veto should record the rejected branch")
            .branch
            .candidate
            .kind,
        ActionKind::Instrumental
    );
    assert_eq!(
        soft_report
            .selected_branch
            .as_ref()
            .expect("soft veto should force a regulative alternative")
            .branch
            .candidate
            .kind,
        ActionKind::Regulative
    );
    assert!(
        soft_report.gate.regulative_branch.is_some(),
        "soft veto should keep the forced regulative path in the report",
    );

    let hard_branch =
        sample_branch(ActionKind::Instrumental, "run the destructive migration now")
            .with_risk_marker("unsafe_action");
    let hard_memory = sample_working_memory(vec![hard_branch.clone()], Vec::new());
    let hard_scored = scorer.score_branches(vec![BranchValueInput::new(
        hard_branch,
        ValueVector {
            goal_progress: 0.95,
            information_gain: 0.10,
            risk_avoidance: 0.05,
            resource_efficiency: 0.90,
            agent_robustness: 0.10,
        },
    )]);
    let hard_report = service.evaluate(&hard_memory, hard_scored);

    assert_eq!(hard_report.gate.decision, GateDecision::HardVeto);
    assert!(hard_report.selected_branch.is_none());
    assert_eq!(
        hard_report.gate.safe_response.as_deref(),
        Some("pause execution and request a safer alternative"),
    );

    let escalate_branch = sample_branch(ActionKind::Epistemic, "keep searching without review");
    let escalate_memory = sample_working_memory(
        vec![escalate_branch.clone()],
        vec![MetacognitiveFlag {
            code: "human_review_required".to_string(),
            detail: Some("operator must approve the next step".to_string()),
        }],
    );
    let escalate_scored = scorer.score_branches(vec![BranchValueInput::new(
        escalate_branch,
        ValueVector {
            goal_progress: 0.50,
            information_gain: 0.90,
            risk_avoidance: 0.70,
            resource_efficiency: 0.60,
            agent_robustness: 0.80,
        },
    )]);
    let escalate_report = service.evaluate(&escalate_memory, escalate_scored);

    assert_eq!(escalate_report.gate.decision, GateDecision::Escalate);
    assert!(escalate_report.selected_branch.is_none());
    assert!(escalate_report.gate.autonomy_paused);
    assert!(
        escalate_report
            .gate
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("human_review_required")),
        "escalation diagnostics should preserve the triggering flag",
    );
}
