use agent_memos::memory::{
    dsl::{FactDslDraft, FactDslRecord},
    record::TruthLayer,
    store::{FactDslStore, InMemoryFactDslStore, JsonFileFactDslStore, PersistedFactDslRecordV1},
    taxonomy::{AspectV1, DomainV1, KindV1, TaxonomyPathV1, TopicV1},
};

#[test]
fn public_persisted_fact_dsl_record_supports_round_trip() {
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

    let persisted = PersistedFactDslRecordV1::from_fact_dsl_record("mem-1", &record)
        .expect("persisted wrapper should build");
    let rebuilt = persisted
        .clone()
        .into_fact_dsl_record()
        .expect("persisted wrapper should rebuild record");

    assert_eq!(persisted.record_id, "mem-1");
    assert_eq!(rebuilt, record);
}

#[test]
fn public_in_memory_store_supports_persistence_contract() {
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

    let store = InMemoryFactDslStore::new();
    let persisted = PersistedFactDslRecordV1::from_fact_dsl_record("mem-1", &record)
        .expect("persisted wrapper should build");

    store.put_fact_dsl(&persisted).expect("store should persist");
    let loaded = store
        .get_fact_dsl("mem-1")
        .expect("lookup should succeed")
        .expect("row should exist");

    assert_eq!(loaded, persisted);
}

#[test]
fn public_in_memory_store_supports_listing() {
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

    let store = InMemoryFactDslStore::new();
    let persisted = PersistedFactDslRecordV1::from_fact_dsl_record("mem-1", &record)
        .expect("persisted wrapper should build");

    store.put_fact_dsl(&persisted).expect("store should persist");
    let listed = store.list_fact_dsls().expect("listing should succeed");

    assert_eq!(listed, vec![persisted]);
}

#[test]
fn public_json_file_store_supports_contract() {
    let path = std::env::temp_dir()
        .join("agent-memos-store-tests")
        .join(format!("public-{}.json", std::process::id()));
    let _ = std::fs::remove_file(&path);

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

    let store = JsonFileFactDslStore::new(&path);
    let persisted = PersistedFactDslRecordV1::from_fact_dsl_record("mem-1", &record)
        .expect("persisted wrapper should build");

    store.put_fact_dsl(&persisted).expect("store should persist");
    let loaded = store
        .get_fact_dsl("mem-1")
        .expect("lookup should succeed")
        .expect("row should exist");

    assert_eq!(loaded, persisted);
    let listed = store.list_fact_dsls().expect("listing should succeed");
    assert_eq!(listed, vec![persisted]);

    let _ = std::fs::remove_file(path);
}

#[test]
fn public_store_contract_supports_deletion() {
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

    let store = InMemoryFactDslStore::new();
    let persisted = PersistedFactDslRecordV1::from_fact_dsl_record("mem-1", &record)
        .expect("persisted wrapper should build");

    store.put_fact_dsl(&persisted).expect("store should persist");
    let removed = store
        .delete_fact_dsl("mem-1")
        .expect("delete should succeed")
        .expect("row should exist");

    assert_eq!(removed, persisted);
    assert!(store.get_fact_dsl("mem-1").expect("lookup should succeed").is_none());
}

#[test]
fn public_store_contract_supports_topic_and_path_filters() {
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

    let store = InMemoryFactDslStore::new();
    let persisted = PersistedFactDslRecordV1::from_fact_dsl_record("mem-1", &record)
        .expect("persisted wrapper should build");

    store.put_fact_dsl(&persisted).expect("store should persist");
    let by_topic = store
        .list_fact_dsls_by_topic(TopicV1::Retrieval)
        .expect("topic filter should succeed");
    let by_path = store
        .list_fact_dsls_by_path(&record.taxonomy)
        .expect("path filter should succeed");

    assert_eq!(by_topic, vec![persisted.clone()]);
    assert_eq!(by_path, vec![persisted]);
}

#[test]
fn public_store_contract_supports_bulk_persistence() {
    let store = InMemoryFactDslStore::new();

    let first = PersistedFactDslRecordV1::from_fact_dsl_record(
        "mem-1",
        &FactDslRecord {
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
        },
    )
    .expect("first wrapper should build");

    let second = PersistedFactDslRecordV1::from_fact_dsl_record(
        "mem-2",
        &FactDslRecord {
            taxonomy: TaxonomyPathV1::new(
                DomainV1::Project,
                TopicV1::Retrieval,
                AspectV1::Behavior,
                KindV1::Decision,
            )
            .expect("taxonomy path should be valid"),
            draft: FactDslDraft {
                claim: "keep lexical recall debuggable".to_string(),
                why: Some("citations stay stable".to_string()),
                time: Some("2026-04".to_string()),
                ..Default::default()
            },
            truth_layer: TruthLayer::T2,
            source_ref: "notes#phase9".to_string(),
        },
    )
    .expect("second wrapper should build");

    store
        .put_many_fact_dsls(&[first.clone(), second.clone()])
        .expect("bulk persist should succeed");

    let listed = store.list_fact_dsls().expect("listing should succeed");
    assert_eq!(listed, vec![first, second]);
}
