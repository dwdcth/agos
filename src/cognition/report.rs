use serde::Serialize;

use crate::cognition::{
    action::ActionBranch,
    metacog::GateDecision,
    value::{ProjectedScore, ScoredBranch, ValueVector},
    working_memory::MetacognitiveFlag,
};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScoredBranchReport {
    pub branch: ActionBranch,
    pub value: ValueVector,
    pub projected: ProjectedScore,
}

impl From<ScoredBranch> for ScoredBranchReport {
    fn from(value: ScoredBranch) -> Self {
        Self {
            branch: value.branch,
            value: value.value,
            projected: value.projected,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GateReport {
    pub decision: GateDecision,
    pub diagnostics: Vec<String>,
    pub rejected_branch: Option<ScoredBranchReport>,
    pub regulative_branch: Option<ScoredBranchReport>,
    pub safe_response: Option<String>,
    pub autonomy_paused: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DecisionReport {
    pub scored_branches: Vec<ScoredBranchReport>,
    pub selected_branch: Option<ScoredBranchReport>,
    pub gate: GateReport,
    pub active_risks: Vec<String>,
    pub metacog_flags: Vec<MetacognitiveFlag>,
}
