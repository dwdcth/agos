use agent_memos::memory::{
    pipeline::DefaultMemoryPipeline,
    record::{
        MemoryRecord, Provenance, RecordTimestamp, RecordType, Scope, SourceKind, SourceRef,
        TruthLayer, ValidityWindow,
    },
};

#[tokio::test]
async fn public_memory_pipeline_accepts_memory_record_entrypoint() {
    let record = MemoryRecord {
        id: "mem-1".to_string(),
        source: SourceRef {
            uri: "memo://project/retrieval".to_string(),
            kind: SourceKind::Note,
            label: Some("retrieval note".to_string()),
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
        content_text: "Use lexical-first as baseline because explainability matters.".to_string(),
        chunk: None,
        validity: ValidityWindow::default(),
    };

    let pipeline = DefaultMemoryPipeline::default_v1();
    let built = pipeline
        .build_record_from_memory_record(&record)
        .await
        .expect("default pipeline should accept memory records");

    assert_eq!(built.source_ref, "memo://project/retrieval");
    assert_eq!(built.truth_layer, TruthLayer::T2);
    assert_eq!(built.taxonomy.topic.as_str(), "retrieval");
}

#[tokio::test]
async fn public_memory_pipeline_emits_report_and_encoded_from_memory_record() {
    let record = MemoryRecord {
        id: "mem-1".to_string(),
        source: SourceRef {
            uri: "memo://project/retrieval".to_string(),
            kind: SourceKind::Note,
            label: Some("retrieval note".to_string()),
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
        content_text: "2026-04 use lexical-first as baseline because explainability matters."
            .to_string(),
        chunk: None,
        validity: ValidityWindow::default(),
    };

    let pipeline = DefaultMemoryPipeline::default_v1();
    let report = pipeline
        .build_report_from_memory_record(&record)
        .await
        .expect("pipeline should emit report from memory record");
    let encoded = pipeline
        .build_encoded_from_memory_record(&record)
        .await
        .expect("pipeline should emit encoded DSL from memory record");

    assert_eq!(report.record.source_ref, "memo://project/retrieval");
    assert_eq!(encoded, report.encoded);
}
