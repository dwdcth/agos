use agent_memos::memory::{
    record::TruthLayer,
    summary::{FactSummaryGenerator, FactSummaryInput, RuleBasedSummaryGenerator},
    taxonomy::{AspectV1, DomainV1, KindV1, TaxonomyPathV1, TopicV1},
};

#[tokio::test]
async fn public_rule_based_summary_generator_produces_hypothesis_confidence() {
    let input = FactSummaryInput::new(
        TaxonomyPathV1::new(
            DomainV1::Process,
            TopicV1::Experiment,
            AspectV1::Behavior,
            KindV1::Hypothesis,
        )
        .expect("taxonomy path should be valid"),
        TruthLayer::T3,
        "notes#classifier",
        "2026-04 tfidf might be enough for taxonomy v1 because the corpus is still small.",
    );

    let draft = RuleBasedSummaryGenerator
        .summarize(&input)
        .await
        .expect("rule-based summary should succeed");

    assert_eq!(draft.time.as_deref(), Some("2026-04"));
    assert_eq!(draft.conf, Some(0.5));
    assert!(draft.claim.contains("tfidf might be enough"));
}

#[tokio::test]
async fn public_rule_based_summary_generator_extracts_condition_and_impact() {
    let input = FactSummaryInput::new(
        TaxonomyPathV1::new(
            DomainV1::System,
            TopicV1::Runtime,
            AspectV1::Risk,
            KindV1::Risk,
        )
        .expect("taxonomy path should be valid"),
        TruthLayer::T2,
        "notes#risk",
        "If embedding replaces lexical baseline, recall may drift, so debugging becomes harder.",
    );

    let draft = RuleBasedSummaryGenerator
        .summarize(&input)
        .await
        .expect("rule-based summary should succeed");

    assert!(draft.cond.as_deref().is_some_and(|value| value.contains("embedding replaces lexical baseline")));
    assert_eq!(draft.impact.as_deref(), Some("debugging becomes harder."));
}
