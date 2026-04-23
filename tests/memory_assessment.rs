use agent_memos::memory::{
    dsl::{FactDslDraft, FactDslRecord},
    record::TruthLayer,
    taxonomy::{AspectV1, DomainV1, KindV1, TaxonomyPathV1, TopicV1},
};

#[test]
fn public_dsl_record_exposes_kind_field_assessment() {
    let record = FactDslRecord {
        taxonomy: TaxonomyPathV1::new(
            DomainV1::Project,
            TopicV1::Retrieval,
            AspectV1::Behavior,
            KindV1::Decision,
        )
        .expect("taxonomy path should be valid"),
        draft: FactDslDraft {
            claim: "use lexical-first as baseline".to_string(),
            ..Default::default()
        },
        truth_layer: TruthLayer::T2,
        source_ref: "roadmap#phase9".to_string(),
    };

    let assessment = record.assess_kind_fields();
    assert_eq!(assessment.missing_recommended.len(), 3);
    assert!(assessment.present_discouraged.is_empty());
}
