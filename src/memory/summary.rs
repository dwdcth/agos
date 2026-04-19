use std::{future::Future, pin::Pin};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::memory::{
    dsl::{FactDslDraft, FactDslError, FactDslRecord, KindFieldPolicyV1},
    record::TruthLayer,
    taxonomy::{KindV1, TaxonomyPathV1},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactSummaryInput {
    pub taxonomy: TaxonomyPathV1,
    pub truth_layer: TruthLayer,
    pub source_ref: String,
    pub raw_text: String,
}

impl FactSummaryInput {
    pub fn new(
        taxonomy: TaxonomyPathV1,
        truth_layer: TruthLayer,
        source_ref: impl Into<String>,
        raw_text: impl Into<String>,
    ) -> Self {
        Self {
            taxonomy,
            truth_layer,
            source_ref: source_ref.into(),
            raw_text: raw_text.into(),
        }
    }

    pub fn validate(&self) -> Result<(), FactSummaryError> {
        self.taxonomy.validate().map_err(FactSummaryError::Taxonomy)?;

        if self.source_ref.trim().is_empty() {
            return Err(FactSummaryError::MissingSourceRef);
        }
        if self.raw_text.trim().is_empty() {
            return Err(FactSummaryError::MissingRawText);
        }

        Ok(())
    }

    pub fn kind_policy(&self) -> KindFieldPolicyV1 {
        KindFieldPolicyV1::for_kind(self.taxonomy.kind)
    }

    pub fn into_record(self, draft: FactDslDraft) -> Result<FactDslRecord, FactSummaryError> {
        self.validate()?;
        validate_summary_output(self.taxonomy.kind, &draft)?;

        let record = FactDslRecord {
            taxonomy: self.taxonomy,
            draft,
            truth_layer: self.truth_layer,
            source_ref: self.source_ref,
        };
        record.validate().map_err(FactSummaryError::Dsl)?;
        Ok(record)
    }
}

pub trait FactSummaryGenerator {
    fn summarize<'a>(
        &'a self,
        input: &'a FactSummaryInput,
    ) -> Pin<Box<dyn Future<Output = Result<FactDslDraft, FactSummaryError>> + Send + 'a>>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RuleBasedSummaryGenerator;

impl FactSummaryGenerator for RuleBasedSummaryGenerator {
    fn summarize<'a>(
        &'a self,
        input: &'a FactSummaryInput,
    ) -> Pin<Box<dyn Future<Output = Result<FactDslDraft, FactSummaryError>> + Send + 'a>>
    {
        Box::pin(async move {
            input.validate()?;

            let claim = first_sentence(&input.raw_text)
                .unwrap_or_else(|| input.raw_text.trim().to_string());

            let draft = FactDslDraft {
                claim,
                why: extract_reason(&input.raw_text),
                time: extract_time_hint(&input.raw_text),
                cond: extract_condition(&input.raw_text),
                impact: extract_impact(&input.raw_text),
                conf: matches!(input.taxonomy.kind, KindV1::Hypothesis).then_some(0.5),
                rel: None,
            };

            validate_summary_output(input.taxonomy.kind, &draft)?;
            Ok(draft)
        })
    }
}

#[derive(Debug, Error)]
pub enum FactSummaryError {
    #[error("missing summary source reference")]
    MissingSourceRef,
    #[error("missing raw text for summarization")]
    MissingRawText,
    #[error("generated draft is missing claim")]
    MissingClaim,
    #[error("generated draft is missing confidence for hypothesis")]
    MissingHypothesisConfidence,
    #[error(transparent)]
    Dsl(#[from] FactDslError),
    #[error(transparent)]
    Taxonomy(#[from] crate::memory::taxonomy::TaxonomyError),
    #[error("{0}")]
    Generator(String),
}

pub fn validate_summary_output(kind: KindV1, draft: &FactDslDraft) -> Result<(), FactSummaryError> {
    if draft.claim.trim().is_empty() {
        return Err(FactSummaryError::MissingClaim);
    }

    if matches!(kind, KindV1::Hypothesis) && draft.conf.is_none() {
        return Err(FactSummaryError::MissingHypothesisConfidence);
    }

    Ok(())
}

fn first_sentence(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    let end = trimmed
        .char_indices()
        .find(|(_, ch)| matches!(ch, '.' | '!' | '?' | '。' | '！' | '？'))
        .map(|(index, ch)| index + ch.len_utf8())
        .unwrap_or(trimmed.len());

    Some(trimmed[..end].trim().to_string())
}

fn extract_reason(text: &str) -> Option<String> {
    let lowered = text.to_lowercase();
    if let Some(index) = lowered.find("because ") {
        return Some(text[index + "because ".len()..].trim().to_string());
    }
    if let Some(index) = text.find("因为") {
        return Some(text[index + "因为".len()..].trim().to_string());
    }
    None
}

fn extract_time_hint(text: &str) -> Option<String> {
    for token in text.split(|ch: char| ch.is_whitespace() || matches!(ch, ',' | '，' | ';' | '；')) {
        let token = token.trim_matches(|ch: char| matches!(ch, '.' | '!' | '?' | '。' | '！' | '？'));
        if looks_like_time_hint(token) {
            return Some(token.to_string());
        }
    }
    None
}

fn extract_condition(text: &str) -> Option<String> {
    let lowered = text.to_lowercase();
    if let Some(index) = lowered.find("if ") {
        return Some(text[index + "if ".len()..].trim().to_string());
    }
    if let Some(index) = text.find("如果") {
        return Some(text[index + "如果".len()..].trim().to_string());
    }
    None
}

fn extract_impact(text: &str) -> Option<String> {
    let lowered = text.to_lowercase();
    if let Some(index) = lowered.find("so that ") {
        return Some(text[index + "so that ".len()..].trim().to_string());
    }
    if let Some(index) = lowered.find("so ") {
        return Some(text[index + "so ".len()..].trim().to_string());
    }
    if let Some(index) = text.find("因此") {
        return Some(text[index + "因此".len()..].trim().to_string());
    }
    None
}

fn looks_like_time_hint(token: &str) -> bool {
    let bytes = token.as_bytes();
    if bytes.len() >= 4
        && bytes[0..4].iter().all(u8::is_ascii_digit)
        && (bytes.len() == 4
            || token.contains('-')
            || token.contains('/')
            || token.contains("年"))
    {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::taxonomy::{AspectV1, DomainV1, TopicV1};

    fn sample_input(kind: KindV1) -> FactSummaryInput {
        FactSummaryInput::new(
            TaxonomyPathV1::new(DomainV1::Project, TopicV1::Retrieval, AspectV1::Behavior, kind)
                .expect("sample taxonomy path should construct"),
            TruthLayer::T2,
            "roadmap#phase9",
            "Use lexical-first as baseline because explainability matters.",
        )
    }

    #[test]
    fn summary_input_exposes_kind_policy() {
        let input = sample_input(KindV1::Decision);
        let policy = input.kind_policy();

        assert_eq!(policy.recommended.len(), 3);
        assert_eq!(policy.recommended[0].as_str(), "WHY");
    }

    #[test]
    fn summary_input_rejects_empty_raw_text() {
        let mut input = sample_input(KindV1::Fact);
        input.raw_text.clear();

        let err = input.validate().expect_err("empty raw text should fail");
        assert!(matches!(err, FactSummaryError::MissingRawText));
    }

    #[test]
    fn hypothesis_output_requires_confidence() {
        let err = validate_summary_output(
            KindV1::Hypothesis,
            &FactDslDraft {
                claim: "tfidf is enough".to_string(),
                ..Default::default()
            },
        )
        .expect_err("hypothesis outputs should include confidence");

        assert!(matches!(err, FactSummaryError::MissingHypothesisConfidence));
    }

    #[test]
    fn summary_input_builds_dsl_record_from_valid_draft() {
        let input = sample_input(KindV1::Decision);
        let record = input
            .into_record(FactDslDraft {
                claim: "use lexical-first as baseline".to_string(),
                why: Some("explainability matters".to_string()),
                time: Some("2026-04".to_string()),
                ..Default::default()
            })
            .expect("valid draft should build DSL record");

        assert_eq!(record.taxonomy.kind, KindV1::Decision);
        assert_eq!(record.truth_layer, TruthLayer::T2);
        assert_eq!(record.source_ref, "roadmap#phase9");
    }

    #[tokio::test]
    async fn rule_based_summary_generator_extracts_reason_and_time() {
        let input = FactSummaryInput::new(
            TaxonomyPathV1::new(
                DomainV1::Project,
                TopicV1::Retrieval,
                AspectV1::Behavior,
                KindV1::Decision,
            )
            .expect("taxonomy path should construct"),
            TruthLayer::T2,
            "roadmap#phase9",
            "2026-04 use lexical-first as baseline because explainability matters.",
        );

        let draft = RuleBasedSummaryGenerator
            .summarize(&input)
            .await
            .expect("rule-based summary should succeed");

        assert_eq!(draft.claim, "2026-04 use lexical-first as baseline because explainability matters.");
        assert_eq!(draft.why.as_deref(), Some("explainability matters."));
        assert_eq!(draft.time.as_deref(), Some("2026-04"));
    }

    #[tokio::test]
    async fn rule_based_summary_generator_extracts_condition_and_impact() {
        let input = FactSummaryInput::new(
            TaxonomyPathV1::new(
                DomainV1::System,
                TopicV1::Runtime,
                AspectV1::Risk,
                KindV1::Risk,
            )
            .expect("taxonomy path should construct"),
            TruthLayer::T2,
            "notes#risk",
            "If embedding replaces lexical baseline, recall may drift, so debugging becomes harder.",
        );

        let draft = RuleBasedSummaryGenerator
            .summarize(&input)
            .await
            .expect("rule-based summary should succeed");

        assert_eq!(
            draft.cond.as_deref(),
            Some("embedding replaces lexical baseline, recall may drift, so debugging becomes harder.")
        );
        assert_eq!(draft.impact.as_deref(), Some("debugging becomes harder."));
    }
}
