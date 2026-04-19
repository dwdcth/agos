use agent_memos::memory::{
    dsl::{FactDslDraft, FactDslRecord, KindFieldPolicyV1},
    record::TruthLayer,
    taxonomy::{AspectV1, DomainV1, KindV1, TaxonomyPathV1, TopicV1},
};

#[test]
fn public_memory_dsl_api_encodes_decision_records() {
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
            why: Some("explainability and stable recall".to_string()),
            time: Some("2026-04".to_string()),
            cond: None,
            impact: Some("keeps ordinary retrieval debuggable".to_string()),
            conf: None,
            rel: None,
        },
        truth_layer: TruthLayer::T2,
        source_ref: "roadmap#phase9".to_string(),
    };

    let encoded = record.encode().expect("public DSL record should encode");
    assert_eq!(
        encoded,
        "F|DOM=project|TOP=retrieval|ASP=behavior|KIND=decision|CLAIM=use lexical-first as baseline|TRUTH=T2|SRC=roadmap#phase9|WHY=explainability and stable recall|TIME=2026-04|IMPACT=keeps ordinary retrieval debuggable"
    );
}

#[test]
fn public_kind_field_policy_exposes_risk_shape() {
    let policy = KindFieldPolicyV1::for_kind(KindV1::Risk);

    assert_eq!(policy.recommended.len(), 3);
    assert_eq!(policy.recommended[0].as_str(), "IMPACT");
    assert_eq!(policy.recommended[1].as_str(), "COND");
    assert_eq!(policy.recommended[2].as_str(), "TIME");
}
