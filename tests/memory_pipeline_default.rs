use agent_memos::memory::{
    pipeline::DefaultMemoryPipeline,
    record::TruthLayer,
    store::{FactDslStore, InMemoryFactDslStore},
};

#[tokio::test]
async fn public_default_memory_pipeline_builds_records() {
    let pipeline = DefaultMemoryPipeline::default_v1();

    let record = pipeline
        .build_record(
            TruthLayer::T2,
            "roadmap#phase9",
            "2026-04 use lexical-first retrieval as the baseline decision because explainability matters.",
        )
        .await
        .expect("default memory pipeline should build a record");

    assert_eq!(record.taxonomy.domain.as_str(), "project");
    assert_eq!(record.taxonomy.topic.as_str(), "retrieval");
    assert_eq!(record.taxonomy.aspect.as_str(), "behavior");
    assert_eq!(record.taxonomy.kind.as_str(), "decision");
}

#[tokio::test]
async fn public_default_memory_pipeline_can_emit_encoded_dsl() {
    let pipeline = DefaultMemoryPipeline::default_v1();

    let encoded = pipeline
        .build_encoded(
            TruthLayer::T2,
            "roadmap#phase9",
            "2026-04 use lexical-first retrieval as the baseline decision because explainability matters.",
        )
        .await
        .expect("default pipeline should emit DSL");

    assert!(encoded.contains("CLAIM=2026-04 use lexical-first retrieval as the baseline decision because explainability matters."));
    assert!(encoded.contains("WHY=explainability matters."));
}

#[tokio::test]
async fn public_default_memory_pipeline_can_emit_full_report() {
    let pipeline = DefaultMemoryPipeline::default_v1();

    let report = pipeline
        .build_report(
            TruthLayer::T2,
            "roadmap#phase9",
            "2026-04 use lexical-first retrieval as the baseline decision because explainability matters.",
        )
        .await
        .expect("default pipeline should emit a report");

    assert_eq!(report.classification.taxonomy.kind.as_str(), "decision");
    assert_eq!(report.record.taxonomy.kind.as_str(), "decision");
    assert!(report.encoded.starts_with("F|DOM=project|TOP=retrieval"));
}

#[tokio::test]
async fn public_default_memory_pipeline_can_emit_flattened_records() {
    let pipeline = DefaultMemoryPipeline::default_v1();

    let flat = pipeline
        .build_flattened(
            TruthLayer::T2,
            "roadmap#phase9",
            "2026-04 use lexical-first retrieval as the baseline decision because explainability matters.",
        )
        .await
        .expect("default pipeline should emit flattened record");

    assert_eq!(flat.domain, "project");
    assert_eq!(flat.topic, "retrieval");
    assert_eq!(flat.kind, "decision");
}

#[tokio::test]
async fn public_default_memory_pipeline_can_emit_json_reports() {
    let pipeline = DefaultMemoryPipeline::default_v1();

    let json = pipeline
        .build_json_report(
            TruthLayer::T2,
            "roadmap#phase9",
            "2026-04 use lexical-first retrieval as the baseline decision because explainability matters.",
        )
        .await
        .expect("default pipeline should emit JSON report");

    assert!(json.contains("\"classification\""));
    assert!(json.contains("\"encoded\""));
}

#[tokio::test]
async fn public_default_memory_pipeline_can_emit_persisted_wrappers() {
    let pipeline = DefaultMemoryPipeline::default_v1();

    let persisted = pipeline
        .build_persisted(
            "mem-1",
            TruthLayer::T2,
            "roadmap#phase9",
            "2026-04 use lexical-first retrieval as the baseline decision because explainability matters.",
        )
        .await
        .expect("default pipeline should emit persisted wrapper");

    assert_eq!(persisted.record_id, "mem-1");
    assert_eq!(persisted.payload.domain, "project");
    assert_eq!(persisted.payload.kind, "decision");
}

#[tokio::test]
async fn public_default_memory_pipeline_can_persist_via_store_contract() {
    let pipeline = DefaultMemoryPipeline::default_v1();
    let store = InMemoryFactDslStore::new();

    let persisted = pipeline
        .persist_with_store(
            &store,
            "mem-1",
            TruthLayer::T2,
            "roadmap#phase9",
            "2026-04 use lexical-first retrieval as the baseline decision because explainability matters.",
        )
        .await
        .expect("default pipeline should persist via store contract");

    let loaded = store
        .get_fact_dsl("mem-1")
        .expect("lookup should succeed")
        .expect("row should exist");
    assert_eq!(loaded, persisted);
}

#[tokio::test]
async fn public_default_memory_pipeline_can_bulk_persist_memory_records() {
    use agent_memos::memory::record::{
        MemoryRecord, Provenance, RecordTimestamp, RecordType, Scope, SourceKind, SourceRef,
        ValidityWindow,
    };

    let pipeline = DefaultMemoryPipeline::default_v1();
    let store = InMemoryFactDslStore::new();
    let make_record = |id: &str, text: &str| MemoryRecord {
        id: id.to_string(),
        source: SourceRef {
            uri: format!("memo://project/{id}"),
            kind: SourceKind::Note,
            label: Some(format!("note-{id}")),
        },
        timestamp: RecordTimestamp {
            recorded_at: "2026-04-19T00:00:00Z".to_string(),
            created_at: "2026-04-19T00:00:00Z".to_string(),
            updated_at: "2026-04-19T00:00:00Z".to_string(),
        },
        scope: Scope::Project,
        record_type: RecordType::Observation,
        truth_layer: TruthLayer::T2,
        provenance: Provenance {
            origin: "test".to_string(),
            imported_via: None,
            derived_from: Vec::new(),
        },
        content_text: text.to_string(),
        chunk: None,
        validity: ValidityWindow::default(),
    };

    let first = make_record(
        "mem-1",
        "2026-04 use lexical-first retrieval as the baseline decision because explainability matters.",
    );
    let second = make_record(
        "mem-2",
        "2026-04 keep retrieval debuggable because citations must stay stable.",
    );

    let persisted = pipeline
        .persist_many_memory_records_with_store(&store, &[first, second])
        .await
        .expect("default pipeline should persist many memory records");

    assert_eq!(persisted.len(), 2);
    let listed = store.list_fact_dsls().expect("listing should succeed");
    assert_eq!(listed, persisted);
}

#[tokio::test]
async fn public_default_memory_pipeline_can_bulk_persist_raw_inputs() {
    let pipeline = DefaultMemoryPipeline::default_v1();
    let store = InMemoryFactDslStore::new();

    let persisted = pipeline
        .persist_many_with_store(
            &store,
            TruthLayer::T2,
            &[
                (
                    "mem-1".to_string(),
                    "roadmap#phase9".to_string(),
                    "2026-04 use lexical-first retrieval as the baseline decision because explainability matters.".to_string(),
                ),
                (
                    "mem-2".to_string(),
                    "notes#phase9".to_string(),
                    "2026-04 keep retrieval debuggable because citations must stay stable.".to_string(),
                ),
            ],
        )
        .await
        .expect("default pipeline should bulk persist raw inputs");

    assert_eq!(persisted.len(), 2);
    let listed = store.list_fact_dsls().expect("listing should succeed");
    assert_eq!(listed, persisted);
}
