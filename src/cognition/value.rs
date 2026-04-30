use serde::{Deserialize, Serialize};

use crate::cognition::action::ActionBranch;

/// Signed delta per dimension for value weight adjustment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValueAdjustment {
    pub goal_progress: f32,
    pub information_gain: f32,
    pub risk_avoidance: f32,
    pub resource_efficiency: f32,
    pub agent_robustness: f32,
}

impl ValueAdjustment {
    pub fn zero() -> Self {
        Self {
            goal_progress: 0.0,
            information_gain: 0.0,
            risk_avoidance: 0.0,
            resource_efficiency: 0.0,
            agent_robustness: 0.0,
        }
    }

    pub fn with_goal_progress(mut self, delta: f32) -> Self {
        self.goal_progress = delta;
        self
    }

    pub fn with_information_gain(mut self, delta: f32) -> Self {
        self.information_gain = delta;
        self
    }

    pub fn with_risk_avoidance(mut self, delta: f32) -> Self {
        self.risk_avoidance = delta;
        self
    }

    pub fn with_resource_efficiency(mut self, delta: f32) -> Self {
        self.resource_efficiency = delta;
        self
    }

    pub fn with_agent_robustness(mut self, delta: f32) -> Self {
        self.agent_robustness = delta;
        self
    }
}

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
            goal_progress: 0.35,
            information_gain: 0.15,
            risk_avoidance: 0.15,
            resource_efficiency: 0.20,
            agent_robustness: 0.15,
        }
    }
}

impl ValueConfig {
    pub const WEIGHT_FLOOR: f32 = 0.05;
    pub const WEIGHT_CEILING: f32 = 0.60;

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

    /// Apply a single adjustment with learning rate, clamping, and renormalization.
    pub fn apply_adjustment(
        &self,
        adjustment: &ValueAdjustment,
        learning_rate: f32,
    ) -> ValueConfig {
        let mut config = ValueConfig {
            goal_progress: self.goal_progress + (adjustment.goal_progress * learning_rate),
            information_gain: self.information_gain + (adjustment.information_gain * learning_rate),
            risk_avoidance: self.risk_avoidance + (adjustment.risk_avoidance * learning_rate),
            resource_efficiency: self.resource_efficiency
                + (adjustment.resource_efficiency * learning_rate),
            agent_robustness: self.agent_robustness + (adjustment.agent_robustness * learning_rate),
        };

        config.clamp_weights();
        config.renormalize();
        config
    }

    /// Fold multiple adjustments into a base config.
    pub fn from_persisted_adjustments(
        base: &ValueConfig,
        adjustments: &[ValueAdjustment],
        learning_rate: f32,
    ) -> ValueConfig {
        adjustments.iter().fold(base.clone(), |config, adjustment| {
            config.apply_adjustment(adjustment, learning_rate)
        })
    }

    fn clamp_weights(&mut self) {
        self.goal_progress = self
            .goal_progress
            .clamp(Self::WEIGHT_FLOOR, Self::WEIGHT_CEILING);
        self.information_gain = self
            .information_gain
            .clamp(Self::WEIGHT_FLOOR, Self::WEIGHT_CEILING);
        self.risk_avoidance = self
            .risk_avoidance
            .clamp(Self::WEIGHT_FLOOR, Self::WEIGHT_CEILING);
        self.resource_efficiency = self
            .resource_efficiency
            .clamp(Self::WEIGHT_FLOOR, Self::WEIGHT_CEILING);
        self.agent_robustness = self
            .agent_robustness
            .clamp(Self::WEIGHT_FLOOR, Self::WEIGHT_CEILING);
    }

    fn renormalize(&mut self) {
        let sum = self.goal_progress
            + self.information_gain
            + self.risk_avoidance
            + self.resource_efficiency
            + self.agent_robustness;

        if sum > 0.0 {
            self.goal_progress /= sum;
            self.information_gain /= sum;
            self.risk_avoidance /= sum;
            self.resource_efficiency /= sum;
            self.agent_robustness /= sum;
        }
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
        inputs
            .into_iter()
            .map(|input| self.score_branch(input))
            .collect()
    }
}
