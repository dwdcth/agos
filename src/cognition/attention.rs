use serde::{Deserialize, Serialize};

/// Where an attention cue originated from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttentionLane {
    Goal,
    Risk,
    Metacog,
    Readiness,
    Capability,
}

/// A single attention cue extracted from request metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttentionCue {
    pub lane: AttentionLane,
    pub source: String,
    pub cue: String,
    pub weight: f32,
}

/// Session-level persistent baseline representing the cognitive situation context.
///
/// Dimensions are f32 in [0.0, 1.0], defaulting to 0.5 (neutral).
/// The baseline is updated via smooth exponential moving average after each assembly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttentionBaseline {
    pub time_pressure: f32,
    pub cognitive_load: f32,
    pub uncertainty_level: f32,
    pub exploration_mode: f32,
}

impl Default for AttentionBaseline {
    fn default() -> Self {
        Self {
            time_pressure: 0.5,
            cognitive_load: 0.5,
            uncertainty_level: 0.5,
            exploration_mode: 0.5,
        }
    }
}

impl AttentionBaseline {
    /// Smooth baseline update: `Baseline(t+1) = Baseline(t) + eta * (Observed - Baseline(t))`
    ///
    /// Each dimension is independently updated and clamped to [0.0, 1.0].
    pub fn update(&self, observed: &AttentionBaseline, learning_rate: f32) -> AttentionBaseline {
        let clamp = |v: f32| v.clamp(0.0, 1.0);
        AttentionBaseline {
            time_pressure: clamp(
                self.time_pressure + learning_rate * (observed.time_pressure - self.time_pressure),
            ),
            cognitive_load: clamp(
                self.cognitive_load
                    + learning_rate * (observed.cognitive_load - self.cognitive_load),
            ),
            uncertainty_level: clamp(
                self.uncertainty_level
                    + learning_rate * (observed.uncertainty_level - self.uncertainty_level),
            ),
            exploration_mode: clamp(
                self.exploration_mode
                    + learning_rate * (observed.exploration_mode - self.exploration_mode),
            ),
        }
    }

    /// Return the baseline as a 4-element vector in dimension order.
    pub fn as_vector(&self) -> [f32; 4] {
        [
            self.time_pressure,
            self.cognitive_load,
            self.uncertainty_level,
            self.exploration_mode,
        ]
    }
}

/// Predefined emotion profiles that define per-dimension mask vectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmotionProfile {
    Neutral,
    Cautious,
    Curious,
    Urgent,
}

impl EmotionProfile {
    /// Return the per-dimension mask for this profile.
    pub fn mask(&self) -> [f32; 4] {
        match self {
            EmotionProfile::Neutral => [0.0, 0.0, 0.0, 0.0],
            // Cautious: boost uncertainty, suppress exploration
            EmotionProfile::Cautious => [-0.1, 0.1, 0.3, -0.2],
            // Curious: boost exploration, suppress time_pressure
            EmotionProfile::Curious => [-0.2, -0.1, -0.1, 0.3],
            // Urgent: boost time_pressure, suppress exploration
            EmotionProfile::Urgent => [0.3, 0.1, 0.0, -0.3],
        }
    }

    /// Return the default intensity for this profile.
    pub fn default_intensity(&self) -> f32 {
        match self {
            EmotionProfile::Neutral => 0.0,
            EmotionProfile::Cautious => 0.5,
            EmotionProfile::Curious => 0.5,
            EmotionProfile::Urgent => 0.7,
        }
    }
}

/// Multiplicative emotion modulator applied to the ContextBase baseline.
///
/// Formula: `E(c, e, i) = c * (1 + intensity * mask_e)`
/// where `c` is the baseline vector, `i` is intensity, and `M_e` is the profile mask.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmotionModulator {
    pub profile: EmotionProfile,
    pub intensity: f32,
    pub mask: [f32; 4],
}

impl Default for EmotionModulator {
    fn default() -> Self {
        Self::neutral()
    }
}

impl EmotionModulator {
    pub fn neutral() -> Self {
        Self {
            profile: EmotionProfile::Neutral,
            intensity: 0.0,
            mask: EmotionProfile::Neutral.mask(),
        }
    }

    pub fn cautious() -> Self {
        let profile = EmotionProfile::Cautious;
        Self {
            profile,
            intensity: profile.default_intensity(),
            mask: profile.mask(),
        }
    }

    pub fn curious() -> Self {
        let profile = EmotionProfile::Curious;
        Self {
            profile,
            intensity: profile.default_intensity(),
            mask: profile.mask(),
        }
    }

    pub fn urgent() -> Self {
        let profile = EmotionProfile::Urgent;
        Self {
            profile,
            intensity: profile.default_intensity(),
            mask: profile.mask(),
        }
    }

    /// Apply emotion modulation to a baseline.
    ///
    /// Returns `c * (1 + intensity * mask)` for each dimension.
    pub fn modulate(&self, baseline: &AttentionBaseline) -> [f32; 4] {
        let c = baseline.as_vector();
        [
            c[0] * (1.0 + self.intensity * self.mask[0]),
            c[1] * (1.0 + self.intensity * self.mask[1]),
            c[2] * (1.0 + self.intensity * self.mask[2]),
            c[3] * (1.0 + self.intensity * self.mask[3]),
        ]
    }

    /// Returns true when this modulator has no effect (neutral profile / zero intensity).
    pub fn is_neutral(&self) -> bool {
        self.profile == EmotionProfile::Neutral || self.intensity == 0.0
    }
}

/// A single inhibition constraint derived from self-model limitations.
///
/// When a candidate's fields match the constraint pattern, the weight is summed
/// as a negative penalty during attention scoring.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InhibitionConstraint {
    pub source: String,
    pub pattern: String,
    pub weight: f32,
}

/// Compute inhibition penalty for a candidate by matching constraints against
/// candidate fields (label, content, dsl_claim).
///
/// Returns the sum of matched constraint weights (always >= 0.0).
/// The caller subtracts this from the positive attention bonus.
pub fn compute_inhibition(
    constraints: &[InhibitionConstraint],
    candidate_label: &str,
    candidate_content: &str,
    candidate_dsl_claim: Option<&str>,
) -> f32 {
    let label_lower = candidate_label.to_lowercase();
    let content_lower = candidate_content.to_lowercase();
    let dsl_claim_lower = candidate_dsl_claim
        .map(|c| c.to_lowercase())
        .unwrap_or_default();

    let mut total: f32 = 0.0;
    for constraint in constraints {
        let pattern_lower = constraint.pattern.to_lowercase();
        let pattern_terms = pattern_lower
            .split(|ch: char| ch.is_ascii_punctuation() || ch.is_whitespace())
            .filter(|term| !term.is_empty())
            .collect::<Vec<_>>();

        if pattern_terms.is_empty() {
            continue;
        }

        let matched = pattern_terms.iter().all(|term| {
            (!label_lower.is_empty() && label_lower.contains(term))
                || content_lower.contains(term)
                || (!dsl_claim_lower.is_empty() && dsl_claim_lower.contains(term))
        });

        if matched {
            total += constraint.weight;
        }
    }

    total
}

/// Metacognitive modifier that adjusts attention scoring parameters based on
/// active metacognitive flags.
///
/// - `goal_weight_multiplier`: scales the goal lane weight (default 1.0)
/// - `diversity_temperature`: controls cue sensitivity spread (default 1.0)
/// - `inhibition_strength`: scales the inhibition penalty (default 1.0)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetacogModifier {
    pub goal_weight_multiplier: f32,
    pub diversity_temperature: f32,
    pub inhibition_strength: f32,
}

impl Default for MetacogModifier {
    fn default() -> Self {
        Self::none()
    }
}

impl MetacogModifier {
    /// No-op modifier: all multipliers at identity values.
    pub fn none() -> Self {
        Self {
            goal_weight_multiplier: 1.0,
            diversity_temperature: 1.0,
            inhibition_strength: 1.0,
        }
    }

    /// Derive a modifier from active metacognitive flags.
    ///
    /// - `Warning` code -> slight risk boost (increased goal weight, moderate inhibition)
    /// - `SoftVeto` / `soft_veto_active` -> strong risk boost (increased inhibition)
    pub fn from_flags(flags: &[crate::cognition::working_memory::MetacognitiveFlag]) -> Self {
        let has_warning = flags
            .iter()
            .any(|f| f.code.contains("warning") || f.code.contains("Warning"));
        let has_soft_veto = flags
            .iter()
            .any(|f| f.code.contains("soft_veto") || f.code.contains("SoftVeto"));

        match (has_warning, has_soft_veto) {
            (false, false) => Self::none(),
            (true, false) => Self {
                goal_weight_multiplier: 1.1,
                diversity_temperature: 1.0,
                inhibition_strength: 1.2,
            },
            (_, true) => Self {
                goal_weight_multiplier: 1.0,
                diversity_temperature: 0.9,
                inhibition_strength: 1.5,
            },
        }
    }

    /// Returns true when this modifier has no effect.
    pub fn is_none(&self) -> bool {
        self.goal_weight_multiplier == 1.0
            && self.diversity_temperature == 1.0
            && self.inhibition_strength == 1.0
    }
}

/// A single contribution to the attention bonus for a candidate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttentionContribution {
    pub lane: AttentionLane,
    pub source: String,
    pub cue: String,
    pub matched_fields: Vec<String>,
    pub bonus: f32,
}

/// Transient bias derived from request metadata.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AttentionDelta {
    pub total_bonus: f32,
    pub contributions: Vec<AttentionContribution>,
}

/// Dual-timescale attention state with full modulation architecture.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AttentionState {
    pub baseline: AttentionBaseline,
    pub emotion: EmotionModulator,
    pub metacog_modifier: MetacogModifier,
    pub delta: AttentionDelta,
    pub inhibition_constraints: Vec<InhibitionConstraint>,
}

impl AttentionState {
    /// Returns true when this state carries no active delta cues.
    pub fn is_empty(&self) -> bool {
        self.delta.contributions.is_empty() && self.delta.total_bonus == 0.0
    }

    /// Derive an [`AttentionDelta`] from request metadata fields.
    ///
    /// Each non-empty metadata field becomes one or more [`AttentionCue`] values
    /// with lane-specific weights.
    pub fn derive_delta(
        active_goal: Option<&str>,
        active_risks: &[String],
        metacog_flags: &[crate::cognition::working_memory::MetacognitiveFlag],
        readiness_flags: &[String],
        capability_flags: &[String],
    ) -> AttentionDelta {
        let mut contributions = Vec::new();

        if let Some(goal) = active_goal {
            let lowered = goal.trim().to_lowercase();
            if !lowered.is_empty() {
                contributions.push(AttentionContribution {
                    lane: AttentionLane::Goal,
                    source: "active_goal".to_string(),
                    cue: lowered,
                    matched_fields: Vec::new(),
                    bonus: 0.0,
                });
            }
        }

        for risk in active_risks {
            let lowered = risk.trim().to_lowercase();
            if !lowered.is_empty() {
                contributions.push(AttentionContribution {
                    lane: AttentionLane::Risk,
                    source: "active_risks".to_string(),
                    cue: lowered,
                    matched_fields: Vec::new(),
                    bonus: 0.0,
                });
            }
        }

        for flag in metacog_flags {
            let lowered = flag.code.trim().to_lowercase();
            if !lowered.is_empty() {
                contributions.push(AttentionContribution {
                    lane: AttentionLane::Metacog,
                    source: "metacog_flags".to_string(),
                    cue: lowered,
                    matched_fields: Vec::new(),
                    bonus: 0.0,
                });
            }
        }

        for flag in readiness_flags {
            let lowered = flag.trim().to_lowercase();
            if !lowered.is_empty() {
                contributions.push(AttentionContribution {
                    lane: AttentionLane::Readiness,
                    source: "readiness_flags".to_string(),
                    cue: lowered,
                    matched_fields: Vec::new(),
                    bonus: 0.0,
                });
            }
        }

        for flag in capability_flags {
            let lowered = flag.trim().to_lowercase();
            if !lowered.is_empty() {
                contributions.push(AttentionContribution {
                    lane: AttentionLane::Capability,
                    source: "capability_flags".to_string(),
                    cue: lowered,
                    matched_fields: Vec::new(),
                    bonus: 0.0,
                });
            }
        }

        AttentionDelta {
            total_bonus: 0.0,
            contributions,
        }
    }

    /// Derive inhibition constraints from capability and readiness flags.
    ///
    /// Capability and readiness flags represent self-model constraints that
    /// can inhibit attention toward candidates that match those constraints.
    pub fn derive_inhibition_constraints(
        capability_flags: &[String],
        readiness_flags: &[String],
    ) -> Vec<InhibitionConstraint> {
        let mut constraints = Vec::new();

        for flag in capability_flags {
            let lowered = flag.trim().to_lowercase();
            if !lowered.is_empty() {
                constraints.push(InhibitionConstraint {
                    source: "capability_flag".to_string(),
                    pattern: lowered,
                    weight: CAPABILITY_INHIBITION_WEIGHT,
                });
            }
        }

        for flag in readiness_flags {
            let lowered = flag.trim().to_lowercase();
            if !lowered.is_empty() {
                constraints.push(InhibitionConstraint {
                    source: "readiness_flag".to_string(),
                    pattern: lowered,
                    weight: READINESS_INHIBITION_WEIGHT,
                });
            }
        }

        constraints
    }
}

/// Trace output for explainability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttentionTrace {
    pub total_bonus: f32,
    pub contributions: Vec<AttentionContribution>,
}

/// Per-lane attention weight used during scoring.
pub(crate) const GOAL_WEIGHT: f32 = 0.06;
pub(crate) const RISK_WEIGHT: f32 = 0.04;
pub(crate) const METACOG_WEIGHT: f32 = 0.03;
pub(crate) const READINESS_WEIGHT: f32 = 0.02;
pub(crate) const CAPABILITY_WEIGHT: f32 = 0.02;

/// Inhibition weights for self-model constraint penalties.
pub(crate) const CAPABILITY_INHIBITION_WEIGHT: f32 = 0.01;
pub(crate) const READINESS_INHIBITION_WEIGHT: f32 = 0.01;

/// Maximum total attention bonus that can be applied to a single candidate.
pub const ATTENTION_BONUS_CAP: f32 = 0.15;

/// Default learning rate for baseline smooth update.
pub const BASELINE_LEARNING_RATE: f32 = 0.1;

/// Return the default weight for a given lane.
pub(crate) fn lane_weight(lane: AttentionLane) -> f32 {
    match lane {
        AttentionLane::Goal => GOAL_WEIGHT,
        AttentionLane::Risk => RISK_WEIGHT,
        AttentionLane::Metacog => METACOG_WEIGHT,
        AttentionLane::Readiness => READINESS_WEIGHT,
        AttentionLane::Capability => CAPABILITY_WEIGHT,
    }
}
