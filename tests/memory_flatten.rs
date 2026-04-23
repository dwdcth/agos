use agent_memos::memory::{
    dsl::{FactDslDraft, FactDslRecord},
    record::TruthLayer,
    taxonomy::{AspectV1, DomainV1, KindV1, TaxonomyPathV1, TopicV1},
};

#[test]
fn public_dsl_record_supports_flattened_storage_view() {
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
            why: Some("explainability matters".to_string()),
            time: Some("2026-04".to_string()),
            ..Default::default()
        },
        truth_layer: TruthLayer::T2,
        source_ref: "roadmap#phase9".to_string(),
    };

    let flat = record.flatten();
    assert_eq!(flat.domain, "project");
    assert_eq!(flat.topic, "retrieval");
    assert_eq!(flat.truth_layer, "t2");

    let rebuilt = flat.into_record().expect("flat record should rebuild");
    assert_eq!(rebuilt, record);
}

#[test]
fn public_flattened_storage_view_supports_json_round_trip() {
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
            why: Some("explainability matters".to_string()),
            time: Some("2026-04".to_string()),
            ..Default::default()
        },
        truth_layer: TruthLayer::T2,
        source_ref: "roadmap#phase9".to_string(),
    };

    let flat = record.flatten();
    let json = flat.to_json_string().expect("flat record should serialize");
    let parsed = agent_memos::memory::dsl::FlatFactDslRecordV1::from_json_str(&json)
        .expect("json should parse");

    assert_eq!(parsed, flat);
}
