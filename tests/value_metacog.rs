use agent_memos::cognition::{
    action::{ActionBranch, ActionCandidate, ActionKind},
    value::{BranchValueInput, ValueConfig, ValueScorer, ValueVector},
};

fn sample_branch(kind: ActionKind, summary: &str) -> ActionBranch {
    ActionBranch::new(ActionCandidate::new(kind, summary))
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
