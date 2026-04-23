use agent_memos::memory::taxonomy::{AspectV1, DomainV1, KindV1, TaxonomyPathV1, TopicV1};

#[test]
fn public_taxonomy_api_parses_and_formats_values() {
    assert_eq!(DomainV1::parse("project"), Some(DomainV1::Project));
    assert_eq!(TopicV1::parse("retrieval"), Some(TopicV1::Retrieval));
    assert_eq!(AspectV1::parse("behavior"), Some(AspectV1::Behavior));
    assert_eq!(KindV1::parse("decision"), Some(KindV1::Decision));

    assert_eq!(DomainV1::Project.as_str(), "project");
    assert_eq!(TopicV1::Retrieval.as_str(), "retrieval");
    assert_eq!(AspectV1::Behavior.as_str(), "behavior");
    assert_eq!(KindV1::Decision.as_str(), "decision");
}

#[test]
fn public_taxonomy_api_enforces_domain_topic_compatibility() {
    let path = TaxonomyPathV1::new(
        DomainV1::External,
        TopicV1::Provider,
        AspectV1::General,
        KindV1::Fact,
    )
    .expect("external/provider should be allowed");

    assert_eq!(path.domain.as_str(), "external");
    assert_eq!(path.topic.as_str(), "provider");

    let invalid = TaxonomyPathV1::new(
        DomainV1::Process,
        TopicV1::Provider,
        AspectV1::General,
        KindV1::Fact,
    );
    assert!(invalid.is_err(), "process/provider should stay invalid");
}
