use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::{future::Future, pin::Pin};

use thiserror::Error;
use crate::memory::{
    record::{MemoryRecord, TruthLayer},
    summary::FactSummaryInput,
};
use crate::memory::taxonomy::{AspectV1, DomainV1, KindV1, TaxonomyPathV1, TopicV1};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassificationInput {
    pub source_ref: String,
    pub raw_text: String,
    pub source_kind: Option<String>,
    pub scope: Option<String>,
    pub record_type: Option<String>,
}

impl ClassificationInput {
    pub fn new(source_ref: impl Into<String>, raw_text: impl Into<String>) -> Self {
        Self {
            source_ref: source_ref.into(),
            raw_text: raw_text.into(),
            source_kind: None,
            scope: None,
            record_type: None,
        }
    }

    pub fn validate(&self) -> Result<(), ClassificationError> {
        if self.source_ref.trim().is_empty() {
            return Err(ClassificationError::MissingSourceRef);
        }
        if self.raw_text.trim().is_empty() {
            return Err(ClassificationError::MissingRawText);
        }
        Ok(())
    }

    pub fn from_record(record: &MemoryRecord) -> Self {
        Self {
            source_ref: record.source.uri.clone(),
            raw_text: record.content_text.clone(),
            source_kind: Some(record.source.kind.as_str().to_string()),
            scope: Some(record.scope.as_str().to_string()),
            record_type: Some(record.record_type.as_str().to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassificationOutput {
    pub taxonomy: TaxonomyPathV1,
    pub confidence: f32,
    pub needs_review: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeywordClassificationReport {
    pub domain_hits: Vec<String>,
    pub topic_hits: Vec<String>,
    pub aspect_hits: Vec<String>,
    pub kind_hits: Vec<String>,
}

impl ClassificationOutput {
    pub fn new(
        taxonomy: TaxonomyPathV1,
        confidence: f32,
        needs_review: bool,
    ) -> Result<Self, ClassificationError> {
        let output = Self {
            taxonomy,
            confidence,
            needs_review,
        };
        output.validate()?;
        Ok(output)
    }

    pub fn validate(&self) -> Result<(), ClassificationError> {
        self.taxonomy.validate().map_err(ClassificationError::Taxonomy)?;
        if !(0.0..=1.0).contains(&self.confidence) {
            return Err(ClassificationError::InvalidConfidence(self.confidence));
        }
        Ok(())
    }

    pub fn fallback_observation(domain: DomainV1) -> Self {
        let topic = TopicV1::allowed_for(domain)
            .first()
            .copied()
            .unwrap_or(TopicV1::General);

        Self {
            taxonomy: TaxonomyPathV1::new(domain, topic, AspectV1::General, KindV1::Observation)
                .expect("fallback taxonomy path should stay valid"),
            confidence: 0.0,
            needs_review: true,
        }
    }

    pub fn into_summary_input(
        self,
        truth_layer: TruthLayer,
        source_ref: impl Into<String>,
        raw_text: impl Into<String>,
    ) -> Result<FactSummaryInput, ClassificationError> {
        self.validate()?;

        let input = FactSummaryInput::new(self.taxonomy, truth_layer, source_ref, raw_text);
        input.validate().map_err(ClassificationError::Summary)?;
        Ok(input)
    }
}

pub trait TaxonomyClassifier {
    fn classify<'a>(
        &'a self,
        input: &'a ClassificationInput,
    ) -> Pin<Box<dyn Future<Output = Result<ClassificationOutput, ClassificationError>> + Send + 'a>>;
}

type ClassificationReportFuture<'a> = Pin<
    Box<
        dyn Future<
                Output = Result<
                    (ClassificationOutput, KeywordClassificationReport),
                    ClassificationError,
                >,
            > + Send
            + 'a,
    >,
>;

#[derive(Debug, Clone, Copy)]
pub struct KeywordTaxonomyClassifier {
    pub default_domain: DomainV1,
    pub review_threshold: f32,
}

impl Default for KeywordTaxonomyClassifier {
    fn default() -> Self {
        Self {
            default_domain: DomainV1::Project,
            review_threshold: 0.6,
        }
    }
}

impl KeywordTaxonomyClassifier {
    pub fn new(default_domain: DomainV1) -> Self {
        Self {
            default_domain,
            ..Self::default()
        }
    }

    pub fn with_review_threshold(mut self, review_threshold: f32) -> Self {
        self.review_threshold = review_threshold;
        self
    }

    fn classify_sync(&self, input: &ClassificationInput) -> Result<ClassificationOutput, ClassificationError> {
        Ok(self.classify_with_report_sync(input)?.0)
    }

    fn classify_with_report_sync(
        &self,
        input: &ClassificationInput,
    ) -> Result<(ClassificationOutput, KeywordClassificationReport), ClassificationError> {
        input.validate()?;

        let text = format!(
            "{} {} {} {} {}",
            input.source_ref,
            input.raw_text,
            input.source_kind.as_deref().unwrap_or(""),
            input.scope.as_deref().unwrap_or(""),
            input.record_type.as_deref().unwrap_or(""),
        )
        .to_lowercase();
        let domain = self.classify_domain(&text);
        let topic = self.classify_topic(domain, &text);
        let aspect = self.classify_aspect(&text);
        let kind = self.classify_kind(&text);

        let domain_hits = matched_keywords(&text, domain_keywords(domain));
        let topic_hits = matched_keywords(&text, topic_keywords(domain, topic));
        let aspect_hits = matched_keywords(&text, aspect_keywords(aspect));
        let kind_hits = matched_keywords(&text, kind_keywords(kind));

        let score = [
            domain_hits.len(),
            topic_hits.len(),
            aspect_hits.len(),
            kind_hits.len(),
        ]
        .into_iter()
        .sum::<usize>();

        let confidence = if score == 0 {
            0.0
        } else {
            ((score as f32) / 8.0).min(1.0)
        };
        let needs_review = confidence < self.review_threshold;

        let output = ClassificationOutput::new(
            TaxonomyPathV1::new(domain, topic, aspect, kind)?,
            confidence,
            needs_review,
        )?;

        Ok((
            output,
            KeywordClassificationReport {
                domain_hits,
                topic_hits,
                aspect_hits,
                kind_hits,
            },
        ))
    }

    fn classify_domain(&self, text: &str) -> DomainV1 {
        let candidates = [
            DomainV1::Project,
            DomainV1::System,
            DomainV1::Process,
            DomainV1::External,
        ];

        choose_best(text, &candidates, |domain| domain_keywords(*domain)).unwrap_or(self.default_domain)
    }

    fn classify_topic(&self, domain: DomainV1, text: &str) -> TopicV1 {
        choose_best(text, TopicV1::allowed_for(domain), |topic| topic_keywords(domain, *topic))
            .unwrap_or(TopicV1::General)
    }

    fn classify_aspect(&self, text: &str) -> AspectV1 {
        let candidates = [
            AspectV1::Provider,
            AspectV1::Policy,
            AspectV1::Capability,
            AspectV1::Structure,
            AspectV1::Interface,
            AspectV1::Behavior,
            AspectV1::State,
            AspectV1::Timeline,
            AspectV1::Evidence,
            AspectV1::Cost,
            AspectV1::Risk,
            AspectV1::Constraint,
        ];

        choose_best(text, &candidates, |aspect| aspect_keywords(*aspect)).unwrap_or(AspectV1::General)
    }

    fn classify_kind(&self, text: &str) -> KindV1 {
        let candidates = [
            KindV1::Decision,
            KindV1::Constraint,
            KindV1::Risk,
            KindV1::Hypothesis,
            KindV1::Pattern,
            KindV1::Issue,
            KindV1::Rule,
            KindV1::Fact,
        ];

        choose_best(text, &candidates, |kind| kind_keywords(*kind)).unwrap_or(KindV1::Observation)
    }
}

impl TaxonomyClassifier for KeywordTaxonomyClassifier {
    fn classify<'a>(
        &'a self,
        input: &'a ClassificationInput,
    ) -> Pin<Box<dyn Future<Output = Result<ClassificationOutput, ClassificationError>> + Send + 'a>>
    {
        Box::pin(async move { self.classify_sync(input) })
    }
}

impl KeywordTaxonomyClassifier {
    pub fn classify_with_report<'a>(&'a self, input: &'a ClassificationInput) -> ClassificationReportFuture<'a> {
        Box::pin(async move { self.classify_with_report_sync(input) })
    }

    pub fn classify_record<'a>(
        &'a self,
        record: &'a MemoryRecord,
    ) -> Pin<Box<dyn Future<Output = Result<ClassificationOutput, ClassificationError>> + Send + 'a>> {
        Box::pin(async move {
            let input = ClassificationInput::from_record(record);
            self.classify_sync(&input)
        })
    }

    pub fn classify_record_with_report<'a>(
        &'a self,
        record: &'a MemoryRecord,
    ) -> ClassificationReportFuture<'a> {
        Box::pin(async move {
            let input = ClassificationInput::from_record(record);
            self.classify_with_report_sync(&input)
        })
    }
}

fn choose_best<T: Copy>(
    text: &str,
    candidates: &[T],
    keywords: impl Fn(&T) -> &'static [&'static str],
) -> Option<T> {
    let mut best = None;
    let mut best_score = 0usize;

    for candidate in candidates {
        let score = score_keywords(text, keywords(candidate));
        if score > best_score {
            best_score = score;
            best = Some(*candidate);
        }
    }

    if best_score == 0 {
        None
    } else {
        best
    }
}

fn score_keywords(text: &str, keywords: &'static [&'static str]) -> usize {
    matched_keywords(text, keywords).len()
}

fn matched_keywords(text: &str, keywords: &'static [&'static str]) -> Vec<String> {
    let tokens = text
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .collect::<BTreeSet<_>>();

    keywords
        .iter()
        .copied()
        .filter(|keyword| {
            if keyword.contains(' ') {
                text.contains(*keyword)
            } else {
                tokens.contains(*keyword)
            }
        })
        .map(ToOwned::to_owned)
        .collect()
}

fn domain_keywords(domain: DomainV1) -> &'static [&'static str] {
    match domain {
        DomainV1::Project => &[
            "roadmap", "phase", "project", "memory", "retrieval", "agent", "truth", "config",
            "docs", "testing",
        ],
        DomainV1::System => &[
            "runtime", "architecture", "storage", "model", "security", "performance",
            "integration", "database", "latency",
        ],
        DomainV1::Process => &[
            "plan", "planning", "implement", "verification", "review", "experiment",
            "benchmark", "workflow",
        ],
        DomainV1::External => &[
            "provider", "dependency", "api", "regulation", "vendor", "cost", "clerk",
            "auth0", "openai",
        ],
    }
}

fn topic_keywords(domain: DomainV1, topic: TopicV1) -> &'static [&'static str] {
    match (domain, topic) {
        (_, TopicV1::General) => &[],
        (DomainV1::Project, TopicV1::Memory) => &["memory", "memo", "recall"],
        (DomainV1::Project, TopicV1::Retrieval) => &["retrieval", "search", "lexical", "hybrid"],
        (DomainV1::Project, TopicV1::Agent) => &["agent", "working memory", "follow-up"],
        (DomainV1::Project, TopicV1::Truth) => &["truth", "t1", "t2", "t3"],
        (DomainV1::Project, TopicV1::Config) => &["config", "toml", "setting"],
        (DomainV1::Project, TopicV1::Testing) => &["test", "assert", "verification"],
        (DomainV1::Project, TopicV1::Docs) => &["doc", "docs", "architecture", "spec"],
        (DomainV1::System, TopicV1::Architecture) => &["architecture", "module", "boundary"],
        (DomainV1::System, TopicV1::Storage) => &["storage", "sqlite", "persist", "record"],
        (DomainV1::System, TopicV1::Runtime) => &["runtime", "gate", "doctor", "status"],
        (DomainV1::System, TopicV1::Model) => &["model", "llm", "summary", "embedding"],
        (DomainV1::System, TopicV1::Security) => &["security", "secret", "auth", "credential"],
        (DomainV1::System, TopicV1::Performance) => &["latency", "performance", "throughput"],
        (DomainV1::System, TopicV1::Integration) => &["integration", "bridge", "pipeline"],
        (DomainV1::Process, TopicV1::Planning) => &["plan", "planning", "phase"],
        (DomainV1::Process, TopicV1::Implementation) => &["implement", "coding", "patch"],
        (DomainV1::Process, TopicV1::Verification) => &["verify", "verification", "test", "clippy"],
        (DomainV1::Process, TopicV1::Review) => &["review", "audit", "inspection"],
        (DomainV1::Process, TopicV1::Experiment) => &["experiment", "hypothesis", "iteration"],
        (DomainV1::External, TopicV1::Provider) => &["provider", "clerk", "auth0", "vendor"],
        (DomainV1::External, TopicV1::Dependency) => &["dependency", "crate", "library"],
        (DomainV1::External, TopicV1::Api) => &["api", "endpoint", "provider api"],
        (DomainV1::External, TopicV1::Regulation) => &["regulation", "policy", "law"],
        (DomainV1::External, TopicV1::Cost) => &["cost", "pricing", "budget"],
        _ => &[],
    }
}

fn aspect_keywords(aspect: AspectV1) -> &'static [&'static str] {
    match aspect {
        AspectV1::General => &[],
        AspectV1::Provider => &["provider", "vendor", "source"],
        AspectV1::Policy => &["policy", "rule", "governance"],
        AspectV1::Capability => &["can", "cannot", "ability", "capability"],
        AspectV1::Structure => &["structure", "layout", "module", "boundary"],
        AspectV1::Interface => &["interface", "api", "input", "output"],
        AspectV1::Behavior => &["behavior", "works", "flow", "use", "uses", "baseline"],
        AspectV1::State => &["state", "status", "mode", "ready", "gated"],
        AspectV1::Timeline => &["timeline", "phase", "date", "2026", "before", "after"],
        AspectV1::Evidence => &["evidence", "source", "citation", "proof"],
        AspectV1::Cost => &["cost", "pricing", "budget"],
        AspectV1::Risk => &["risk", "danger", "failure", "drift"],
        AspectV1::Constraint => &["constraint", "must", "cannot", "limit"],
    }
}

fn kind_keywords(kind: KindV1) -> &'static [&'static str] {
    match kind {
        KindV1::Observation => &[],
        KindV1::Fact => &["is", "are", "remains", "supports"],
        KindV1::Decision => &["decide", "decision", "choose", "use"],
        KindV1::Constraint => &["must", "cannot", "constraint", "required"],
        KindV1::Risk => &["risk", "failure", "drift", "break"],
        KindV1::Hypothesis => &["hypothesis", "assume", "might", "could"],
        KindV1::Pattern => &["pattern", "usually", "reuse", "repeat"],
        KindV1::Issue => &["issue", "bug", "problem", "blocked"],
        KindV1::Rule => &["rule", "policy", "always", "never"],
    }
}

#[derive(Debug, Error)]
pub enum ClassificationError {
    #[error("missing classification source reference")]
    MissingSourceRef,
    #[error("missing raw text for classification")]
    MissingRawText,
    #[error("invalid classification confidence: {0}")]
    InvalidConfidence(f32),
    #[error(transparent)]
    Summary(#[from] crate::memory::summary::FactSummaryError),
    #[error(transparent)]
    Taxonomy(#[from] crate::memory::taxonomy::TaxonomyError),
    #[error("{0}")]
    Classifier(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classification_input_rejects_empty_text() {
        let input = ClassificationInput::new("note#1", "");
        let err = input.validate().expect_err("empty text should fail");

        assert!(matches!(err, ClassificationError::MissingRawText));
    }

    #[test]
    fn classification_output_rejects_invalid_confidence() {
        let output = ClassificationOutput::new(
            TaxonomyPathV1::new(
                DomainV1::Project,
                TopicV1::Retrieval,
                AspectV1::Behavior,
                KindV1::Fact,
            )
            .expect("taxonomy path should be valid"),
            1.4,
            false,
        );

        assert!(matches!(output, Err(ClassificationError::InvalidConfidence(_))));
    }

    #[test]
    fn classification_fallback_uses_general_observation_shape() {
        let fallback = ClassificationOutput::fallback_observation(DomainV1::External);

        assert_eq!(fallback.taxonomy.domain, DomainV1::External);
        assert_eq!(fallback.taxonomy.topic, TopicV1::General);
        assert_eq!(fallback.taxonomy.aspect, AspectV1::General);
        assert_eq!(fallback.taxonomy.kind, KindV1::Observation);
        assert!(fallback.needs_review);
    }

    #[test]
    fn classification_output_builds_summary_input() {
        let output = ClassificationOutput::new(
            TaxonomyPathV1::new(
                DomainV1::Project,
                TopicV1::Retrieval,
                AspectV1::Behavior,
                KindV1::Decision,
            )
            .expect("taxonomy path should be valid"),
            0.88,
            false,
        )
        .expect("classification output should validate");

        let summary_input = output
            .into_summary_input(
                TruthLayer::T2,
                "roadmap#phase9",
                "Use lexical-first as baseline.",
            )
            .expect("classification output should build summary input");

        assert_eq!(summary_input.taxonomy.kind, KindV1::Decision);
        assert_eq!(summary_input.truth_layer, TruthLayer::T2);
    }

    #[tokio::test]
    async fn keyword_classifier_detects_project_retrieval_decision_shape() {
        let classifier = KeywordTaxonomyClassifier::default();
        let output = classifier
            .classify(&ClassificationInput::new(
                "roadmap#phase9",
                "Use lexical-first retrieval as the baseline decision because explainability matters.",
            ))
            .await
            .expect("keyword classifier should classify");

        assert_eq!(output.taxonomy.domain, DomainV1::Project);
        assert_eq!(output.taxonomy.topic, TopicV1::Retrieval);
        assert_eq!(output.taxonomy.aspect, AspectV1::Behavior);
        assert_eq!(output.taxonomy.kind, KindV1::Decision);
    }

    #[tokio::test]
    async fn keyword_classifier_report_exposes_matched_keywords() {
        let classifier = KeywordTaxonomyClassifier::default();
        let (output, report) = classifier
            .classify_with_report(&ClassificationInput::new(
                "roadmap#phase9",
                "Use lexical-first retrieval as the baseline decision because explainability matters.",
            ))
            .await
            .expect("keyword classifier should produce a report");

        assert_eq!(output.taxonomy.topic, TopicV1::Retrieval);
        assert!(report.domain_hits.contains(&"roadmap".to_string()));
        assert!(report.topic_hits.contains(&"retrieval".to_string()));
        assert!(report.aspect_hits.contains(&"use".to_string()));
        assert!(report.kind_hits.contains(&"decision".to_string()));
    }

    #[tokio::test]
    async fn keyword_classifier_threshold_controls_needs_review() {
        let classifier = KeywordTaxonomyClassifier::default().with_review_threshold(1.1);
        let output = classifier
            .classify(&ClassificationInput::new(
                "roadmap#phase9",
                "Use lexical-first retrieval as the baseline decision because explainability matters.",
            ))
            .await
            .expect("keyword classifier should classify");

        assert!(output.needs_review, "high threshold should force review");
    }

    #[test]
    fn classification_input_can_be_built_from_memory_record() {
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
            content_text: "Use lexical-first as baseline.".to_string(),
            chunk: None,
            validity: crate::memory::record::ValidityWindow::default(),
        };

        let input = ClassificationInput::from_record(&record);
        assert_eq!(input.source_ref, "memo://project/retrieval");
        assert_eq!(input.raw_text, "Use lexical-first as baseline.");
        assert_eq!(input.source_kind.as_deref(), Some("note"));
        assert_eq!(input.scope.as_deref(), Some("project"));
        assert_eq!(input.record_type.as_deref(), Some("observation"));
    }

    #[tokio::test]
    async fn keyword_classifier_can_classify_memory_records_directly() {
        let classifier = KeywordTaxonomyClassifier::default();
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
            content_text: "Use lexical-first retrieval as the baseline decision because explainability matters."
                .to_string(),
            chunk: None,
            validity: crate::memory::record::ValidityWindow::default(),
        };

        let (output, report) = classifier
            .classify_record_with_report(&record)
            .await
            .expect("record classification should succeed");

        assert_eq!(output.taxonomy.topic, TopicV1::Retrieval);
        assert!(report.topic_hits.contains(&"retrieval".to_string()));
    }
}
