use agent_memos::memory::{
    dsl::{FactDslDraft, FactDslRecord},
    record::TruthLayer,
    summary::FactSummaryInput,
    taxonomy::{AspectV1, DomainV1, KindV1, TaxonomyPathV1, TopicV1},
};

#[test]
fn memory_pipeline_round_trips_from_summary_input_to_dsl_text() {
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
            impact: Some("keeps ordinary retrieval debuggable".to_string()),
            ..Default::default()
        })
        .expect("valid summary draft should convert into DSL record");

    let encoded = record.encode().expect("DSL record should encode");
    let parsed = FactDslRecord::parse(&encoded).expect("encoded DSL should parse");

    assert_eq!(parsed, record);
    assert!(encoded.contains("WHY=explainability matters"));
    assert!(encoded.contains("IMPACT=keeps ordinary retrieval debuggable"));
}
