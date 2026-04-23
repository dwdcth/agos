use agent_memos::memory::{
    pipeline::DefaultMemoryPipeline,
    record::{
        MemoryRecord, Provenance, RecordTimestamp, RecordType, Scope, SourceKind, SourceRef,
        TruthLayer, ValidityWindow,
    },
};

fn sample_record() -> MemoryRecord {
    MemoryRecord {
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
        content_text:
            "2026-04 use lexical-first as the baseline decision because explainability matters."
                .to_string(),
        chunk: None,
        validity: ValidityWindow::default(),
    }
}

#[tokio::test]
async fn public_memory_pipeline_emits_flattened_and_json_outputs_from_memory_record() {
    let pipeline = DefaultMemoryPipeline::default_v1();
    let record = sample_record();

    let flat = pipeline
        .build_flattened_from_memory_record(&record)
        .await
        .expect("pipeline should emit flattened record");
    let json = pipeline
        .build_json_report_from_memory_record(&record)
        .await
        .expect("pipeline should emit JSON report");

    assert_eq!(flat.domain, "project");
    assert_eq!(flat.topic, "retrieval");
    assert!(json.contains("\"classification\""));
    assert!(json.contains("\"encoded\""));
}
