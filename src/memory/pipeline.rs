use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::memory::{
    classifier::{
        ClassificationError, ClassificationInput, KeywordTaxonomyClassifier, TaxonomyClassifier,
    },
    dsl::{FactDslError, FactDslRecord, FlatFactDslRecordV1, KindFieldAssessmentV1},
    record::{MemoryRecord, TruthLayer},
    store::{FactDslStore, FactDslStoreError, PersistedFactDslRecordV1},
    summary::{
        FactSummaryError, FactSummaryGenerator, FactSummaryInput, RuleBasedSummaryGenerator,
    },
};

pub async fn build_fact_dsl_record<C, S>(
    classifier: &C,
    summarizer: &S,
    truth_layer: TruthLayer,
    source_ref: impl Into<String>,
    raw_text: impl Into<String>,
) -> Result<FactDslRecord, MemoryPipelineError>
where
    C: TaxonomyClassifier + Sync,
    S: FactSummaryGenerator + Sync,
{
    let source_ref = source_ref.into();
    let raw_text = raw_text.into();

    let classification_input = ClassificationInput::new(source_ref.clone(), raw_text.clone());
    classification_input
        .validate()
        .map_err(MemoryPipelineError::Classification)?;

    let classification = classifier
        .classify(&classification_input)
        .await
        .map_err(MemoryPipelineError::Classification)?;

    let summary_input = classification
        .into_summary_input(truth_layer, source_ref, raw_text)
        .map_err(MemoryPipelineError::Classification)?;

    let draft = summarizer
        .summarize(&summary_input)
        .await
        .map_err(MemoryPipelineError::Summary)?;

    summary_input
        .into_record(draft)
        .map_err(MemoryPipelineError::Summary)
}

pub async fn build_fact_dsl_record_from_memory_record<C, S>(
    classifier: &C,
    summarizer: &S,
    record: &MemoryRecord,
) -> Result<FactDslRecord, MemoryPipelineError>
where
    C: TaxonomyClassifier + Sync,
    S: FactSummaryGenerator + Sync,
{
    build_fact_dsl_record(
        classifier,
        summarizer,
        record.truth_layer,
        &record.source.uri,
        &record.content_text,
    )
    .await
}

pub async fn build_memory_report<C, S>(
    classifier: &C,
    summarizer: &S,
    truth_layer: TruthLayer,
    source_ref: impl Into<String>,
    raw_text: impl Into<String>,
) -> Result<MemoryPipelineReport, MemoryPipelineError>
where
    C: TaxonomyClassifier + Sync,
    S: FactSummaryGenerator + Sync,
{
    let source_ref = source_ref.into();
    let raw_text = raw_text.into();

    let classification_input = ClassificationInput::new(source_ref.clone(), raw_text.clone());
    classification_input
        .validate()
        .map_err(MemoryPipelineError::Classification)?;

    let classification = classifier
        .classify(&classification_input)
        .await
        .map_err(MemoryPipelineError::Classification)?;

    let summary_input = classification
        .clone()
        .into_summary_input(truth_layer, source_ref, raw_text)
        .map_err(MemoryPipelineError::Classification)?;

    let draft = summarizer
        .summarize(&summary_input)
        .await
        .map_err(MemoryPipelineError::Summary)?;

    let record = summary_input
        .clone()
        .into_record(draft)
        .map_err(MemoryPipelineError::Summary)?;
    let assessment = record.assess_kind_fields();
    let encoded = record.encode().map_err(MemoryPipelineError::Dsl)?;

    Ok(MemoryPipelineReport {
        classification,
        summary_input,
        record,
        assessment,
        encoded,
    })
}

pub async fn build_memory_report_from_memory_record<C, S>(
    classifier: &C,
    summarizer: &S,
    record: &MemoryRecord,
) -> Result<MemoryPipelineReport, MemoryPipelineError>
where
    C: TaxonomyClassifier + Sync,
    S: FactSummaryGenerator + Sync,
{
    build_memory_report(
        classifier,
        summarizer,
        record.truth_layer,
        &record.source.uri,
        &record.content_text,
    )
    .await
}

pub struct MemoryPipeline<C, S> {
    classifier: C,
    summarizer: S,
}

impl<C, S> MemoryPipeline<C, S> {
    pub fn new(classifier: C, summarizer: S) -> Self {
        Self {
            classifier,
            summarizer,
        }
    }
}

impl<C, S> MemoryPipeline<C, S>
where
    C: TaxonomyClassifier + Sync,
    S: FactSummaryGenerator + Sync,
{
    pub async fn build_report(
        &self,
        truth_layer: TruthLayer,
        source_ref: impl Into<String>,
        raw_text: impl Into<String>,
    ) -> Result<MemoryPipelineReport, MemoryPipelineError> {
        build_memory_report(
            &self.classifier,
            &self.summarizer,
            truth_layer,
            source_ref,
            raw_text,
        )
        .await
    }

    pub async fn build_record(
        &self,
        truth_layer: TruthLayer,
        source_ref: impl Into<String>,
        raw_text: impl Into<String>,
    ) -> Result<FactDslRecord, MemoryPipelineError> {
        build_fact_dsl_record(
            &self.classifier,
            &self.summarizer,
            truth_layer,
            source_ref,
            raw_text,
        )
        .await
    }

    pub async fn build_record_from_memory_record(
        &self,
        record: &MemoryRecord,
    ) -> Result<FactDslRecord, MemoryPipelineError> {
        build_fact_dsl_record_from_memory_record(&self.classifier, &self.summarizer, record).await
    }

    pub async fn build_report_from_memory_record(
        &self,
        record: &MemoryRecord,
    ) -> Result<MemoryPipelineReport, MemoryPipelineError> {
        build_memory_report_from_memory_record(&self.classifier, &self.summarizer, record).await
    }

    pub async fn build_encoded_from_memory_record(
        &self,
        record: &MemoryRecord,
    ) -> Result<String, MemoryPipelineError> {
        let report = self.build_report_from_memory_record(record).await?;
        Ok(report.encoded)
    }

    pub async fn build_encoded(
        &self,
        truth_layer: TruthLayer,
        source_ref: impl Into<String>,
        raw_text: impl Into<String>,
    ) -> Result<String, MemoryPipelineError> {
        let report = self.build_report(truth_layer, source_ref, raw_text).await?;
        Ok(report.encoded)
    }

    pub async fn build_flattened(
        &self,
        truth_layer: TruthLayer,
        source_ref: impl Into<String>,
        raw_text: impl Into<String>,
    ) -> Result<FlatFactDslRecordV1, MemoryPipelineError> {
        let report = self.build_report(truth_layer, source_ref, raw_text).await?;
        Ok(report.flattened_record())
    }

    pub async fn build_flattened_from_memory_record(
        &self,
        record: &MemoryRecord,
    ) -> Result<FlatFactDslRecordV1, MemoryPipelineError> {
        let report = self.build_report_from_memory_record(record).await?;
        Ok(report.flattened_record())
    }

    pub async fn build_json_report(
        &self,
        truth_layer: TruthLayer,
        source_ref: impl Into<String>,
        raw_text: impl Into<String>,
    ) -> Result<String, MemoryPipelineError> {
        let report = self.build_report(truth_layer, source_ref, raw_text).await?;
        report.to_json_string()
    }

    pub async fn build_json_report_from_memory_record(
        &self,
        record: &MemoryRecord,
    ) -> Result<String, MemoryPipelineError> {
        let report = self.build_report_from_memory_record(record).await?;
        report.to_json_string()
    }

    pub async fn build_persisted(
        &self,
        record_id: impl Into<String>,
        truth_layer: TruthLayer,
        source_ref: impl Into<String>,
        raw_text: impl Into<String>,
    ) -> Result<PersistedFactDslRecordV1, MemoryPipelineError> {
        let record = self.build_record(truth_layer, source_ref, raw_text).await?;
        PersistedFactDslRecordV1::from_fact_dsl_record(record_id, &record)
            .map_err(MemoryPipelineError::Store)
    }

    pub async fn build_persisted_from_memory_record(
        &self,
        record: &MemoryRecord,
    ) -> Result<PersistedFactDslRecordV1, MemoryPipelineError> {
        let built = self.build_record_from_memory_record(record).await?;
        PersistedFactDslRecordV1::from_fact_dsl_record(&record.id, &built)
            .map_err(MemoryPipelineError::Store)
    }

    pub async fn persist_with_store<T: FactDslStore>(
        &self,
        store: &T,
        record_id: impl Into<String>,
        truth_layer: TruthLayer,
        source_ref: impl Into<String>,
        raw_text: impl Into<String>,
    ) -> Result<PersistedFactDslRecordV1, MemoryPipelineError> {
        let persisted = self
            .build_persisted(record_id, truth_layer, source_ref, raw_text)
            .await?;
        store
            .put_fact_dsl(&persisted)
            .map_err(MemoryPipelineError::Store)?;
        Ok(persisted)
    }

    pub async fn persist_memory_record_with_store<T: FactDslStore>(
        &self,
        store: &T,
        record: &MemoryRecord,
    ) -> Result<PersistedFactDslRecordV1, MemoryPipelineError> {
        let persisted = self.build_persisted_from_memory_record(record).await?;
        store
            .put_fact_dsl(&persisted)
            .map_err(MemoryPipelineError::Store)?;
        Ok(persisted)
    }

    pub async fn persist_many_memory_records_with_store<T: FactDslStore>(
        &self,
        store: &T,
        records: &[MemoryRecord],
    ) -> Result<Vec<PersistedFactDslRecordV1>, MemoryPipelineError> {
        let mut persisted = Vec::with_capacity(records.len());
        for record in records {
            persisted.push(self.build_persisted_from_memory_record(record).await?);
        }
        store
            .put_many_fact_dsls(&persisted)
            .map_err(MemoryPipelineError::Store)?;
        Ok(persisted)
    }

    pub async fn persist_many_with_store<T: FactDslStore>(
        &self,
        store: &T,
        truth_layer: TruthLayer,
        inputs: &[(String, String, String)],
    ) -> Result<Vec<PersistedFactDslRecordV1>, MemoryPipelineError> {
        let mut persisted = Vec::with_capacity(inputs.len());
        for (record_id, source_ref, raw_text) in inputs {
            persisted.push(
                self.build_persisted(
                    record_id.clone(),
                    truth_layer,
                    source_ref.clone(),
                    raw_text.clone(),
                )
                .await?,
            );
        }
        store
            .put_many_fact_dsls(&persisted)
            .map_err(MemoryPipelineError::Store)?;
        Ok(persisted)
    }
}

pub type DefaultMemoryPipeline =
    MemoryPipeline<KeywordTaxonomyClassifier, RuleBasedSummaryGenerator>;

impl DefaultMemoryPipeline {
    pub fn default_v1() -> Self {
        Self::new(
            KeywordTaxonomyClassifier::default(),
            RuleBasedSummaryGenerator,
        )
    }

    pub fn build_record_sync_from_memory_record(
        &self,
        record: &MemoryRecord,
    ) -> Result<FactDslRecord, MemoryPipelineError> {
        let classification = self
            .classifier
            .classify_record_sync(record)
            .map_err(MemoryPipelineError::Classification)?;
        let summary_input = classification
            .into_summary_input(record.truth_layer, &record.source.uri, &record.content_text)
            .map_err(MemoryPipelineError::Classification)?;
        let draft = self
            .summarizer
            .summarize_sync(&summary_input)
            .map_err(MemoryPipelineError::Summary)?;
        summary_input
            .into_record(draft)
            .map_err(MemoryPipelineError::Summary)
    }

    pub fn build_persisted_sync_from_memory_record(
        &self,
        record: &MemoryRecord,
    ) -> Result<PersistedFactDslRecordV1, MemoryPipelineError> {
        let classification = self
            .classifier
            .classify_record_sync(record)
            .map_err(MemoryPipelineError::Classification)?;
        let summary_input = classification
            .clone()
            .into_summary_input(record.truth_layer, &record.source.uri, &record.content_text)
            .map_err(MemoryPipelineError::Classification)?;
        let draft = self
            .summarizer
            .summarize_sync(&summary_input)
            .map_err(MemoryPipelineError::Summary)?;
        let built = summary_input
            .into_record(draft)
            .map_err(MemoryPipelineError::Summary)?;

        let mut persisted = PersistedFactDslRecordV1::from_fact_dsl_record(&record.id, &built)
            .map_err(MemoryPipelineError::Store)?;
        persisted.classification_confidence = Some(classification.confidence);
        persisted.needs_review = classification.needs_review;
        persisted.validate().map_err(MemoryPipelineError::Store)?;
        Ok(persisted)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryPipelineReport {
    pub classification: crate::memory::classifier::ClassificationOutput,
    pub summary_input: FactSummaryInput,
    pub record: FactDslRecord,
    pub assessment: KindFieldAssessmentV1,
    pub encoded: String,
}

impl MemoryPipelineReport {
    pub fn flattened_record(&self) -> FlatFactDslRecordV1 {
        self.record.flatten()
    }

    pub fn into_flattened_record(self) -> FlatFactDslRecordV1 {
        self.record.flatten()
    }

    pub fn into_encoded(self) -> String {
        self.encoded
    }

    pub fn into_persisted(
        self,
        record_id: impl Into<String>,
    ) -> Result<PersistedFactDslRecordV1, MemoryPipelineError> {
        let mut persisted = PersistedFactDslRecordV1::from_fact_dsl_record(record_id, &self.record)
            .map_err(MemoryPipelineError::Store)?;
        persisted.classification_confidence = Some(self.classification.confidence);
        persisted.needs_review = self.classification.needs_review;
        persisted.validate().map_err(MemoryPipelineError::Store)?;
        Ok(persisted)
    }

    pub fn to_json_string(&self) -> Result<String, MemoryPipelineError> {
        serde_json::to_string(self).map_err(MemoryPipelineError::Json)
    }

    pub fn from_json_str(value: &str) -> Result<Self, MemoryPipelineError> {
        serde_json::from_str(value).map_err(MemoryPipelineError::Json)
    }
}

#[derive(Debug, Error)]
pub enum MemoryPipelineError {
    #[error(transparent)]
    Classification(#[from] ClassificationError),
    #[error(transparent)]
    Dsl(#[from] FactDslError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Store(#[from] FactDslStoreError),
    #[error(transparent)]
    Summary(#[from] FactSummaryError),
}

#[cfg(test)]
mod tests {
    use std::{future::Future, pin::Pin};

    use super::*;
    use crate::memory::{
        classifier::{ClassificationOutput, TaxonomyClassifier},
        dsl::FactDslDraft,
        store::{FactDslStore, InMemoryFactDslStore},
        summary::{FactSummaryGenerator, FactSummaryInput},
        taxonomy::{AspectV1, DomainV1, KindV1, TaxonomyPathV1, TopicV1},
    };

    struct StubClassifier;

    impl TaxonomyClassifier for StubClassifier {
        fn classify<'a>(
            &'a self,
            _input: &'a ClassificationInput,
        ) -> Pin<
            Box<dyn Future<Output = Result<ClassificationOutput, ClassificationError>> + Send + 'a>,
        > {
            Box::pin(async move {
                ClassificationOutput::new(
                    TaxonomyPathV1::new(
                        DomainV1::Project,
                        TopicV1::Retrieval,
                        AspectV1::Behavior,
                        KindV1::Decision,
                    )
                    .expect("stub taxonomy should be valid"),
                    0.9,
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
    async fn pipeline_builds_fact_dsl_record_from_trait_boundaries() {
        let record = build_fact_dsl_record(
            &StubClassifier,
            &StubSummarizer,
            TruthLayer::T2,
            "roadmap#phase9",
            "Use lexical-first as baseline because explainability matters.",
        )
        .await
        .expect("pipeline should produce DSL record");

        assert_eq!(record.taxonomy.kind, KindV1::Decision);
        assert_eq!(record.truth_layer, TruthLayer::T2);
        assert_eq!(record.source_ref, "roadmap#phase9");
        assert_eq!(record.draft.why.as_deref(), Some("explainability matters"));
    }

    #[tokio::test]
    async fn report_collects_classification_record_assessment_and_encoding() {
        let report = build_memory_report(
            &StubClassifier,
            &StubSummarizer,
            TruthLayer::T2,
            "roadmap#phase9",
            "Use lexical-first as baseline because explainability matters.",
        )
        .await
        .expect("report should build");

        assert_eq!(report.classification.taxonomy.kind, KindV1::Decision);
        assert_eq!(report.record.taxonomy.kind, KindV1::Decision);
        assert!(
            report
                .assessment
                .missing_recommended
                .contains(&crate::memory::dsl::DslFieldV1::Impact)
        );
        assert!(report.encoded.starts_with("F|DOM=project|TOP=retrieval"));
    }

    #[tokio::test]
    async fn default_pipeline_provides_ready_to_run_memory_flow() {
        let pipeline = DefaultMemoryPipeline::default_v1();

        let record = pipeline
            .build_record(
                TruthLayer::T2,
                "roadmap#phase9",
                "2026-04 use lexical-first retrieval as the baseline decision because explainability matters.",
            )
            .await
            .expect("default pipeline should build DSL record");

        assert_eq!(record.taxonomy.domain.as_str(), "project");
        assert_eq!(record.taxonomy.topic.as_str(), "retrieval");
        assert_eq!(record.taxonomy.kind.as_str(), "decision");
        assert_eq!(record.draft.time.as_deref(), Some("2026-04"));
    }

    #[tokio::test]
    async fn default_pipeline_can_emit_encoded_dsl_directly() {
        let pipeline = DefaultMemoryPipeline::default_v1();

        let encoded = pipeline
            .build_encoded(
                TruthLayer::T2,
                "roadmap#phase9",
                "2026-04 use lexical-first retrieval as the baseline decision because explainability matters.",
            )
            .await
            .expect("default pipeline should emit encoded DSL");

        assert!(encoded.starts_with("F|DOM=project|TOP=retrieval|ASP=behavior|KIND=decision"));
    }

    #[tokio::test]
    async fn report_serializes_to_json_and_back() {
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
        let rebuilt = MemoryPipelineReport::from_json_str(&json).expect("json should parse");

        assert_eq!(rebuilt, report);
    }

    #[tokio::test]
    async fn report_exposes_flattened_record_view() {
        let pipeline = DefaultMemoryPipeline::default_v1();
        let report = pipeline
            .build_report(
                TruthLayer::T2,
                "roadmap#phase9",
                "2026-04 use lexical-first retrieval as the baseline decision because explainability matters.",
            )
            .await
            .expect("report should build");

        let flat = report.flattened_record();
        assert_eq!(flat.domain, "project");
        assert_eq!(flat.topic, "retrieval");
        assert_eq!(flat.kind, "decision");
    }

    #[tokio::test]
    async fn default_pipeline_can_emit_persisted_records() {
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
    async fn report_can_be_consumed_into_flattened_or_encoded_outputs() {
        let pipeline = DefaultMemoryPipeline::default_v1();
        let report = pipeline
            .build_report(
                TruthLayer::T2,
                "roadmap#phase9",
                "2026-04 use lexical-first retrieval as the baseline decision because explainability matters.",
            )
            .await
            .expect("report should build");

        let flat = report.clone().into_flattened_record();
        let encoded = report.into_encoded();

        assert_eq!(flat.domain, "project");
        assert!(encoded.starts_with("F|DOM=project|TOP=retrieval"));
    }

    #[tokio::test]
    async fn report_can_be_consumed_into_persisted_wrapper() {
        let pipeline = DefaultMemoryPipeline::default_v1();
        let report = pipeline
            .build_report(
                TruthLayer::T2,
                "roadmap#phase9",
                "2026-04 use lexical-first retrieval as the baseline decision because explainability matters.",
            )
            .await
            .expect("report should build");

        let persisted = report
            .into_persisted("mem-1")
            .expect("report should convert into persisted wrapper");

        assert_eq!(persisted.record_id, "mem-1");
        assert_eq!(persisted.payload.domain, "project");
        assert_eq!(persisted.payload.kind, "decision");
    }

    #[tokio::test]
    async fn default_pipeline_can_emit_json_report_directly() {
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
    async fn default_pipeline_can_emit_flattened_record_directly() {
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
    async fn default_pipeline_can_persist_through_store_contract() {
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
            .expect("default pipeline should persist through store contract");

        let loaded = store
            .get_fact_dsl("mem-1")
            .expect("lookup should succeed")
            .expect("row should exist");

        assert_eq!(loaded, persisted);
    }

    #[tokio::test]
    async fn pipeline_builds_record_from_memory_record() {
        let record = MemoryRecord {
            id: "mem-1".to_string(),
            source: crate::memory::record::SourceRef {
                uri: "memo://project/retrieval".to_string(),
                kind: crate::memory::record::SourceKind::Note,
                label: Some("retrieval note".to_string()),
            },
            timestamp: crate::memory::record::RecordTimestamp {
                recorded_at: "2026-04-19T00:00:00Z".to_string(),
                created_at: "2026-04-19T00:00:00Z".to_string(),
                updated_at: "2026-04-19T00:00:00Z".to_string(),
            },
            scope: crate::memory::record::Scope::Project,
            record_type: crate::memory::record::RecordType::Observation,
            truth_layer: TruthLayer::T2,
            provenance: crate::memory::record::Provenance {
                origin: "test".to_string(),
                imported_via: None,
                derived_from: Vec::new(),
            },
            content_text: "Use lexical-first as baseline because explainability matters."
                .to_string(),
            chunk: None,
            validity: crate::memory::record::ValidityWindow::default(),
        };

        let built =
            build_fact_dsl_record_from_memory_record(&StubClassifier, &StubSummarizer, &record)
                .await
                .expect("pipeline should build from memory record");

        assert_eq!(built.truth_layer, TruthLayer::T2);
        assert_eq!(built.source_ref, "memo://project/retrieval");
    }

    #[tokio::test]
    async fn pipeline_builds_report_from_memory_record() {
        let record = MemoryRecord {
            id: "mem-1".to_string(),
            source: crate::memory::record::SourceRef {
                uri: "memo://project/retrieval".to_string(),
                kind: crate::memory::record::SourceKind::Note,
                label: Some("retrieval note".to_string()),
            },
            timestamp: crate::memory::record::RecordTimestamp {
                recorded_at: "2026-04-19T00:00:00Z".to_string(),
                created_at: "2026-04-19T00:00:00Z".to_string(),
                updated_at: "2026-04-19T00:00:00Z".to_string(),
            },
            scope: crate::memory::record::Scope::Project,
            record_type: crate::memory::record::RecordType::Observation,
            truth_layer: TruthLayer::T2,
            provenance: crate::memory::record::Provenance {
                origin: "test".to_string(),
                imported_via: None,
                derived_from: Vec::new(),
            },
            content_text: "2026-04 use lexical-first as baseline because explainability matters."
                .to_string(),
            chunk: None,
            validity: crate::memory::record::ValidityWindow::default(),
        };

        let pipeline = DefaultMemoryPipeline::default_v1();
        let report = pipeline
            .build_report_from_memory_record(&record)
            .await
            .expect("pipeline should build report from memory record");

        assert_eq!(report.record.source_ref, "memo://project/retrieval");
        assert!(report.encoded.starts_with("F|DOM=project|TOP=retrieval"));
    }
}
