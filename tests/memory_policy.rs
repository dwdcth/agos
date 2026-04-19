use agent_memos::memory::{
    dsl::{DslFieldV1, FactDslDraft, KindFieldAssessmentV1},
    taxonomy::KindV1,
};

#[test]
fn public_kind_field_assessment_surfaces_policy_mismatches() {
    let assessment = KindFieldAssessmentV1::assess(
        KindV1::Hypothesis,
        &FactDslDraft {
            claim: "tfidf is enough".to_string(),
            impact: Some("could miss edge cases".to_string()),
            ..Default::default()
        },
    );

    assert_eq!(
        assessment.missing_recommended,
        vec![DslFieldV1::Cond, DslFieldV1::Conf, DslFieldV1::Time]
    );
    assert_eq!(assessment.present_discouraged, vec![DslFieldV1::Impact]);
}
