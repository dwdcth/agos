use serde::{Deserialize, Serialize};

use crate::cognition::{action::ActionBranch, working_memory::MetacognitiveFlag};

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

/// Configurable per-dimension floor for non-compensatory threshold gates.
///
/// If any dimension's value is below its threshold, the projected score becomes
/// 0.0 regardless of how high the weighted sum would be. Dimensions without a
/// threshold (floor = 0.0) are unconstrained.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValueThresholds {
    pub goal_progress: f32,
    pub information_gain: f32,
    pub risk_avoidance: f32,
    pub resource_efficiency: f32,
    pub agent_robustness: f32,
}

impl Default for ValueThresholds {
    fn default() -> Self {
        Self {
            goal_progress: 0.0,
            information_gain: 0.0,
            risk_avoidance: 0.15,
            resource_efficiency: 0.0,
            agent_robustness: 0.0,
        }
    }
}

impl ValueThresholds {
    /// All thresholds set to 0.0 (no gating).
    pub fn none() -> Self {
        Self {
            goal_progress: 0.0,
            information_gain: 0.0,
            risk_avoidance: 0.0,
            resource_efficiency: 0.0,
            agent_robustness: 0.0,
        }
    }

    /// Returns `true` if every dimension meets or exceeds its threshold.
    /// Returns `false` if ANY dimension is below its threshold.
    pub fn check(&self, value: &ValueVector) -> bool {
        value.goal_progress >= self.goal_progress
            && value.information_gain >= self.information_gain
            && value.risk_avoidance >= self.risk_avoidance
            && value.resource_efficiency >= self.resource_efficiency
            && value.agent_robustness >= self.agent_robustness
    }
}

/// Ephemeral per-request signed deltas applied on top of the learned baseline.
///
/// Derived from the current request context (goal, risks, metacog flags,
/// readiness/capability flags) and NOT persisted.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DynamicWeightDelta {
    pub goal_progress: f32,
    pub information_gain: f32,
    pub risk_avoidance: f32,
    pub resource_efficiency: f32,
    pub agent_robustness: f32,
}

impl DynamicWeightDelta {
    /// Zero deltas (no adjustment to baseline weights).
    pub fn zero() -> Self {
        Self::default()
    }

    /// Apply these deltas to a `ValueConfig`, producing a new config with
    /// clamped and renormalized weights.
    ///
    /// When all deltas are zero, returns the config unchanged (avoids floating
    /// point drift through the clamp-renormalize path).
    pub fn apply_to(&self, config: &ValueConfig) -> ValueConfig {
        if *self == Self::zero() {
            return config.clone();
        }

        let mut effective = ValueConfig {
            goal_progress: config.goal_progress + self.goal_progress,
            information_gain: config.information_gain + self.information_gain,
            risk_avoidance: config.risk_avoidance + self.risk_avoidance,
            resource_efficiency: config.resource_efficiency + self.resource_efficiency,
            agent_robustness: config.agent_robustness + self.agent_robustness,
        };
        effective.clamp_weights();
        effective.renormalize();
        effective
    }
}

/// Derive a `DynamicWeightDelta` from the current request context.
///
/// The delta is ephemeral (not persisted) and adjusts weights based on:
/// - **DW_task**: from active_goal presence and content
/// - **DW_self**: from readiness/capability flags
/// - **DW_metacog**: from metacognitive flags
/// - **DW_risk**: from active risks
pub fn derive_dynamic_delta(
    active_goal: Option<&str>,
    active_risks: &[String],
    metacog_flags: &[MetacognitiveFlag],
    readiness_flags: &[String],
    capability_flags: &[String],
) -> DynamicWeightDelta {
    let mut delta = DynamicWeightDelta::zero();

    // DW_task: from active_goal content
    if let Some(goal) = active_goal {
        let goal_lower = goal.to_lowercase();
        if goal_lower.contains("explore")
            || goal_lower.contains("investigate")
            || goal_lower.contains("understand")
        {
            delta.information_gain += 0.05;
        }
        if goal_lower.contains("deliver")
            || goal_lower.contains("execute")
            || goal_lower.contains("implement")
        {
            delta.goal_progress += 0.05;
        }
    }

    // DW_self: from readiness/capability flags
    let low_readiness = readiness_flags
        .iter()
        .any(|f| f.contains("low") || f.contains("degraded") || f.contains("unavailable"));
    if low_readiness {
        delta.agent_robustness += 0.04;
        delta.resource_efficiency -= 0.02;
    }

    let sparse_capabilities = capability_flags.len() <= 1;
    if sparse_capabilities {
        delta.information_gain += 0.03;
    }

    // DW_metacog: from metacognitive flags
    for flag in metacog_flags {
        match flag.code.as_str() {
            "warning_under_supported" | "warning" => {
                delta.risk_avoidance += 0.03;
            }
            "soft_veto_active" | "soft_veto" => {
                delta.risk_avoidance += 0.06;
                delta.information_gain += 0.04;
            }
            "human_review_required" | "escalate" => {
                delta.risk_avoidance += 0.08;
                delta.agent_robustness += 0.05;
            }
            _ => {}
        }
    }

    // DW_risk: from active risks
    let risk_boost = (active_risks.len() as f32 * 0.02).min(0.06);
    delta.risk_avoidance += risk_boost;

    delta
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
    pub threshold_passed: bool,
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
    thresholds: ValueThresholds,
}

impl ValueScorer {
    pub fn new(config: ValueConfig) -> Self {
        Self {
            config,
            thresholds: ValueThresholds::default(),
        }
    }

    pub fn with_thresholds(mut self, thresholds: ValueThresholds) -> Self {
        self.thresholds = thresholds;
        self
    }

    pub fn config(&self) -> &ValueConfig {
        &self.config
    }

    pub fn thresholds(&self) -> &ValueThresholds {
        &self.thresholds
    }

    pub fn project_with_delta(
        &self,
        value: &ValueVector,
        delta: &DynamicWeightDelta,
    ) -> ProjectedScore {
        let effective_config = delta.apply_to(&self.config);

        debug_assert!(
            effective_config.is_normalized(),
            "effective value weights should stay normalized"
        );
        debug_assert!(
            value
                .all_dimensions()
                .into_iter()
                .all(|dimension| (0.0..=1.0).contains(&dimension)),
            "value dimensions should stay normalized",
        );

        if !self.thresholds.check(value) {
            return ProjectedScore {
                final_score: 0.0,
                weight_snapshot: effective_config,
                threshold_passed: false,
            };
        }

        let final_score = (value.goal_progress * effective_config.goal_progress)
            + (value.information_gain * effective_config.information_gain)
            + (value.risk_avoidance * effective_config.risk_avoidance)
            + (value.resource_efficiency * effective_config.resource_efficiency)
            + (value.agent_robustness * effective_config.agent_robustness);

        ProjectedScore {
            final_score,
            weight_snapshot: effective_config,
            threshold_passed: true,
        }
    }

    pub fn project(&self, value: &ValueVector) -> ProjectedScore {
        self.project_with_delta(value, &DynamicWeightDelta::zero())
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
