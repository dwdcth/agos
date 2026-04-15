use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::memory::record::{MemoryRecord, TruthLayer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum T3Confidence {
    Low,
    Medium,
    High,
}

impl T3Confidence {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "low" => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high" => Some(Self::High),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum T3RevocationState {
    Active,
    Revoked,
}

impl T3RevocationState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Revoked => "revoked",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "revoked" => Some(Self::Revoked),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct T3State {
    pub record_id: String,
    pub confidence: T3Confidence,
    pub revocation_state: T3RevocationState,
    pub revoked_at: Option<String>,
    pub revocation_reason: Option<String>,
    pub shared_conflict_note: Option<String>,
    pub last_reviewed_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewGateState {
    Pending,
    Passed,
    Rejected,
}

impl ReviewGateState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Passed => "passed",
            Self::Rejected => "rejected",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "passed" => Some(Self::Passed),
            "rejected" => Some(Self::Rejected),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromotionDecisionState {
    Pending,
    Approved,
    Rejected,
    Cancelled,
}

impl PromotionDecisionState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "approved" => Some(Self::Approved),
            "rejected" => Some(Self::Rejected),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionReview {
    pub review_id: String,
    pub source_record_id: String,
    pub target_layer: TruthLayer,
    pub result_trigger_state: ReviewGateState,
    pub evidence_review_state: ReviewGateState,
    pub consensus_check_state: ReviewGateState,
    pub metacog_approval_state: ReviewGateState,
    pub decision_state: PromotionDecisionState,
    pub review_notes: Option<Value>,
    pub approved_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRole {
    Supporting,
    Contradicting,
    Context,
}

impl EvidenceRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Supporting => "supporting",
            Self::Contradicting => "contradicting",
            Self::Context => "context",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "supporting" => Some(Self::Supporting),
            "contradicting" => Some(Self::Contradicting),
            "context" => Some(Self::Context),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionEvidence {
    pub review_id: String,
    pub evidence_record_id: String,
    pub evidence_role: EvidenceRole,
    pub evidence_note: Option<Value>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateReviewState {
    Pending,
    Passed,
    Rejected,
}

impl CandidateReviewState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Passed => "passed",
            Self::Rejected => "rejected",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "passed" => Some(Self::Passed),
            "rejected" => Some(Self::Rejected),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OntologyCandidateState {
    Pending,
    Accepted,
    Rejected,
    Withdrawn,
}

impl OntologyCandidateState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "accepted" => Some(Self::Accepted),
            "rejected" => Some(Self::Rejected),
            "withdrawn" => Some(Self::Withdrawn),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OntologyCandidate {
    pub candidate_id: String,
    pub source_record_id: String,
    pub basis_record_ids: Vec<String>,
    pub proposed_structure: Value,
    pub time_stability_state: CandidateReviewState,
    pub agent_reproducibility_state: CandidateReviewState,
    pub context_invariance_state: CandidateReviewState,
    pub predictive_utility_state: CandidateReviewState,
    pub structural_review_state: CandidateReviewState,
    pub candidate_state: OntologyCandidateState,
    pub decided_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TruthRecord {
    T1 {
        base: MemoryRecord,
    },
    T2 {
        base: MemoryRecord,
    },
    T3 {
        base: MemoryRecord,
        t3_state: Option<T3State>,
    },
}

impl TruthRecord {
    pub fn record(&self) -> &MemoryRecord {
        match self {
            Self::T1 { base } | Self::T2 { base } | Self::T3 { base, .. } => base,
        }
    }

    pub fn truth_layer(&self) -> TruthLayer {
        self.record().truth_layer
    }
}
