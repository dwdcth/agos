use agent_memos::memory::{
    dsl::FactDslDraft,
    record::TruthLayer,
    summary::{FactSummaryInput, validate_summary_output},
    taxonomy::{AspectV1, DomainV1, KindV1, TaxonomyPathV1, TopicV1},
};

#[test]
fn public_summary_input_uses_taxonomy_and_kind_policy() {
    let input = FactSummaryInput::new(
        TaxonomyPathV1::new(
            DomainV1::Project,
            TopicV1::Retrieval,
            AspectV1::Behavior,
            KindV1::Decision,
        )
        .expect("taxonomy path should be valid"),
        TruthLayer::T2,
        "roadmap#phase9",
        "Use lexical-first as baseline because explainability matters.",
    );

    input.validate().expect("input should validate");
    let policy = input.kind_policy();
    assert_eq!(policy.recommended[0].as_str(), "WHY");
}

#[test]
fn public_summary_validation_requires_claim() {
    let err = validate_summary_output(KindV1::Fact, &FactDslDraft::default())
        .expect_err("missing claim should fail output validation");

    assert_eq!(err.to_string(), "generated draft is missing claim");
}

#[test]
fn public_summary_input_converts_valid_draft_into_record() {
    let input = FactSummaryInput::new(
        TaxonomyPathV1::new(
            DomainV1::Project,
            TopicV1::Retrieval,
            AspectV1::Behavior,
            KindV1::Decision,
        )
        .expect("taxonomy path should be valid"),
        TruthLayer::T2,
        "roadmap#phase9",
        "Use lexical-first as baseline because explainability matters.",
    );

    let record = input
        .into_record(FactDslDraft {
            claim: "use lexical-first as baseline".to_string(),
            why: Some("explainability matters".to_string()),
            time: Some("2026-04".to_string()),
            ..Default::default()
        })
        .expect("valid summary draft should convert into DSL record");

    assert_eq!(record.taxonomy.kind.as_str(), "decision");
    assert_eq!(record.source_ref, "roadmap#phase9");
}
