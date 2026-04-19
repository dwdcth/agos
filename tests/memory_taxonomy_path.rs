use std::str::FromStr;

use agent_memos::memory::taxonomy::TaxonomyPathV1;

#[test]
fn public_taxonomy_path_supports_string_round_trip() {
    let path =
        TaxonomyPathV1::from_str("project/retrieval/behavior/decision").expect("path should parse");

    assert_eq!(path.to_string(), "project/retrieval/behavior/decision");
}
