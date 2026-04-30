use serde::Serialize;

/// Where an attention cue originated from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AttentionLane {
    Goal,
    Risk,
    Metacog,
    Readiness,
    Capability,
}

/// A single attention cue extracted from request metadata.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AttentionCue {
    pub lane: AttentionLane,
    pub source: String,
    pub cue: String,
    pub weight: f32,
}

/// Session-level persistent state (MVP: default/empty).
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct AttentionBaseline;

/// A single contribution to the attention bonus for a candidate.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AttentionContribution {
    pub lane: AttentionLane,
    pub source: String,
    pub cue: String,
    pub matched_fields: Vec<String>,
    pub bonus: f32,
}

/// Transient bias derived from request metadata.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct AttentionDelta {
    pub total_bonus: f32,
    pub contributions: Vec<AttentionContribution>,
}

/// Dual-timescale attention state.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct AttentionState {
    pub baseline: AttentionBaseline,
    pub delta: AttentionDelta,
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
}

/// Trace output for explainability.
#[derive(Debug, Clone, PartialEq, Serialize)]
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

/// Maximum total attention bonus that can be applied to a single candidate.
pub const ATTENTION_BONUS_CAP: f32 = 0.15;

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
