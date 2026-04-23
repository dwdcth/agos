use serde::Serialize;
use thiserror::Error;

use crate::{
    cognition::action::ActionBranch,
    memory::{
        dsl::FlatFactDslRecordV1,
        record::{Provenance, TruthLayer},
        truth::{T3State, TruthRecord},
    },
    search::{Citation, ResultTrace, ScoreBreakdown},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SelfStateFact {
    pub key: String,
    pub value: String,
    pub source_record_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SelfStateSnapshot {
    pub task_context: Option<String>,
    pub capability_flags: Vec<String>,
    pub readiness_flags: Vec<String>,
    pub facts: Vec<SelfStateFact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ActiveGoal {
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MetacognitiveFlag {
    pub code: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TruthContext {
    pub truth_layer: TruthLayer,
    pub t3_state: Option<T3State>,
    pub open_review_ids: Vec<String>,
    pub open_candidate_ids: Vec<String>,
}

impl TruthContext {
    pub fn from_truth_record(record: &TruthRecord) -> Self {
        match record {
            TruthRecord::T1 { base } => Self {
                truth_layer: base.truth_layer,
                t3_state: None,
                open_review_ids: Vec::new(),
                open_candidate_ids: Vec::new(),
            },
            TruthRecord::T2 {
                base,
                open_candidates,
            } => Self {
                truth_layer: base.truth_layer,
                t3_state: None,
                open_review_ids: Vec::new(),
                open_candidate_ids: open_candidates
                    .iter()
                    .map(|candidate| candidate.candidate_id.clone())
                    .collect(),
            },
            TruthRecord::T3 {
                base,
                t3_state,
                open_reviews,
            } => Self {
                truth_layer: base.truth_layer,
                t3_state: t3_state.clone(),
                open_review_ids: open_reviews
                    .iter()
                    .map(|review| review.review_id.clone())
                    .collect(),
                open_candidate_ids: Vec::new(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EvidenceFragment {
    pub record_id: String,
    pub snippet: String,
    pub citation: Citation,
    pub provenance: Provenance,
    pub truth_context: TruthContext,
    pub dsl: Option<FlatFactDslRecordV1>,
    pub trace: ResultTrace,
    pub score: ScoreBreakdown,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PresentFrame {
    pub world_fragments: Vec<EvidenceFragment>,
    pub self_state: SelfStateSnapshot,
    pub active_goal: Option<ActiveGoal>,
    pub active_risks: Vec<String>,
    pub metacog_flags: Vec<MetacognitiveFlag>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WorkingMemory {
    pub present: PresentFrame,
    pub branches: Vec<ActionBranch>,
}

impl WorkingMemory {
    pub fn builder() -> WorkingMemoryBuilder {
        WorkingMemoryBuilder::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorkingMemoryBuildError {
    #[error("working memory requires a present frame before it can be built")]
    MissingPresentFrame,
}

#[derive(Debug, Clone, Default)]
pub struct WorkingMemoryBuilder {
    present: Option<PresentFrame>,
    branches: Vec<ActionBranch>,
}

impl WorkingMemoryBuilder {
    pub fn present(mut self, present: PresentFrame) -> Self {
        self.present = Some(present);
        self
    }

    pub fn push_branch(mut self, branch: ActionBranch) -> Self {
        self.branches.push(branch);
        self
    }

    pub fn extend_branches<I>(mut self, branches: I) -> Self
    where
        I: IntoIterator<Item = ActionBranch>,
    {
        self.branches.extend(branches);
        self
    }

    pub fn build(self) -> Result<WorkingMemory, WorkingMemoryBuildError> {
        let Some(present) = self.present else {
            return Err(WorkingMemoryBuildError::MissingPresentFrame);
        };

        Ok(WorkingMemory {
            present,
            branches: self.branches,
        })
    }
}
