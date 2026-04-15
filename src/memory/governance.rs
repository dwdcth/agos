use rusqlite::Connection;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

use crate::memory::{
    record::{MemoryRecord, Provenance, TruthLayer},
    repository::{MemoryRepository, RepositoryError},
    truth::{
        CandidateReviewState, EvidenceRole, OntologyCandidate, OntologyCandidateState,
        PromotionDecisionState, PromotionEvidence, PromotionReview, ReviewGateState,
        T3RevocationState, TruthRecord,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PromotionGate {
    ResultTrigger,
    EvidenceReview,
    ConsensusCheck,
    MetacogApproval,
}

impl PromotionGate {
    pub fn label(self) -> &'static str {
        match self {
            Self::ResultTrigger => "result_trigger",
            Self::EvidenceReview => "evidence_review",
            Self::ConsensusCheck => "consensus_check",
            Self::MetacogApproval => "metacog_approval",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatePromotionReviewRequest {
    pub review_id: String,
    pub source_record_id: String,
    pub created_at: String,
    pub review_notes: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachPromotionEvidenceRequest {
    pub review_id: String,
    pub evidence_record_id: String,
    pub evidence_role: EvidenceRole,
    pub evidence_note: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdatePromotionGateRequest {
    pub review_id: String,
    pub gate: PromotionGate,
    pub state: ReviewGateState,
    pub updated_at: String,
    pub review_notes: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectPromotionRequest {
    pub review_id: String,
    pub rejected_at: String,
    pub review_notes: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovePromotionRequest {
    pub review_id: String,
    pub derived_record_id: String,
    pub approved_at: String,
    pub review_notes: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateOntologyCandidateRequest {
    pub candidate_id: String,
    pub source_record_id: String,
    pub basis_record_ids: Vec<String>,
    pub proposed_structure: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PromotionReviewReport {
    pub review: PromotionReview,
    pub evidence: Vec<PromotionEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PromotionApprovalReport {
    pub review: PromotionReview,
    pub evidence: Vec<PromotionEvidence>,
    pub derived_record: MemoryRecord,
}

#[derive(Debug, Error)]
pub enum TruthGovernanceError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error("source record {record_id} was not found")]
    SourceRecordNotFound { record_id: String },
    #[error("source record {record_id} is {truth_layer} and cannot enter the T3 promotion gate")]
    SourceRecordNotT3 {
        record_id: String,
        truth_layer: &'static str,
    },
    #[error("source record {record_id} is {truth_layer} and cannot create a T2-to-T1 ontology candidate")]
    SourceRecordNotT2 {
        record_id: String,
        truth_layer: &'static str,
    },
    #[error("source record {record_id} is revoked and cannot enter the promotion gate")]
    SourceRecordRevoked { record_id: String },
    #[error("promotion review {review_id} was not found")]
    PromotionReviewNotFound { review_id: String },
    #[error("promotion review {review_id} is already {decision_state}")]
    PromotionReviewClosed {
        review_id: String,
        decision_state: &'static str,
    },
    #[error("promotion review {review_id} is missing {field}")]
    MissingReviewMetadata {
        review_id: String,
        field: &'static str,
    },
    #[error("evidence record {record_id} was not found")]
    EvidenceRecordNotFound { record_id: String },
    #[error("candidate {candidate_id} must include at least one basis record")]
    MissingCandidateBasis { candidate_id: String },
    #[error("basis record {record_id} was not found")]
    BasisRecordNotFound { record_id: String },
    #[error("promotion review {review_id} has no attached evidence")]
    MissingEvidence { review_id: String },
    #[error("promotion review {review_id} has not passed {gate:?}")]
    GateNotPassed {
        review_id: String,
        gate: PromotionGate,
    },
}

pub struct TruthGovernanceService<'db> {
    repository: MemoryRepository<'db>,
}

impl<'db> TruthGovernanceService<'db> {
    pub fn new(conn: &'db Connection) -> Self {
        Self {
            repository: MemoryRepository::new(conn),
        }
    }

    pub fn create_promotion_review(
        &self,
        request: CreatePromotionReviewRequest,
    ) -> Result<PromotionReviewReport, TruthGovernanceError> {
        if request.review_notes.is_none() {
            return Err(TruthGovernanceError::MissingReviewMetadata {
                review_id: request.review_id,
                field: "review_notes",
            });
        }

        self.require_active_t3_source(&request.source_record_id)?;

        let review = PromotionReview {
            review_id: request.review_id,
            source_record_id: request.source_record_id,
            target_layer: TruthLayer::T2,
            result_trigger_state: ReviewGateState::Pending,
            evidence_review_state: ReviewGateState::Pending,
            consensus_check_state: ReviewGateState::Pending,
            metacog_approval_state: ReviewGateState::Pending,
            decision_state: PromotionDecisionState::Pending,
            review_notes: request.review_notes,
            approved_at: None,
            created_at: request.created_at.clone(),
            updated_at: request.created_at,
        };

        self.repository.insert_promotion_review(&review)?;
        self.repository
            .update_t3_last_reviewed_at(&review.source_record_id, &review.updated_at)?;
        self.review_report(&review.review_id)
    }

    pub fn attach_evidence(
        &self,
        request: AttachPromotionEvidenceRequest,
    ) -> Result<PromotionReviewReport, TruthGovernanceError> {
        let review = self.require_open_review(&request.review_id)?;
        self.require_active_t3_source(&review.source_record_id)?;
        self.require_record_exists(&request.evidence_record_id)?;

        let evidence = PromotionEvidence {
            review_id: request.review_id,
            evidence_record_id: request.evidence_record_id,
            evidence_role: request.evidence_role,
            evidence_note: request.evidence_note,
            created_at: Some(review.updated_at.clone()),
        };

        self.repository.insert_promotion_evidence(&evidence)?;
        self.repository
            .update_t3_last_reviewed_at(&review.source_record_id, &review.updated_at)?;
        self.review_report(&review.review_id)
    }

    pub fn update_promotion_gate(
        &self,
        request: UpdatePromotionGateRequest,
    ) -> Result<PromotionReviewReport, TruthGovernanceError> {
        let mut review = self.require_open_review(&request.review_id)?;
        self.require_active_t3_source(&review.source_record_id)?;

        apply_gate_state(&mut review, request.gate, request.state);
        if let Some(review_notes) = request.review_notes {
            review.review_notes = Some(review_notes);
        }
        review.updated_at = request.updated_at;

        self.repository.update_promotion_review(&review)?;
        self.repository
            .update_t3_last_reviewed_at(&review.source_record_id, &review.updated_at)?;
        self.review_report(&review.review_id)
    }

    pub fn reject_promotion(
        &self,
        request: RejectPromotionRequest,
    ) -> Result<PromotionReviewReport, TruthGovernanceError> {
        let mut review = self.require_open_review(&request.review_id)?;
        self.require_active_t3_source(&review.source_record_id)?;
        let Some(review_notes) = request.review_notes.or_else(|| review.review_notes.clone()) else {
            return Err(TruthGovernanceError::MissingReviewMetadata {
                review_id: request.review_id,
                field: "review_notes",
            });
        };

        review.review_notes = Some(review_notes);
        review.decision_state = PromotionDecisionState::Rejected;
        review.updated_at = request.rejected_at;

        self.repository.update_promotion_review(&review)?;
        self.repository
            .update_t3_last_reviewed_at(&review.source_record_id, &review.updated_at)?;
        self.review_report(&review.review_id)
    }

    pub fn approve_promotion(
        &self,
        request: ApprovePromotionRequest,
    ) -> Result<PromotionApprovalReport, TruthGovernanceError> {
        let mut review = self.require_open_review(&request.review_id)?;
        let source_record = self.require_active_t3_source(&review.source_record_id)?;
        let evidence = self.repository.list_promotion_evidence(&review.review_id)?;
        if evidence.is_empty() {
            return Err(TruthGovernanceError::MissingEvidence {
                review_id: review.review_id.clone(),
            });
        }

        let review_notes = request.review_notes.or_else(|| review.review_notes.clone());
        if review_notes.is_none() {
            return Err(TruthGovernanceError::MissingReviewMetadata {
                review_id: review.review_id.clone(),
                field: "review_notes",
            });
        }

        for gate in [
            PromotionGate::ResultTrigger,
            PromotionGate::EvidenceReview,
            PromotionGate::ConsensusCheck,
            PromotionGate::MetacogApproval,
        ] {
            if !gate_is_passed(&review, gate) {
                return Err(TruthGovernanceError::GateNotPassed {
                    review_id: review.review_id.clone(),
                    gate,
                });
            }
        }

        let derived_record = build_derived_t2_record(
            &source_record,
            &evidence,
            request.derived_record_id,
            request.approved_at.clone(),
        );
        self.repository.insert_record(&derived_record)?;

        review.review_notes = review_notes;
        review.decision_state = PromotionDecisionState::Approved;
        review.approved_at = Some(request.approved_at.clone());
        review.updated_at = request.approved_at;
        self.repository.update_promotion_review(&review)?;
        self.repository
            .update_t3_last_reviewed_at(&review.source_record_id, &review.updated_at)?;

        Ok(PromotionApprovalReport {
            review,
            evidence,
            derived_record,
        })
    }

    pub fn create_ontology_candidate(
        &self,
        request: CreateOntologyCandidateRequest,
    ) -> Result<OntologyCandidate, TruthGovernanceError> {
        self.require_t2_source(&request.source_record_id)?;

        if request.basis_record_ids.is_empty() {
            return Err(TruthGovernanceError::MissingCandidateBasis {
                candidate_id: request.candidate_id,
            });
        }

        let mut basis_record_ids = Vec::with_capacity(request.basis_record_ids.len());
        for record_id in request.basis_record_ids {
            if !basis_record_ids.contains(&record_id) {
                basis_record_ids.push(record_id);
            }
        }
        for record_id in &basis_record_ids {
            if self.repository.get_record(record_id)?.is_none() {
                return Err(TruthGovernanceError::BasisRecordNotFound {
                    record_id: record_id.clone(),
                });
            }
        }

        let candidate = OntologyCandidate {
            candidate_id: request.candidate_id,
            source_record_id: request.source_record_id,
            basis_record_ids,
            proposed_structure: request.proposed_structure,
            time_stability_state: CandidateReviewState::Pending,
            agent_reproducibility_state: CandidateReviewState::Pending,
            context_invariance_state: CandidateReviewState::Pending,
            predictive_utility_state: CandidateReviewState::Pending,
            structural_review_state: CandidateReviewState::Pending,
            candidate_state: OntologyCandidateState::Pending,
            decided_at: None,
            created_at: request.created_at.clone(),
            updated_at: request.created_at,
        };

        self.repository.insert_ontology_candidate(&candidate)?;

        Ok(candidate)
    }

    pub fn get_truth_record(
        &self,
        record_id: &str,
    ) -> Result<Option<TruthRecord>, TruthGovernanceError> {
        self.repository.get_truth_record(record_id).map_err(Into::into)
    }

    pub fn list_pending_reviews(&self) -> Result<Vec<PromotionReview>, TruthGovernanceError> {
        self.repository
            .list_pending_promotion_reviews()
            .map_err(Into::into)
    }

    pub fn list_pending_candidates(&self) -> Result<Vec<OntologyCandidate>, TruthGovernanceError> {
        self.repository
            .list_pending_ontology_candidates()
            .map_err(Into::into)
    }

    fn require_active_t3_source(
        &self,
        record_id: &str,
    ) -> Result<MemoryRecord, TruthGovernanceError> {
        let truth_record = self
            .repository
            .get_truth_record(record_id)?
            .ok_or_else(|| TruthGovernanceError::SourceRecordNotFound {
                record_id: record_id.to_string(),
            })?;

        match truth_record {
            TruthRecord::T3 {
                base,
                t3_state: Some(t3_state),
                ..
            } => {
                if matches!(t3_state.revocation_state, T3RevocationState::Revoked) {
                    return Err(TruthGovernanceError::SourceRecordRevoked {
                        record_id: record_id.to_string(),
                    });
                }

                Ok(base)
            }
            TruthRecord::T3 { .. } => Err(TruthGovernanceError::MissingReviewMetadata {
                review_id: record_id.to_string(),
                field: "t3_state",
            }),
            other => Err(TruthGovernanceError::SourceRecordNotT3 {
                record_id: record_id.to_string(),
                truth_layer: other.truth_layer().as_str(),
            }),
        }
    }

    fn require_t2_source(&self, record_id: &str) -> Result<MemoryRecord, TruthGovernanceError> {
        let truth_record = self
            .repository
            .get_truth_record(record_id)?
            .ok_or_else(|| TruthGovernanceError::SourceRecordNotFound {
                record_id: record_id.to_string(),
            })?;

        match truth_record {
            TruthRecord::T2 { base, .. } => Ok(base),
            other => Err(TruthGovernanceError::SourceRecordNotT2 {
                record_id: record_id.to_string(),
                truth_layer: other.truth_layer().as_str(),
            }),
        }
    }

    fn require_record_exists(&self, record_id: &str) -> Result<(), TruthGovernanceError> {
        if self.repository.get_record(record_id)?.is_none() {
            return Err(TruthGovernanceError::EvidenceRecordNotFound {
                record_id: record_id.to_string(),
            });
        }

        Ok(())
    }

    fn require_open_review(
        &self,
        review_id: &str,
    ) -> Result<PromotionReview, TruthGovernanceError> {
        let review = self
            .repository
            .get_promotion_review(review_id)?
            .ok_or_else(|| TruthGovernanceError::PromotionReviewNotFound {
                review_id: review_id.to_string(),
            })?;

        if !matches!(review.decision_state, PromotionDecisionState::Pending) {
            return Err(TruthGovernanceError::PromotionReviewClosed {
                review_id: review_id.to_string(),
                decision_state: review.decision_state.as_str(),
            });
        }

        Ok(review)
    }

    fn review_report(&self, review_id: &str) -> Result<PromotionReviewReport, TruthGovernanceError> {
        let review = self
            .repository
            .get_promotion_review(review_id)?
            .ok_or_else(|| TruthGovernanceError::PromotionReviewNotFound {
                review_id: review_id.to_string(),
            })?;
        let evidence = self.repository.list_promotion_evidence(review_id)?;

        Ok(PromotionReviewReport { review, evidence })
    }
}

fn apply_gate_state(review: &mut PromotionReview, gate: PromotionGate, state: ReviewGateState) {
    match gate {
        PromotionGate::ResultTrigger => review.result_trigger_state = state,
        PromotionGate::EvidenceReview => review.evidence_review_state = state,
        PromotionGate::ConsensusCheck => review.consensus_check_state = state,
        PromotionGate::MetacogApproval => review.metacog_approval_state = state,
    }
}

fn gate_is_passed(review: &PromotionReview, gate: PromotionGate) -> bool {
    match gate {
        PromotionGate::ResultTrigger => {
            matches!(review.result_trigger_state, ReviewGateState::Passed)
        }
        PromotionGate::EvidenceReview => {
            matches!(review.evidence_review_state, ReviewGateState::Passed)
        }
        PromotionGate::ConsensusCheck => {
            matches!(review.consensus_check_state, ReviewGateState::Passed)
        }
        PromotionGate::MetacogApproval => {
            matches!(review.metacog_approval_state, ReviewGateState::Passed)
        }
    }
}

fn build_derived_t2_record(
    source_record: &MemoryRecord,
    evidence: &[PromotionEvidence],
    derived_record_id: String,
    approved_at: String,
) -> MemoryRecord {
    let mut derived_from = Vec::with_capacity(
        source_record.provenance.derived_from.len() + evidence.len() + 1,
    );
    derived_from.push(source_record.id.clone());
    derived_from.extend(source_record.provenance.derived_from.iter().cloned());
    derived_from.extend(evidence.iter().map(|item| item.evidence_record_id.clone()));
    derived_from.sort();
    derived_from.dedup();

    MemoryRecord {
        id: derived_record_id,
        source: source_record.source.clone(),
        timestamp: crate::memory::record::RecordTimestamp {
            recorded_at: approved_at.clone(),
            created_at: approved_at.clone(),
            updated_at: approved_at,
        },
        scope: source_record.scope,
        record_type: source_record.record_type,
        truth_layer: TruthLayer::T2,
        provenance: Provenance {
            origin: "truth_governance".to_string(),
            imported_via: Some("promotion_review".to_string()),
            derived_from,
        },
        content_text: source_record.content_text.clone(),
        chunk: source_record.chunk.clone(),
        validity: source_record.validity.clone(),
    }
}
