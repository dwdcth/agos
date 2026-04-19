use agent_memos::memory::{pipeline::DefaultMemoryPipeline, record::TruthLayer};

#[tokio::test]
async fn public_memory_pipeline_report_supports_json_round_trip() {
    let pipeline = DefaultMemoryPipeline::default_v1();
    let report = pipeline
        .build_report(
            TruthLayer::T2,
            "roadmap#phase9",
            "2026-04 use lexical-first retrieval as the baseline decision because explainability matters.",
        )
        .await
        .expect("report should build");

    let json = report.to_json_string().expect("report should serialize");
    let rebuilt = agent_memos::memory::pipeline::MemoryPipelineReport::from_json_str(&json)
        .expect("json should parse");

    assert_eq!(rebuilt, report);
}
