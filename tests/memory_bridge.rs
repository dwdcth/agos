use agent_memos::memory::{
    classifier::ClassificationOutput,
    record::TruthLayer,
    taxonomy::{AspectV1, DomainV1, KindV1, TaxonomyPathV1, TopicV1},
};

#[test]
fn public_classifier_output_bridges_into_summary_input() {
    let output = ClassificationOutput::new(
        TaxonomyPathV1::new(
            DomainV1::Project,
            TopicV1::Retrieval,
            AspectV1::Behavior,
            KindV1::Decision,
        )
        .expect("taxonomy path should be valid"),
        0.9,
        false,
    )
    .expect("classification output should validate");

    let summary_input = output
        .into_summary_input(
            TruthLayer::T2,
            "roadmap#phase9",
            "Use lexical-first as baseline because explainability matters.",
        )
        .expect("classification output should bridge into summary input");

    assert_eq!(summary_input.taxonomy.topic.as_str(), "retrieval");
    assert_eq!(summary_input.truth_layer.as_str(), "t2");
}
