use std::{future::Future, pin::Pin};

use agent_memos::memory::{
    classifier::{ClassificationError, ClassificationInput, ClassificationOutput, TaxonomyClassifier},
    dsl::FactDslDraft,
    pipeline::build_fact_dsl_record,
    record::TruthLayer,
    summary::{FactSummaryError, FactSummaryGenerator, FactSummaryInput},
    taxonomy::{AspectV1, DomainV1, KindV1, TaxonomyPathV1, TopicV1},
};

struct StubClassifier;

impl TaxonomyClassifier for StubClassifier {
    fn classify<'a>(
        &'a self,
        _input: &'a ClassificationInput,
    ) -> Pin<Box<dyn Future<Output = Result<ClassificationOutput, ClassificationError>> + Send + 'a>>
    {
        Box::pin(async move {
            ClassificationOutput::new(
                TaxonomyPathV1::new(
                    DomainV1::Project,
                    TopicV1::Retrieval,
                    AspectV1::Behavior,
                    KindV1::Decision,
                )
                .expect("stub taxonomy should be valid"),
                0.84,
                false,
            )
        })
    }
}

struct StubSummarizer;

impl FactSummaryGenerator for StubSummarizer {
    fn summarize<'a>(
        &'a self,
        _input: &'a FactSummaryInput,
    ) -> Pin<Box<dyn Future<Output = Result<FactDslDraft, FactSummaryError>> + Send + 'a>>
    {
        Box::pin(async move {
            Ok(FactDslDraft {
                claim: "use lexical-first as baseline".to_string(),
                why: Some("explainability matters".to_string()),
                time: Some("2026-04".to_string()),
                ..Default::default()
            })
        })
    }
}

#[tokio::test]
async fn public_memory_pipeline_builder_connects_classifier_and_summary() {
    let record = build_fact_dsl_record(
        &StubClassifier,
        &StubSummarizer,
        TruthLayer::T2,
        "roadmap#phase9",
        "Use lexical-first as baseline because explainability matters.",
    )
    .await
    .expect("pipeline should produce DSL record");

    assert_eq!(record.taxonomy.kind.as_str(), "decision");
    assert_eq!(record.draft.claim, "use lexical-first as baseline");
    assert_eq!(record.draft.time.as_deref(), Some("2026-04"));
}
