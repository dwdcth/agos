use agent_memos::memory::{pipeline::DefaultMemoryPipeline, record::TruthLayer};

#[tokio::test]
async fn public_pipeline_report_exposes_assessment_and_encoded_output() {
    let pipeline = DefaultMemoryPipeline::default_v1();

    let report = pipeline
        .build_report(
            TruthLayer::T2,
            "roadmap#phase9",
            "2026-04 use lexical-first retrieval as the baseline decision because explainability matters.",
        )
        .await
        .expect("pipeline should produce report");

    assert_eq!(report.classification.taxonomy.kind.as_str(), "decision");
    assert!(report
        .assessment
        .missing_recommended
        .iter()
        .any(|field| field.as_str() == "IMPACT"));
    assert!(report.encoded.starts_with("F|DOM=project|TOP=retrieval"));
    let flat = report.flattened_record();
    assert_eq!(flat.domain, "project");
    assert_eq!(flat.kind, "decision");
}

#[tokio::test]
async fn public_pipeline_report_can_be_consumed_without_recomputing() {
    let pipeline = DefaultMemoryPipeline::default_v1();

    let report = pipeline
        .build_report(
            TruthLayer::T2,
            "roadmap#phase9",
            "2026-04 use lexical-first retrieval as the baseline decision because explainability matters.",
        )
        .await
        .expect("pipeline should produce report");

    let flat = report.clone().into_flattened_record();
    let encoded = report.into_encoded();

    assert_eq!(flat.topic, "retrieval");
    assert!(encoded.starts_with("F|DOM=project|TOP=retrieval"));
}

#[tokio::test]
async fn public_pipeline_report_can_be_consumed_into_persisted_wrapper() {
    let pipeline = DefaultMemoryPipeline::default_v1();

    let report = pipeline
        .build_report(
            TruthLayer::T2,
            "roadmap#phase9",
            "2026-04 use lexical-first retrieval as the baseline decision because explainability matters.",
        )
        .await
        .expect("pipeline should produce report");

    let persisted = report
        .into_persisted("mem-1")
        .expect("report should convert into persisted wrapper");

    assert_eq!(persisted.record_id, "mem-1");
    assert_eq!(persisted.payload.domain, "project");
    assert_eq!(persisted.payload.kind, "decision");
}
