use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

use crate::memory::{
    record::TruthLayer,
    taxonomy::{AspectV1, DomainV1, KindV1, TaxonomyPathV1, TopicV1},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DslFieldV1 {
    Dom,
    Top,
    Asp,
    Kind,
    Claim,
    Truth,
    Src,
    Why,
    Time,
    Cond,
    Impact,
    Conf,
    Rel,
}

impl DslFieldV1 {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dom => "DOM",
            Self::Top => "TOP",
            Self::Asp => "ASP",
            Self::Kind => "KIND",
            Self::Claim => "CLAIM",
            Self::Truth => "TRUTH",
            Self::Src => "SRC",
            Self::Why => "WHY",
            Self::Time => "TIME",
            Self::Cond => "COND",
            Self::Impact => "IMPACT",
            Self::Conf => "CONF",
            Self::Rel => "REL",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FactDslDraft {
    pub claim: String,
    pub why: Option<String>,
    pub time: Option<String>,
    pub cond: Option<String>,
    pub impact: Option<String>,
    pub conf: Option<f32>,
    pub rel: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FactDslRecord {
    pub taxonomy: TaxonomyPathV1,
    pub draft: FactDslDraft,
    pub truth_layer: TruthLayer,
    pub source_ref: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlatFactDslRecordV1 {
    pub domain: String,
    pub topic: String,
    pub aspect: String,
    pub kind: String,
    pub claim: String,
    pub truth_layer: String,
    pub source_ref: String,
    pub why: Option<String>,
    pub time: Option<String>,
    pub cond: Option<String>,
    pub impact: Option<String>,
    pub conf: Option<f32>,
    pub rel: Option<Vec<String>>,
}

impl FactDslRecord {
    pub fn validate(&self) -> Result<(), FactDslError> {
        self.taxonomy.validate().map_err(FactDslError::Taxonomy)?;

        if self.draft.claim.trim().is_empty() {
            return Err(FactDslError::MissingClaim);
        }
        if self.source_ref.trim().is_empty() {
            return Err(FactDslError::MissingSourceRef);
        }

        if let Some(conf) = self.draft.conf {
            if !(0.0..=1.0).contains(&conf) {
                return Err(FactDslError::InvalidConfidence(conf));
            }
        }

        if let Some(rel) = &self.draft.rel {
            if rel.iter().any(|value| value.trim().is_empty()) {
                return Err(FactDslError::EmptyRelation);
            }
        }

        Ok(())
    }

    pub fn encode(&self) -> Result<String, FactDslError> {
        self.validate()?;

        let mut fields = vec![
            format!("DOM={}", self.taxonomy.domain.as_str()),
            format!("TOP={}", self.taxonomy.topic.as_str()),
            format!("ASP={}", self.taxonomy.aspect.as_str()),
            format!("KIND={}", self.taxonomy.kind.as_str()),
            format!("CLAIM={}", escape_field_value(self.draft.claim.trim())),
            format!("TRUTH={}", self.truth_layer.as_str().to_uppercase()),
            format!("SRC={}", escape_field_value(self.source_ref.trim())),
        ];

        if let Some(value) = optional_value(&self.draft.why) {
            fields.push(format!("WHY={}", escape_field_value(value)));
        }
        if let Some(value) = optional_value(&self.draft.time) {
            fields.push(format!("TIME={}", escape_field_value(value)));
        }
        if let Some(value) = optional_value(&self.draft.cond) {
            fields.push(format!("COND={}", escape_field_value(value)));
        }
        if let Some(value) = optional_value(&self.draft.impact) {
            fields.push(format!("IMPACT={}", escape_field_value(value)));
        }
        if let Some(value) = self.draft.conf {
            fields.push(format!("CONF={value:.2}"));
        }
        if let Some(rel) = &self.draft.rel {
            let rel = rel
                .iter()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(escape_field_value)
                .collect::<Vec<_>>();
            if !rel.is_empty() {
                fields.push(format!("REL={}", rel.join(",")));
            }
        }

        Ok(format!("F|{}", fields.join("|")))
    }

    pub fn assess_kind_fields(&self) -> KindFieldAssessmentV1 {
        KindFieldAssessmentV1::assess(self.taxonomy.kind, &self.draft)
    }

    pub fn flatten(&self) -> FlatFactDslRecordV1 {
        FlatFactDslRecordV1 {
            domain: self.taxonomy.domain.to_string(),
            topic: self.taxonomy.topic.to_string(),
            aspect: self.taxonomy.aspect.to_string(),
            kind: self.taxonomy.kind.to_string(),
            claim: self.draft.claim.clone(),
            truth_layer: self.truth_layer.as_str().to_string(),
            source_ref: self.source_ref.clone(),
            why: self.draft.why.clone(),
            time: self.draft.time.clone(),
            cond: self.draft.cond.clone(),
            impact: self.draft.impact.clone(),
            conf: self.draft.conf,
            rel: self.draft.rel.clone(),
        }
    }

    pub fn parse(input: &str) -> Result<Self, FactDslError> {
        let mut parts = split_escaped(input, '|').into_iter();
        let Some(prefix) = parts.next() else {
            return Err(FactDslError::MissingPrefix);
        };
        if prefix != "F" {
            return Err(FactDslError::InvalidPrefix(prefix.to_string()));
        }

        let fields = parts.collect::<Vec<_>>();
        let mut index = 0usize;

        let domain = parse_required(&fields, &mut index, DslFieldV1::Dom, DomainV1::parse)?;
        let topic = parse_required(&fields, &mut index, DslFieldV1::Top, TopicV1::parse)?;
        let aspect = parse_required(&fields, &mut index, DslFieldV1::Asp, AspectV1::parse)?;
        let kind = parse_required(&fields, &mut index, DslFieldV1::Kind, KindV1::parse)?;
        let claim = parse_required_string(&fields, &mut index, DslFieldV1::Claim)?;
        let truth_layer = parse_required(
            &fields,
            &mut index,
            DslFieldV1::Truth,
            parse_truth_layer_upper,
        )?;
        let source_ref = parse_required_string(&fields, &mut index, DslFieldV1::Src)?;

        let mut draft = FactDslDraft {
            claim,
            ..Default::default()
        };

        while index < fields.len() {
            let (key, value) = split_field(&fields[index])?;
            match key {
                "WHY" => {
                    draft.why = Some(unescape_field_value(non_empty_value(
                        value,
                        DslFieldV1::Why,
                    )?))
                }
                "TIME" => {
                    draft.time = Some(unescape_field_value(non_empty_value(
                        value,
                        DslFieldV1::Time,
                    )?))
                }
                "COND" => {
                    draft.cond = Some(unescape_field_value(non_empty_value(
                        value,
                        DslFieldV1::Cond,
                    )?))
                }
                "IMPACT" => {
                    draft.impact = Some(unescape_field_value(non_empty_value(
                        value,
                        DslFieldV1::Impact,
                    )?))
                }
                "CONF" => {
                    let conf =
                        f32::from_str(non_empty_value(value, DslFieldV1::Conf)?).map_err(|_| {
                            FactDslError::InvalidFieldValue {
                                field: DslFieldV1::Conf.as_str(),
                                value: value.to_string(),
                            }
                        })?;
                    draft.conf = Some(conf);
                }
                "REL" => {
                    let rel = non_empty_value(value, DslFieldV1::Rel)?.to_string();
                    let rel = split_escaped(&rel, ',')
                        .into_iter()
                        .map(|value| value.trim().to_string())
                        .filter(|value| !value.is_empty())
                        .map(|value| unescape_field_value(&value))
                        .collect::<Vec<_>>();
                    if rel.is_empty() {
                        return Err(FactDslError::EmptyRelation);
                    }
                    draft.rel = Some(rel);
                }
                other => {
                    return Err(FactDslError::UnknownField(other.to_string()));
                }
            }
            index += 1;
        }

        let record = Self {
            taxonomy: TaxonomyPathV1::new(domain, topic, aspect, kind)?,
            draft,
            truth_layer,
            source_ref,
        };
        record.validate()?;
        Ok(record)
    }
}

impl FlatFactDslRecordV1 {
    pub fn to_json_string(&self) -> Result<String, FactDslError> {
        serde_json::to_string(self).map_err(FactDslError::Json)
    }

    pub fn from_json_str(value: &str) -> Result<Self, FactDslError> {
        serde_json::from_str(value).map_err(FactDslError::Json)
    }

    pub fn into_record(self) -> Result<FactDslRecord, FactDslError> {
        let taxonomy =
            TaxonomyPathV1::from_parts(&self.domain, &self.topic, &self.aspect, &self.kind)
                .map_err(FactDslError::Taxonomy)?;
        let truth_layer = TruthLayer::parse(&self.truth_layer).ok_or_else(|| {
            FactDslError::InvalidFieldValue {
                field: DslFieldV1::Truth.as_str(),
                value: self.truth_layer.clone(),
            }
        })?;

        let record = FactDslRecord {
            taxonomy,
            draft: FactDslDraft {
                claim: self.claim,
                why: self.why,
                time: self.time,
                cond: self.cond,
                impact: self.impact,
                conf: self.conf,
                rel: self.rel,
            },
            truth_layer,
            source_ref: self.source_ref,
        };
        record.validate()?;
        Ok(record)
    }
}

fn optional_value(value: &Option<String>) -> Option<&str> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn split_field(field: &str) -> Result<(&str, &str), FactDslError> {
    let parts = split_escaped_once(field, '=')
        .ok_or_else(|| FactDslError::MalformedField(field.to_string()))?;
    Ok(parts)
}

fn non_empty_value(value: &str, field: DslFieldV1) -> Result<&str, FactDslError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(FactDslError::EmptyFieldValue(field.as_str()));
    }
    Ok(value)
}

fn parse_required<T>(
    fields: &[String],
    index: &mut usize,
    expected: DslFieldV1,
    parse: impl FnOnce(&str) -> Option<T>,
) -> Result<T, FactDslError> {
    let value = parse_required_string(fields, index, expected)?;
    parse(&value).ok_or_else(|| FactDslError::InvalidFieldValue {
        field: expected.as_str(),
        value,
    })
}

fn parse_required_string(
    fields: &[String],
    index: &mut usize,
    expected: DslFieldV1,
) -> Result<String, FactDslError> {
    let Some(field) = fields.get(*index) else {
        return Err(FactDslError::MissingField(expected.as_str()));
    };
    let (key, value) = split_field(field)?;
    if key != expected.as_str() {
        return Err(FactDslError::UnexpectedFieldOrder {
            expected: expected.as_str(),
            found: key.to_string(),
        });
    }
    *index += 1;
    Ok(unescape_field_value(non_empty_value(value, expected)?))
}

fn parse_truth_layer_upper(value: &str) -> Option<TruthLayer> {
    match value {
        "T1" => Some(TruthLayer::T1),
        "T2" => Some(TruthLayer::T2),
        "T3" => Some(TruthLayer::T3),
        _ => None,
    }
}

fn escape_field_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '|' => escaped.push_str("\\|"),
            '=' => escaped.push_str("\\="),
            ',' => escaped.push_str("\\,"),
            other => escaped.push(other),
        }
    }
    escaped
}

fn unescape_field_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut escaped = false;
    for ch in value.chars() {
        if escaped {
            out.push(ch);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else {
            out.push(ch);
        }
    }
    if escaped {
        out.push('\\');
    }
    out
}

fn split_escaped(input: &str, separator: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut escaped = false;

    for ch in input.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }

        if ch == '\\' {
            current.push(ch);
            escaped = true;
            continue;
        }

        if ch == separator {
            parts.push(current);
            current = String::new();
        } else {
            current.push(ch);
        }
    }

    parts.push(current);
    parts
}

fn split_escaped_once(input: &str, separator: char) -> Option<(&str, &str)> {
    let mut escaped = false;
    for (index, ch) in input.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == separator {
            return Some((&input[..index], &input[index + ch.len_utf8()..]));
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KindFieldPolicyV1 {
    pub recommended: &'static [DslFieldV1],
    pub discouraged: &'static [DslFieldV1],
}

impl KindFieldPolicyV1 {
    pub fn for_kind(kind: KindV1) -> Self {
        use DslFieldV1::{Cond, Conf, Impact, Rel, Time, Why};

        match kind {
            KindV1::Observation => Self {
                recommended: &[Time],
                discouraged: &[Why, Impact, Conf, Rel],
            },
            KindV1::Fact => Self {
                recommended: &[Time, Cond],
                discouraged: &[Why, Impact, Conf, Rel],
            },
            KindV1::Decision => Self {
                recommended: &[Why, Time, Impact],
                discouraged: &[Conf, Rel],
            },
            KindV1::Constraint => Self {
                recommended: &[Cond, Time],
                discouraged: &[Why, Impact, Conf, Rel],
            },
            KindV1::Risk => Self {
                recommended: &[Impact, Cond, Time],
                discouraged: &[Why, Conf, Rel],
            },
            KindV1::Hypothesis => Self {
                recommended: &[Cond, Conf, Time],
                discouraged: &[Impact, Rel],
            },
            KindV1::Pattern => Self {
                recommended: &[Why, Cond, Rel],
                discouraged: &[Conf],
            },
            KindV1::Issue => Self {
                recommended: &[Impact, Time, Cond],
                discouraged: &[Why, Conf, Rel],
            },
            KindV1::Rule => Self {
                recommended: &[Why, Cond, Time],
                discouraged: &[Impact, Conf, Rel],
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KindFieldAssessmentV1 {
    pub missing_recommended: Vec<DslFieldV1>,
    pub present_discouraged: Vec<DslFieldV1>,
}

impl KindFieldAssessmentV1 {
    pub fn assess(kind: KindV1, draft: &FactDslDraft) -> Self {
        let policy = KindFieldPolicyV1::for_kind(kind);

        let missing_recommended = policy
            .recommended
            .iter()
            .copied()
            .filter(|field| !draft_has_field(draft, *field))
            .collect::<Vec<_>>();

        let present_discouraged = policy
            .discouraged
            .iter()
            .copied()
            .filter(|field| draft_has_field(draft, *field))
            .collect::<Vec<_>>();

        Self {
            missing_recommended,
            present_discouraged,
        }
    }
}

fn draft_has_field(draft: &FactDslDraft, field: DslFieldV1) -> bool {
    match field {
        DslFieldV1::Dom
        | DslFieldV1::Top
        | DslFieldV1::Asp
        | DslFieldV1::Kind
        | DslFieldV1::Claim
        | DslFieldV1::Truth
        | DslFieldV1::Src => true,
        DslFieldV1::Why => optional_value(&draft.why).is_some(),
        DslFieldV1::Time => optional_value(&draft.time).is_some(),
        DslFieldV1::Cond => optional_value(&draft.cond).is_some(),
        DslFieldV1::Impact => optional_value(&draft.impact).is_some(),
        DslFieldV1::Conf => draft.conf.is_some(),
        DslFieldV1::Rel => draft.rel.as_ref().is_some_and(|rel| !rel.is_empty()),
    }
}

#[derive(Debug, Error)]
pub enum FactDslError {
    #[error("missing DSL prefix")]
    MissingPrefix,
    #[error("invalid DSL prefix: {0}")]
    InvalidPrefix(String),
    #[error("malformed DSL field: {0}")]
    MalformedField(String),
    #[error("missing required DSL field: {0}")]
    MissingField(&'static str),
    #[error("unexpected DSL field order, expected '{expected}', found '{found}'")]
    UnexpectedFieldOrder {
        expected: &'static str,
        found: String,
    },
    #[error("unknown DSL field: {0}")]
    UnknownField(String),
    #[error("empty value for DSL field: {0}")]
    EmptyFieldValue(&'static str),
    #[error("invalid value for DSL field '{field}': {value}")]
    InvalidFieldValue { field: &'static str, value: String },
    #[error("missing DSL claim")]
    MissingClaim,
    #[error("missing DSL source reference")]
    MissingSourceRef,
    #[error("invalid DSL confidence: {0}")]
    InvalidConfidence(f32),
    #[error("DSL relations must not contain empty values")]
    EmptyRelation,
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Taxonomy(#[from] crate::memory::taxonomy::TaxonomyError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{
        record::TruthLayer,
        taxonomy::{AspectV1, DomainV1, KindV1, TaxonomyPathV1, TopicV1},
    };

    fn sample_record(kind: KindV1) -> FactDslRecord {
        FactDslRecord {
            taxonomy: TaxonomyPathV1::new(
                DomainV1::Project,
                TopicV1::Retrieval,
                AspectV1::Behavior,
                kind,
            )
            .expect("sample taxonomy path should construct"),
            draft: FactDslDraft {
                claim: "use lexical-first as baseline".to_string(),
                why: Some("explainability and stability".to_string()),
                time: Some("2026-04".to_string()),
                cond: None,
                impact: None,
                conf: None,
                rel: None,
            },
            truth_layer: TruthLayer::T2,
            source_ref: "roadmap#phase9".to_string(),
        }
    }

    #[test]
    fn encoder_renders_required_and_optional_fields_in_fixed_order() {
        let rendered = sample_record(KindV1::Decision)
            .encode()
            .expect("sample record should encode");

        assert_eq!(
            rendered,
            "F|DOM=project|TOP=retrieval|ASP=behavior|KIND=decision|CLAIM=use lexical-first as baseline|TRUTH=T2|SRC=roadmap#phase9|WHY=explainability and stability|TIME=2026-04"
        );
    }

    #[test]
    fn encoder_rejects_empty_claim() {
        let mut record = sample_record(KindV1::Fact);
        record.draft.claim.clear();

        let err = record
            .encode()
            .expect_err("empty claims should fail validation");
        assert!(matches!(err, FactDslError::MissingClaim));
    }

    #[test]
    fn encoder_rejects_out_of_range_confidence() {
        let mut record = sample_record(KindV1::Hypothesis);
        record.draft.conf = Some(1.2);

        let err = record
            .validate()
            .expect_err("confidence should stay within 0..=1");
        assert!(matches!(err, FactDslError::InvalidConfidence(_)));
    }

    #[test]
    fn kind_policy_matches_documented_decision_fields() {
        let policy = KindFieldPolicyV1::for_kind(KindV1::Decision);

        assert_eq!(
            policy.recommended,
            &[DslFieldV1::Why, DslFieldV1::Time, DslFieldV1::Impact]
        );
        assert_eq!(policy.discouraged, &[DslFieldV1::Conf, DslFieldV1::Rel]);
    }

    #[test]
    fn parser_round_trips_encoded_records() {
        let record = sample_record(KindV1::Decision);
        let encoded = record.encode().expect("record should encode");
        let parsed = FactDslRecord::parse(&encoded).expect("encoded record should parse");

        assert_eq!(parsed, record);
    }

    #[test]
    fn parser_rejects_wrong_required_field_order() {
        let err = FactDslRecord::parse(
            "F|TOP=retrieval|DOM=project|ASP=behavior|KIND=decision|CLAIM=x|TRUTH=T2|SRC=a",
        )
        .expect_err("field order should be fixed");

        assert!(matches!(err, FactDslError::UnexpectedFieldOrder { .. }));
    }

    #[test]
    fn encoder_and_parser_escape_reserved_separators() {
        let mut record = sample_record(KindV1::Decision);
        record.draft.claim = "use lexical|first = baseline".to_string();
        record.draft.why = Some("preserve a,b and c".to_string());
        record.source_ref = "roadmap#phase9|line=12".to_string();
        record.draft.rel = Some(vec!["alpha,beta".to_string(), "x|y=z".to_string()]);

        let encoded = record.encode().expect("record should encode");
        assert!(encoded.contains(r"CLAIM=use lexical\|first \= baseline"));
        assert!(encoded.contains(r"WHY=preserve a\,b and c"));
        assert!(encoded.contains(r"SRC=roadmap#phase9\|line\=12"));

        let parsed = FactDslRecord::parse(&encoded).expect("escaped record should parse");
        assert_eq!(parsed, record);
    }

    #[test]
    fn assessment_marks_missing_recommended_and_present_discouraged_fields() {
        let assessment = KindFieldAssessmentV1::assess(
            KindV1::Decision,
            &FactDslDraft {
                claim: "pick lexical-first".to_string(),
                conf: Some(0.4),
                ..Default::default()
            },
        );

        assert_eq!(
            assessment.missing_recommended,
            vec![DslFieldV1::Why, DslFieldV1::Time, DslFieldV1::Impact]
        );
        assert_eq!(assessment.present_discouraged, vec![DslFieldV1::Conf]);
    }

    #[test]
    fn record_assessment_reuses_kind_policy_over_draft() {
        let mut record = sample_record(KindV1::Decision);
        record.draft.why = None;
        record.draft.time = None;

        let assessment = record.assess_kind_fields();
        assert_eq!(
            assessment.missing_recommended,
            vec![DslFieldV1::Why, DslFieldV1::Time, DslFieldV1::Impact]
        );
    }

    #[test]
    fn record_flattens_and_round_trips() {
        let mut record = sample_record(KindV1::Decision);
        record.draft.impact = Some("keeps retrieval debuggable".to_string());
        record.draft.rel = Some(vec!["phase9".to_string()]);

        let flat = record.flatten();
        let rebuilt = flat.into_record().expect("flat record should rebuild");

        assert_eq!(rebuilt, record);
    }

    #[test]
    fn flat_record_json_round_trips() {
        let mut record = sample_record(KindV1::Decision);
        record.draft.impact = Some("keeps retrieval debuggable".to_string());

        let flat = record.flatten();
        let json = flat.to_json_string().expect("flat record should serialize");
        let parsed = FlatFactDslRecordV1::from_json_str(&json).expect("json should parse");

        assert_eq!(parsed, flat);
    }
}
