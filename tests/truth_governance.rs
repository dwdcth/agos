use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    core::db::Database,
    memory::{
        governance::{
            ApprovePromotionRequest, AttachPromotionEvidenceRequest, CreatePromotionReviewRequest,
            CreateOntologyCandidateRequest, PromotionGate, RejectPromotionRequest,
            TruthGovernanceError, TruthGovernanceService, UpdatePromotionGateRequest,
        },
        record::{
            MemoryRecord, Provenance, RecordTimestamp, RecordType, Scope, SourceKind, SourceRef,
            TruthLayer, ValidityWindow,
        },
        repository::MemoryRepository,
        truth::{
            CandidateReviewState, OntologyCandidateState, PromotionDecisionState, ReviewGateState,
            T3Confidence, T3RevocationState, TruthRecord,
        },
    },
};
use serde_json::json;

fn fresh_db_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-truth-tests")
        .join(format!("{name}-{unique}"))
        .join("truth.sqlite")
}

fn sample_record(id: &str, truth_layer: TruthLayer) -> MemoryRecord {
    MemoryRecord {
        id: id.to_string(),
        source: SourceRef {
            uri: format!("memo://truth/{id}"),
            kind: SourceKind::Note,
            label: Some(format!("truth-{id}")),
        },
        timestamp: RecordTimestamp {
            recorded_at: "2026-04-15T10:00:00Z".to_string(),
            created_at: "2026-04-15T10:00:00Z".to_string(),
            updated_at: "2026-04-15T10:00:00Z".to_string(),
        },
        scope: Scope::Project,
        record_type: RecordType::Observation,
        truth_layer,
        provenance: Provenance {
            origin: "test".to_string(),
            imported_via: None,
            derived_from: vec!["seed-1".to_string()],
        },
        content_text: format!("record {id} for truth governance"),
        chunk: None,
        validity: ValidityWindow::default(),
    }
}

fn revoke_t3_record(db: &Database, record_id: &str) {
    db.conn()
        .execute(
            r#"
            UPDATE truth_t3_state
            SET revocation_state = 'revoked',
                revoked_at = '2026-04-15T11:59:00Z',
                revocation_reason = 'user correction'
            WHERE record_id = ?1
            "#,
            [record_id],
        )
        .expect("t3 record should revoke");
}

#[test]
fn truth_model_enums_parse_and_render_as_storage_values() {
    assert_eq!(T3Confidence::parse("high"), Some(T3Confidence::High));
    assert_eq!(T3Confidence::High.as_str(), "high");

    assert_eq!(
        T3RevocationState::parse("active"),
        Some(T3RevocationState::Active)
    );
    assert_eq!(T3RevocationState::Revoked.as_str(), "revoked");

    assert_eq!(
        ReviewGateState::parse("passed"),
        Some(ReviewGateState::Passed)
    );
    assert_eq!(PromotionDecisionState::Approved.as_str(), "approved");

    assert_eq!(
        CandidateReviewState::parse("pending"),
        Some(CandidateReviewState::Pending)
    );
    assert_eq!(OntologyCandidateState::Accepted.as_str(), "accepted");
}

#[test]
fn repository_projects_truth_layers_into_typed_records() {
    let path = fresh_db_path("typed-records");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());

    repo.insert_record(&sample_record("t1-record", TruthLayer::T1))
        .expect("t1 record should insert");
    repo.insert_record(&sample_record("t2-record", TruthLayer::T2))
        .expect("t2 record should insert");
    repo.insert_record(&sample_record("t3-record", TruthLayer::T3))
        .expect("t3 record should insert");

    let t1 = repo
        .get_truth_record("t1-record")
        .expect("truth record lookup should succeed")
        .expect("t1 truth record should exist");
    let t2 = repo
        .get_truth_record("t2-record")
        .expect("truth record lookup should succeed")
        .expect("t2 truth record should exist");
    let t3 = repo
        .get_truth_record("t3-record")
        .expect("truth record lookup should succeed")
        .expect("t3 truth record should exist");

    assert!(matches!(t1, TruthRecord::T1 { .. }));
    assert!(matches!(t2, TruthRecord::T2 { .. }));
    assert!(matches!(t3, TruthRecord::T3 { .. }));
}

#[test]
fn repository_persists_default_t3_governance_state() {
    let path = fresh_db_path("t3-defaults");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());

    repo.insert_record(&sample_record("t3-governed", TruthLayer::T3))
        .expect("t3 record should insert");

    let t3_state = repo
        .get_t3_state("t3-governed")
        .expect("t3 state lookup should succeed")
        .expect("t3 state should exist");

    assert_eq!(t3_state.confidence, T3Confidence::Medium);
    assert_eq!(t3_state.revocation_state, T3RevocationState::Active);
    assert_eq!(t3_state.record_id, "t3-governed");
}

#[test]
fn promotion_review_tracks_gate_states_without_promoting() {
    let path = fresh_db_path("promotion-gates");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());
    let service = TruthGovernanceService::new(db.conn());

    repo.insert_record(&sample_record("t3-source", TruthLayer::T3))
        .expect("t3 record should insert");
    repo.insert_record(&sample_record("support-1", TruthLayer::T2))
        .expect("supporting evidence should insert");
    repo.insert_record(&sample_record("revoked-t3", TruthLayer::T3))
        .expect("revoked t3 record should insert");
    revoke_t3_record(&db, "revoked-t3");

    let created = service
        .create_promotion_review(CreatePromotionReviewRequest {
            review_id: "review-1".to_string(),
            source_record_id: "t3-source".to_string(),
            created_at: "2026-04-15T12:00:00Z".to_string(),
            review_notes: Some(json!({
                "reviewer": "truth-test",
                "summary": "initial promotion review"
            })),
        })
        .expect("review should create");

    assert_eq!(created.review.result_trigger_state, ReviewGateState::Pending);
    assert_eq!(created.review.evidence_review_state, ReviewGateState::Pending);
    assert_eq!(created.review.consensus_check_state, ReviewGateState::Pending);
    assert_eq!(
        created.review.metacog_approval_state,
        ReviewGateState::Pending
    );
    assert_eq!(created.review.decision_state, PromotionDecisionState::Pending);
    assert!(created.evidence.is_empty());

    let attached = service
        .attach_evidence(AttachPromotionEvidenceRequest {
            review_id: "review-1".to_string(),
            evidence_record_id: "support-1".to_string(),
            evidence_role: agent_memos::memory::truth::EvidenceRole::Supporting,
            evidence_note: Some(json!({
                "why": "backs the working hypothesis"
            })),
        })
        .expect("evidence should attach");
    assert_eq!(attached.evidence.len(), 1);

    let after_result = service
        .update_promotion_gate(UpdatePromotionGateRequest {
            review_id: "review-1".to_string(),
            gate: PromotionGate::ResultTrigger,
            state: ReviewGateState::Passed,
            updated_at: "2026-04-15T12:01:00Z".to_string(),
            review_notes: None,
        })
        .expect("result gate should update");
    assert_eq!(after_result.review.result_trigger_state, ReviewGateState::Passed);
    assert_eq!(after_result.review.evidence_review_state, ReviewGateState::Pending);

    let after_consensus = service
        .update_promotion_gate(UpdatePromotionGateRequest {
            review_id: "review-1".to_string(),
            gate: PromotionGate::ConsensusCheck,
            state: ReviewGateState::Rejected,
            updated_at: "2026-04-15T12:02:00Z".to_string(),
            review_notes: Some(json!({
                "risk": "needs more corroboration"
            })),
        })
        .expect("consensus gate should update");
    assert_eq!(
        after_consensus.review.consensus_check_state,
        ReviewGateState::Rejected
    );
    assert_eq!(repo.count_records().expect("count should load"), 3);

    let source_truth = repo
        .get_truth_record("t3-source")
        .expect("truth record lookup should succeed")
        .expect("source truth record should exist");
    match source_truth {
        TruthRecord::T3 {
            t3_state: Some(t3_state),
            open_reviews,
            ..
        } => {
            assert_eq!(t3_state.revocation_state, T3RevocationState::Active);
            assert_eq!(open_reviews.len(), 1);
            assert_eq!(open_reviews[0].review_id, "review-1");
            assert_eq!(open_reviews[0].result_trigger_state, ReviewGateState::Passed);
            assert_eq!(
                open_reviews[0].consensus_check_state,
                ReviewGateState::Rejected
            );
        }
        other => panic!("expected governed T3 record, got {other:?}"),
    }

    let non_t3_error = service
        .create_promotion_review(CreatePromotionReviewRequest {
            review_id: "review-not-t3".to_string(),
            source_record_id: "support-1".to_string(),
            created_at: "2026-04-15T12:03:00Z".to_string(),
            review_notes: Some(json!({
                "summary": "should fail for non-T3"
            })),
        })
        .expect_err("non-T3 promotion should fail");
    assert!(matches!(
        non_t3_error,
        TruthGovernanceError::SourceRecordNotT3 { record_id, .. } if record_id == "support-1"
    ));

    let revoked_error = service
        .create_promotion_review(CreatePromotionReviewRequest {
            review_id: "review-revoked".to_string(),
            source_record_id: "revoked-t3".to_string(),
            created_at: "2026-04-15T12:04:00Z".to_string(),
            review_notes: Some(json!({
                "summary": "should fail for revoked T3"
            })),
        })
        .expect_err("revoked T3 promotion should fail");
    assert!(matches!(
        revoked_error,
        TruthGovernanceError::SourceRecordRevoked { record_id } if record_id == "revoked-t3"
    ));
}

#[test]
fn t3_promotion_requires_all_gate_checks() {
    let path = fresh_db_path("promotion-approval");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());
    let service = TruthGovernanceService::new(db.conn());

    repo.insert_record(&sample_record("t3-source", TruthLayer::T3))
        .expect("t3 record should insert");
    repo.insert_record(&sample_record("support-1", TruthLayer::T2))
        .expect("supporting evidence should insert");

    service
        .create_promotion_review(CreatePromotionReviewRequest {
            review_id: "review-2".to_string(),
            source_record_id: "t3-source".to_string(),
            created_at: "2026-04-15T13:00:00Z".to_string(),
            review_notes: Some(json!({
                "summary": "promotion candidate has explicit review context"
            })),
        })
        .expect("review should create");
    service
        .attach_evidence(AttachPromotionEvidenceRequest {
            review_id: "review-2".to_string(),
            evidence_record_id: "support-1".to_string(),
            evidence_role: agent_memos::memory::truth::EvidenceRole::Supporting,
            evidence_note: Some(json!({
                "confidence": "strong"
            })),
        })
        .expect("evidence should attach");

    let first_error = service
        .approve_promotion(ApprovePromotionRequest {
            review_id: "review-2".to_string(),
            derived_record_id: "derived-t2".to_string(),
            approved_at: "2026-04-15T13:10:00Z".to_string(),
            review_notes: None,
        })
        .expect_err("approval should fail before gates pass");
    assert!(matches!(
        first_error,
        TruthGovernanceError::GateNotPassed {
            review_id,
            gate: PromotionGate::ResultTrigger
        } if review_id == "review-2"
    ));

    for (gate, updated_at) in [
        (PromotionGate::ResultTrigger, "2026-04-15T13:01:00Z"),
        (PromotionGate::EvidenceReview, "2026-04-15T13:02:00Z"),
        (PromotionGate::ConsensusCheck, "2026-04-15T13:03:00Z"),
    ] {
        service
            .update_promotion_gate(UpdatePromotionGateRequest {
                review_id: "review-2".to_string(),
                gate,
                state: ReviewGateState::Passed,
                updated_at: updated_at.to_string(),
                review_notes: None,
            })
            .expect("gate should update");
    }

    let missing_metacog = service
        .approve_promotion(ApprovePromotionRequest {
            review_id: "review-2".to_string(),
            derived_record_id: "derived-t2".to_string(),
            approved_at: "2026-04-15T13:10:00Z".to_string(),
            review_notes: None,
        })
        .expect_err("approval should wait for metacognitive approval");
    assert!(matches!(
        missing_metacog,
        TruthGovernanceError::GateNotPassed {
            review_id,
            gate: PromotionGate::MetacogApproval
        } if review_id == "review-2"
    ));

    service
        .update_promotion_gate(UpdatePromotionGateRequest {
            review_id: "review-2".to_string(),
            gate: PromotionGate::MetacogApproval,
            state: ReviewGateState::Passed,
            updated_at: "2026-04-15T13:04:00Z".to_string(),
            review_notes: None,
        })
        .expect("metacognitive gate should update");

    let approval = service
        .approve_promotion(ApprovePromotionRequest {
            review_id: "review-2".to_string(),
            derived_record_id: "derived-t2".to_string(),
            approved_at: "2026-04-15T13:10:00Z".to_string(),
            review_notes: Some(json!({
                "summary": "promotion approved after full review"
            })),
        })
        .expect("approval should succeed once all gates pass");

    assert_eq!(approval.derived_record.id, "derived-t2");
    assert_eq!(approval.derived_record.truth_layer, TruthLayer::T2);
    assert_eq!(approval.review.decision_state, PromotionDecisionState::Approved);
    assert_eq!(
        approval.derived_record.provenance.origin,
        "truth_governance"
    );
    assert!(
        approval
            .derived_record
            .provenance
            .derived_from
            .contains(&"t3-source".to_string())
    );
    assert!(
        approval
            .derived_record
            .provenance
            .derived_from
            .contains(&"support-1".to_string())
    );

    let source_truth = repo
        .get_truth_record("t3-source")
        .expect("truth record lookup should succeed")
        .expect("source truth record should exist");
    match source_truth {
        TruthRecord::T3 {
            base,
            t3_state: Some(t3_state),
            open_reviews,
        } => {
            assert_eq!(base.truth_layer, TruthLayer::T3);
            assert_eq!(t3_state.revocation_state, T3RevocationState::Active);
            assert_eq!(
                t3_state.last_reviewed_at,
                Some("2026-04-15T13:10:00Z".to_string())
            );
            assert_eq!(open_reviews.len(), 1);
            assert_eq!(open_reviews[0].decision_state, PromotionDecisionState::Approved);
        }
        other => panic!("expected promoted source to remain T3, got {other:?}"),
    }

    let derived_truth = repo
        .get_truth_record("derived-t2")
        .expect("truth record lookup should succeed")
        .expect("derived truth record should exist");
    match derived_truth {
        TruthRecord::T2 {
            base,
            open_candidates,
        } => {
            assert_eq!(base.truth_layer, TruthLayer::T2);
            assert!(open_candidates.is_empty());
        }
        other => panic!("expected derived T2 record, got {other:?}"),
    }
}

#[test]
fn rejected_promotion_stays_auditable() {
    let path = fresh_db_path("promotion-rejected");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());
    let service = TruthGovernanceService::new(db.conn());

    repo.insert_record(&sample_record("t3-source", TruthLayer::T3))
        .expect("t3 record should insert");
    repo.insert_record(&sample_record("support-1", TruthLayer::T2))
        .expect("supporting evidence should insert");

    service
        .create_promotion_review(CreatePromotionReviewRequest {
            review_id: "review-rejected".to_string(),
            source_record_id: "t3-source".to_string(),
            created_at: "2026-04-15T14:00:00Z".to_string(),
            review_notes: Some(json!({
                "summary": "candidate starts under review"
            })),
        })
        .expect("review should create");
    service
        .attach_evidence(AttachPromotionEvidenceRequest {
            review_id: "review-rejected".to_string(),
            evidence_record_id: "support-1".to_string(),
            evidence_role: agent_memos::memory::truth::EvidenceRole::Contradicting,
            evidence_note: Some(json!({
                "reason": "contradicts the hypothesis"
            })),
        })
        .expect("evidence should attach");
    service
        .update_promotion_gate(UpdatePromotionGateRequest {
            review_id: "review-rejected".to_string(),
            gate: PromotionGate::EvidenceReview,
            state: ReviewGateState::Rejected,
            updated_at: "2026-04-15T14:01:00Z".to_string(),
            review_notes: Some(json!({
                "summary": "evidence review rejected the claim"
            })),
        })
        .expect("gate should update");

    let rejected = service
        .reject_promotion(RejectPromotionRequest {
            review_id: "review-rejected".to_string(),
            rejected_at: "2026-04-15T14:02:00Z".to_string(),
            review_notes: Some(json!({
                "summary": "promotion rejected after evidence review"
            })),
        })
        .expect("rejection should succeed");
    assert_eq!(rejected.review.decision_state, PromotionDecisionState::Rejected);

    let source_truth = repo
        .get_truth_record("t3-source")
        .expect("truth record lookup should succeed")
        .expect("source truth record should exist");
    match source_truth {
        TruthRecord::T3 {
            t3_state: Some(t3_state),
            open_reviews,
            ..
        } => {
            assert_eq!(t3_state.record_id, "t3-source");
            assert_eq!(
                t3_state.last_reviewed_at,
                Some("2026-04-15T14:02:00Z".to_string())
            );
            assert_eq!(open_reviews.len(), 1);
            assert_eq!(open_reviews[0].decision_state, PromotionDecisionState::Rejected);
            assert_eq!(
                open_reviews[0].evidence_review_state,
                ReviewGateState::Rejected
            );
        }
        other => panic!("expected source to stay auditable T3, got {other:?}"),
    }
}

#[test]
fn t2_to_t1_creates_candidate_without_t1_mutation() {
    let path = fresh_db_path("t2-to-t1-candidate");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());
    let service = TruthGovernanceService::new(db.conn());

    repo.insert_record(&sample_record("t2-source", TruthLayer::T2))
        .expect("t2 record should insert");
    repo.insert_record(&sample_record("basis-1", TruthLayer::T2))
        .expect("basis record should insert");
    repo.insert_record(&sample_record("existing-t1", TruthLayer::T1))
        .expect("t1 record should insert");

    let candidate = service
        .create_ontology_candidate(CreateOntologyCandidateRequest {
            candidate_id: "candidate-1".to_string(),
            source_record_id: "t2-source".to_string(),
            basis_record_ids: vec!["t2-source".to_string(), "basis-1".to_string()],
            proposed_structure: json!({
                "kind": "ontology_node",
                "label": "shared truth candidate",
                "attributes": {
                    "stability": "under review"
                }
            }),
            created_at: "2026-04-15T15:00:00Z".to_string(),
        })
        .expect("candidate should create from a T2 source");

    assert_eq!(candidate.candidate_id, "candidate-1");
    assert_eq!(candidate.source_record_id, "t2-source");
    assert_eq!(
        candidate.basis_record_ids,
        vec!["t2-source".to_string(), "basis-1".to_string()]
    );
    assert_eq!(
        candidate.time_stability_state,
        CandidateReviewState::Pending
    );
    assert_eq!(
        candidate.agent_reproducibility_state,
        CandidateReviewState::Pending
    );
    assert_eq!(
        candidate.context_invariance_state,
        CandidateReviewState::Pending
    );
    assert_eq!(
        candidate.predictive_utility_state,
        CandidateReviewState::Pending
    );
    assert_eq!(
        candidate.structural_review_state,
        CandidateReviewState::Pending
    );
    assert_eq!(candidate.candidate_state, OntologyCandidateState::Pending);

    let t1_records = repo
        .list_records()
        .expect("records should list")
        .into_iter()
        .filter(|record| matches!(record.truth_layer, TruthLayer::T1))
        .collect::<Vec<_>>();
    assert_eq!(t1_records.len(), 1);
    assert_eq!(t1_records[0].id, "existing-t1");

    let source_truth = service
        .get_truth_record("t2-source")
        .expect("typed truth lookup should succeed")
        .expect("source truth record should exist");
    match source_truth {
        TruthRecord::T2 {
            base,
            open_candidates,
        } => {
            assert_eq!(base.truth_layer, TruthLayer::T2);
            assert_eq!(open_candidates.len(), 1);
            assert_eq!(open_candidates[0].candidate_id, "candidate-1");
        }
        other => panic!("expected source to stay T2 with open candidate, got {other:?}"),
    }

    let non_t2_error = service
        .create_ontology_candidate(CreateOntologyCandidateRequest {
            candidate_id: "candidate-invalid".to_string(),
            source_record_id: "existing-t1".to_string(),
            basis_record_ids: vec!["existing-t1".to_string()],
            proposed_structure: json!({
                "kind": "ontology_node",
                "label": "should fail"
            }),
            created_at: "2026-04-15T15:01:00Z".to_string(),
        })
        .expect_err("non-T2 sources should be rejected");
    assert!(matches!(
        non_t2_error,
        TruthGovernanceError::SourceRecordNotT2 { record_id, .. } if record_id == "existing-t1"
    ));
}
