use agent_memos::memory::truth::{
    CandidateReviewState, OntologyCandidateState, PromotionDecisionState, ReviewGateState,
    T3Confidence, T3RevocationState,
};

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
