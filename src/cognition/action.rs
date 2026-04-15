use serde::Serialize;

use crate::cognition::working_memory::EvidenceFragment;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    Epistemic,
    Instrumental,
    Regulative,
}

impl ActionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Epistemic => "epistemic",
            Self::Instrumental => "instrumental",
            Self::Regulative => "regulative",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "epistemic" => Some(Self::Epistemic),
            "instrumental" => Some(Self::Instrumental),
            "regulative" => Some(Self::Regulative),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ActionCandidate {
    pub kind: ActionKind,
    pub summary: String,
    pub intent: Option<String>,
    pub parameters: Vec<String>,
    pub expected_effects: Vec<String>,
}

impl ActionCandidate {
    pub fn new(kind: ActionKind, summary: impl Into<String>) -> Self {
        Self {
            kind,
            summary: summary.into(),
            intent: None,
            parameters: Vec::new(),
            expected_effects: Vec::new(),
        }
    }

    pub fn with_intent(mut self, intent: impl Into<String>) -> Self {
        self.intent = Some(intent.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ActionBranch {
    pub candidate: ActionCandidate,
    pub supporting_evidence: Vec<EvidenceFragment>,
    pub risk_markers: Vec<String>,
}

impl ActionBranch {
    pub fn new(candidate: ActionCandidate) -> Self {
        Self {
            candidate,
            supporting_evidence: Vec::new(),
            risk_markers: Vec::new(),
        }
    }

    pub fn with_supporting_evidence(mut self, evidence: Vec<EvidenceFragment>) -> Self {
        self.supporting_evidence = evidence;
        self
    }

    pub fn with_risk_marker(mut self, risk_marker: impl Into<String>) -> Self {
        self.risk_markers.push(risk_marker.into());
        self
    }
}
