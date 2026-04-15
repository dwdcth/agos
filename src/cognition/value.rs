use serde::Serialize;

use crate::cognition::action::ActionBranch;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ValueVector {
    pub goal_progress: f32,
    pub information_gain: f32,
    pub risk_avoidance: f32,
    pub resource_efficiency: f32,
    pub agent_robustness: f32,
}

impl ValueVector {
    fn all_dimensions(&self) -> [f32; 5] {
        [
            self.goal_progress,
            self.information_gain,
            self.risk_avoidance,
            self.resource_efficiency,
            self.agent_robustness,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ValueConfig {
    pub goal_progress: f32,
    pub information_gain: f32,
    pub risk_avoidance: f32,
    pub resource_efficiency: f32,
    pub agent_robustness: f32,
}

impl Default for ValueConfig {
    fn default() -> Self {
        Self {
            goal_progress: 0.25,
            information_gain: 0.20,
            risk_avoidance: 0.20,
            resource_efficiency: 0.15,
            agent_robustness: 0.20,
        }
    }
}

impl ValueConfig {
    fn all_weights(&self) -> [f32; 5] {
        [
            self.goal_progress,
            self.information_gain,
            self.risk_avoidance,
            self.resource_efficiency,
            self.agent_robustness,
        ]
    }

    fn is_normalized(&self) -> bool {
        self.all_weights()
            .into_iter()
            .all(|weight| (0.0..=1.0).contains(&weight))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProjectedScore {
    pub final_score: f32,
    pub weight_snapshot: ValueConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BranchValueInput {
    pub branch: ActionBranch,
    pub value: ValueVector,
}

impl BranchValueInput {
    pub fn new(branch: ActionBranch, value: ValueVector) -> Self {
        Self { branch, value }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScoredBranch {
    pub branch: ActionBranch,
    pub value: ValueVector,
    pub projected: ProjectedScore,
}

#[derive(Debug, Clone, Default)]
pub struct ValueScorer {
    config: ValueConfig,
}

impl ValueScorer {
    pub fn new(config: ValueConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &ValueConfig {
        &self.config
    }

    pub fn project(&self, value: &ValueVector) -> ProjectedScore {
        debug_assert!(
            self.config.is_normalized(),
            "value weights should stay normalized"
        );
        debug_assert!(
            value
                .all_dimensions()
                .into_iter()
                .all(|dimension| (0.0..=1.0).contains(&dimension)),
            "value dimensions should stay normalized",
        );

        let final_score = (value.goal_progress * self.config.goal_progress)
            + (value.information_gain * self.config.information_gain)
            + (value.risk_avoidance * self.config.risk_avoidance)
            + (value.resource_efficiency * self.config.resource_efficiency)
            + (value.agent_robustness * self.config.agent_robustness);

        ProjectedScore {
            final_score,
            weight_snapshot: self.config.clone(),
        }
    }

    pub fn score_branch(&self, input: BranchValueInput) -> ScoredBranch {
        let projected = self.project(&input.value);
        ScoredBranch {
            branch: input.branch,
            value: input.value,
            projected,
        }
    }

    pub fn score_branches<I>(&self, inputs: I) -> Vec<ScoredBranch>
    where
        I: IntoIterator<Item = BranchValueInput>,
    {
        inputs.into_iter().map(|input| self.score_branch(input)).collect()
    }
}
