use serde::Serialize;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::memory::{
    record::{RecordType, Scope, TruthLayer},
    taxonomy::{AspectV1, DomainV1, KindV1, TopicV1},
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct SearchFilters {
    pub scope: Option<Scope>,
    pub record_type: Option<RecordType>,
    pub truth_layer: Option<TruthLayer>,
    pub domain: Option<String>,
    pub topic: Option<String>,
    pub aspect: Option<String>,
    pub kind: Option<String>,
    pub valid_at: Option<String>,
    pub recorded_from: Option<String>,
    pub recorded_to: Option<String>,
}

pub type AppliedFilters = SearchFilters;

impl SearchFilters {
    pub fn scope_value(&self) -> Option<&'static str> {
        self.scope.map(Scope::as_str)
    }

    pub fn record_type_value(&self) -> Option<&'static str> {
        self.record_type.map(RecordType::as_str)
    }

    pub fn truth_layer_value(&self) -> Option<&'static str> {
        self.truth_layer.map(TruthLayer::as_str)
    }

    pub fn validate_taxonomy(&self) -> Result<(), String> {
        let domain = self
            .domain
            .as_deref()
            .map(|value| {
                DomainV1::parse(value)
                    .ok_or_else(|| format!("unsupported taxonomy domain: {value}"))
            })
            .transpose()?;
        let topic = self
            .topic
            .as_deref()
            .map(|value| {
                TopicV1::parse(value).ok_or_else(|| format!("unsupported taxonomy topic: {value}"))
            })
            .transpose()?;
        self.aspect
            .as_deref()
            .map(|value| {
                AspectV1::parse(value)
                    .ok_or_else(|| format!("unsupported taxonomy aspect: {value}"))
            })
            .transpose()?;
        self.kind
            .as_deref()
            .map(|value| {
                KindV1::parse(value).ok_or_else(|| format!("unsupported taxonomy kind: {value}"))
            })
            .transpose()?;

        if let (Some(domain), Some(topic)) = (domain, topic)
            && !TopicV1::allowed_for(domain).contains(&topic)
        {
            return Err(format!(
                "unsupported taxonomy combination: domain={} does not allow topic={}",
                domain.as_str(),
                topic.as_str()
            ));
        }

        self.valid_at
            .as_deref()
            .map(|value| parse_rfc3339_filter("valid_at", value))
            .transpose()?;
        let recorded_from = self
            .recorded_from
            .as_deref()
            .map(|value| parse_rfc3339_filter("from", value))
            .transpose()?;
        let recorded_to = self
            .recorded_to
            .as_deref()
            .map(|value| parse_rfc3339_filter("to", value))
            .transpose()?;

        if let (Some(recorded_from), Some(recorded_to)) = (recorded_from, recorded_to)
            && recorded_from > recorded_to
        {
            return Err(format!(
                "invalid temporal range: from={} is later than to={}",
                self.recorded_from.as_deref().unwrap_or(""),
                self.recorded_to.as_deref().unwrap_or("")
            ));
        }

        Ok(())
    }
}

fn parse_rfc3339_filter(field: &str, value: &str) -> Result<OffsetDateTime, String> {
    OffsetDateTime::parse(value, &Rfc3339)
        .map_err(|_| format!("invalid RFC3339 value for {field}: {value}"))
}
