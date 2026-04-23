use agent_memos::memory::{
    classifier::ClassificationOutput,
    dsl::{FactDslDraft, FactDslRecord},
    record::TruthLayer,
    taxonomy::{AspectV1, DomainV1, KindV1, TaxonomyPathV1, TopicV1},
};

#[test]
fn full_memory_flow_runs_from_classifier_output_to_round_tripped_dsl() {
    let classified = ClassificationOutput::new(
        TaxonomyPathV1::new(
            DomainV1::Project,
            TopicV1::Retrieval,
            AspectV1::Behavior,
            KindV1::Decision,
        )
        .expect("taxonomy path should be valid"),
        0.91,
        false,
    )
    .expect("classification output should validate");

    let summary_input = classified
        .into_summary_input(
            TruthLayer::T2,
            "roadmap#phase9",
            "Use lexical-first as baseline because explainability matters.",
        )
        .expect("classification output should bridge into summary input");

    let record = summary_input
        .into_record(FactDslDraft {
            claim: "use lexical-first as baseline".to_string(),
            why: Some("explainability matters".to_string()),
            time: Some("2026-04".to_string()),
            impact: Some("keeps ordinary retrieval debuggable".to_string()),
            ..Default::default()
        })
        .expect("summary input should convert into DSL record");

    let encoded = record.encode().expect("DSL record should encode");
    let parsed = FactDslRecord::parse(&encoded).expect("encoded DSL should parse");

    assert_eq!(parsed, record);
    assert!(encoded.contains("KIND=decision"));
    assert!(encoded.contains("WHY=explainability matters"));
}
