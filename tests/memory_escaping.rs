use agent_memos::memory::{
    dsl::{FactDslDraft, FactDslRecord},
    record::TruthLayer,
    taxonomy::{AspectV1, DomainV1, KindV1, TaxonomyPathV1, TopicV1},
};

#[test]
fn public_memory_dsl_round_trip_preserves_reserved_separators() {
    let record = FactDslRecord {
        taxonomy: TaxonomyPathV1::new(
            DomainV1::Project,
            TopicV1::Retrieval,
            AspectV1::Behavior,
            KindV1::Decision,
        )
        .expect("taxonomy path should be valid"),
        draft: FactDslDraft {
            claim: "use lexical|first = baseline".to_string(),
            why: Some("preserve a,b and c".to_string()),
            time: None,
            cond: None,
            impact: None,
            conf: None,
            rel: Some(vec!["alpha,beta".to_string(), "x|y=z".to_string()]),
        },
        truth_layer: TruthLayer::T2,
        source_ref: "roadmap#phase9|line=12".to_string(),
    };

    let encoded = record.encode().expect("record should encode");
    let parsed = FactDslRecord::parse(&encoded).expect("encoded DSL should parse");

    assert_eq!(parsed, record);
}
