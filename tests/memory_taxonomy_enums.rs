use std::str::FromStr;

use agent_memos::memory::taxonomy::{AspectV1, DomainV1, KindV1, TopicV1};

#[test]
fn public_taxonomy_enums_support_display_and_from_str() {
    assert_eq!(DomainV1::Project.to_string(), "project");
    assert_eq!(TopicV1::Retrieval.to_string(), "retrieval");
    assert_eq!(AspectV1::Behavior.to_string(), "behavior");
    assert_eq!(KindV1::Decision.to_string(), "decision");

    assert_eq!(DomainV1::from_str("project"), Ok(DomainV1::Project));
    assert_eq!(TopicV1::from_str("retrieval"), Ok(TopicV1::Retrieval));
    assert_eq!(AspectV1::from_str("behavior"), Ok(AspectV1::Behavior));
    assert_eq!(KindV1::from_str("decision"), Ok(KindV1::Decision));
}
