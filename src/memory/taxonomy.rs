use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DomainV1 {
    Project,
    System,
    Process,
    External,
}

impl DomainV1 {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::System => "system",
            Self::Process => "process",
            Self::External => "external",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "project" => Some(Self::Project),
            "system" => Some(Self::System),
            "process" => Some(Self::Process),
            "external" => Some(Self::External),
            _ => None,
        }
    }
}

impl fmt::Display for DomainV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for DomainV1 {
    type Err = TaxonomyError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value).ok_or_else(|| TaxonomyError::InvalidValue {
            field: "domain",
            value: value.to_string(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopicV1 {
    General,
    Memory,
    Retrieval,
    Agent,
    Truth,
    Config,
    Testing,
    Docs,
    Architecture,
    Storage,
    Runtime,
    Model,
    Security,
    Performance,
    Integration,
    Planning,
    Implementation,
    Verification,
    Review,
    Experiment,
    Provider,
    Dependency,
    Api,
    Regulation,
    Cost,
}

impl TopicV1 {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Memory => "memory",
            Self::Retrieval => "retrieval",
            Self::Agent => "agent",
            Self::Truth => "truth",
            Self::Config => "config",
            Self::Testing => "testing",
            Self::Docs => "docs",
            Self::Architecture => "architecture",
            Self::Storage => "storage",
            Self::Runtime => "runtime",
            Self::Model => "model",
            Self::Security => "security",
            Self::Performance => "performance",
            Self::Integration => "integration",
            Self::Planning => "planning",
            Self::Implementation => "implementation",
            Self::Verification => "verification",
            Self::Review => "review",
            Self::Experiment => "experiment",
            Self::Provider => "provider",
            Self::Dependency => "dependency",
            Self::Api => "api",
            Self::Regulation => "regulation",
            Self::Cost => "cost",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "general" => Some(Self::General),
            "memory" => Some(Self::Memory),
            "retrieval" => Some(Self::Retrieval),
            "agent" => Some(Self::Agent),
            "truth" => Some(Self::Truth),
            "config" => Some(Self::Config),
            "testing" => Some(Self::Testing),
            "docs" => Some(Self::Docs),
            "architecture" => Some(Self::Architecture),
            "storage" => Some(Self::Storage),
            "runtime" => Some(Self::Runtime),
            "model" => Some(Self::Model),
            "security" => Some(Self::Security),
            "performance" => Some(Self::Performance),
            "integration" => Some(Self::Integration),
            "planning" => Some(Self::Planning),
            "implementation" => Some(Self::Implementation),
            "verification" => Some(Self::Verification),
            "review" => Some(Self::Review),
            "experiment" => Some(Self::Experiment),
            "provider" => Some(Self::Provider),
            "dependency" => Some(Self::Dependency),
            "api" => Some(Self::Api),
            "regulation" => Some(Self::Regulation),
            "cost" => Some(Self::Cost),
            _ => None,
        }
    }

    pub fn allowed_for(domain: DomainV1) -> &'static [Self] {
        use TopicV1::*;

        match domain {
            DomainV1::Project => &[General, Memory, Retrieval, Agent, Truth, Config, Testing, Docs],
            DomainV1::System => {
                &[General, Architecture, Storage, Runtime, Model, Security, Performance, Integration]
            }
            DomainV1::Process => &[General, Planning, Implementation, Verification, Review, Experiment],
            DomainV1::External => &[General, Provider, Dependency, Api, Regulation, Cost],
        }
    }
}

impl fmt::Display for TopicV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for TopicV1 {
    type Err = TaxonomyError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value).ok_or_else(|| TaxonomyError::InvalidValue {
            field: "topic",
            value: value.to_string(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AspectV1 {
    General,
    Provider,
    Policy,
    Capability,
    Structure,
    Interface,
    Behavior,
    State,
    Timeline,
    Evidence,
    Cost,
    Risk,
    Constraint,
}

impl AspectV1 {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Provider => "provider",
            Self::Policy => "policy",
            Self::Capability => "capability",
            Self::Structure => "structure",
            Self::Interface => "interface",
            Self::Behavior => "behavior",
            Self::State => "state",
            Self::Timeline => "timeline",
            Self::Evidence => "evidence",
            Self::Cost => "cost",
            Self::Risk => "risk",
            Self::Constraint => "constraint",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "general" => Some(Self::General),
            "provider" => Some(Self::Provider),
            "policy" => Some(Self::Policy),
            "capability" => Some(Self::Capability),
            "structure" => Some(Self::Structure),
            "interface" => Some(Self::Interface),
            "behavior" => Some(Self::Behavior),
            "state" => Some(Self::State),
            "timeline" => Some(Self::Timeline),
            "evidence" => Some(Self::Evidence),
            "cost" => Some(Self::Cost),
            "risk" => Some(Self::Risk),
            "constraint" => Some(Self::Constraint),
            _ => None,
        }
    }
}

impl fmt::Display for AspectV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for AspectV1 {
    type Err = TaxonomyError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value).ok_or_else(|| TaxonomyError::InvalidValue {
            field: "aspect",
            value: value.to_string(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KindV1 {
    Observation,
    Fact,
    Decision,
    Constraint,
    Risk,
    Hypothesis,
    Pattern,
    Issue,
    Rule,
}

impl KindV1 {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Observation => "observation",
            Self::Fact => "fact",
            Self::Decision => "decision",
            Self::Constraint => "constraint",
            Self::Risk => "risk",
            Self::Hypothesis => "hypothesis",
            Self::Pattern => "pattern",
            Self::Issue => "issue",
            Self::Rule => "rule",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "observation" => Some(Self::Observation),
            "fact" => Some(Self::Fact),
            "decision" => Some(Self::Decision),
            "constraint" => Some(Self::Constraint),
            "risk" => Some(Self::Risk),
            "hypothesis" => Some(Self::Hypothesis),
            "pattern" => Some(Self::Pattern),
            "issue" => Some(Self::Issue),
            "rule" => Some(Self::Rule),
            _ => None,
        }
    }
}

impl fmt::Display for KindV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for KindV1 {
    type Err = TaxonomyError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value).ok_or_else(|| TaxonomyError::InvalidValue {
            field: "kind",
            value: value.to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaxonomyPathV1 {
    pub domain: DomainV1,
    pub topic: TopicV1,
    pub aspect: AspectV1,
    pub kind: KindV1,
}

impl TaxonomyPathV1 {
    pub fn new(domain: DomainV1, topic: TopicV1, aspect: AspectV1, kind: KindV1) -> Result<Self, TaxonomyError> {
        let path = Self {
            domain,
            topic,
            aspect,
            kind,
        };
        path.validate()?;
        Ok(path)
    }

    pub fn validate(&self) -> Result<(), TaxonomyError> {
        if !TopicV1::allowed_for(self.domain).contains(&self.topic) {
            return Err(TaxonomyError::TopicNotAllowedForDomain {
                domain: self.domain.as_str(),
                topic: self.topic.as_str(),
            });
        }

        Ok(())
    }

    pub fn from_parts(
        domain: &str,
        topic: &str,
        aspect: &str,
        kind: &str,
    ) -> Result<Self, TaxonomyError> {
        let domain = DomainV1::parse(domain).ok_or_else(|| TaxonomyError::InvalidValue {
            field: "domain",
            value: domain.to_string(),
        })?;
        let topic = TopicV1::parse(topic).ok_or_else(|| TaxonomyError::InvalidValue {
            field: "topic",
            value: topic.to_string(),
        })?;
        let aspect = AspectV1::parse(aspect).ok_or_else(|| TaxonomyError::InvalidValue {
            field: "aspect",
            value: aspect.to_string(),
        })?;
        let kind = KindV1::parse(kind).ok_or_else(|| TaxonomyError::InvalidValue {
            field: "kind",
            value: kind.to_string(),
        })?;

        Self::new(domain, topic, aspect, kind)
    }
}

impl fmt::Display for TaxonomyPathV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}/{}/{}",
            self.domain.as_str(),
            self.topic.as_str(),
            self.aspect.as_str(),
            self.kind.as_str()
        )
    }
}

impl FromStr for TaxonomyPathV1 {
    type Err = TaxonomyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split('/').collect::<Vec<_>>();
        if parts.len() != 4 {
            return Err(TaxonomyError::InvalidPathShape {
                value: s.to_string(),
            });
        }

        Self::from_parts(parts[0], parts[1], parts[2], parts[3])
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TaxonomyError {
    #[error("topic '{topic}' is not allowed for domain '{domain}'")]
    TopicNotAllowedForDomain {
        domain: &'static str,
        topic: &'static str,
    },
    #[error("invalid taxonomy value for {field}: {value}")]
    InvalidValue {
        field: &'static str,
        value: String,
    },
    #[error("invalid taxonomy path shape: {value}")]
    InvalidPathShape {
        value: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topic_sets_are_domain_scoped() {
        assert!(TopicV1::allowed_for(DomainV1::Project).contains(&TopicV1::Retrieval));
        assert!(!TopicV1::allowed_for(DomainV1::External).contains(&TopicV1::Retrieval));
    }

    #[test]
    fn taxonomy_path_rejects_cross_domain_topic() {
        let err = TaxonomyPathV1::new(
            DomainV1::External,
            TopicV1::Testing,
            AspectV1::General,
            KindV1::Observation,
        )
        .expect_err("external should reject project-only topics");

        assert_eq!(
            err,
            TaxonomyError::TopicNotAllowedForDomain {
                domain: "external",
                topic: "testing",
            }
        );
    }

    #[test]
    fn taxonomy_path_accepts_valid_project_path() {
        let path = TaxonomyPathV1::new(
            DomainV1::Project,
            TopicV1::Docs,
            AspectV1::Policy,
            KindV1::Rule,
        )
        .expect("valid project taxonomy path should construct");

        assert_eq!(path.domain.as_str(), "project");
        assert_eq!(path.topic.as_str(), "docs");
        assert_eq!(path.aspect.as_str(), "policy");
        assert_eq!(path.kind.as_str(), "rule");
    }

    #[test]
    fn taxonomy_path_round_trips_through_string_form() {
        let path = TaxonomyPathV1::new(
            DomainV1::System,
            TopicV1::Runtime,
            AspectV1::State,
            KindV1::Observation,
        )
        .expect("taxonomy path should construct");

        let rendered = path.to_string();
        let parsed = TaxonomyPathV1::from_str(&rendered).expect("rendered path should parse");

        assert_eq!(rendered, "system/runtime/state/observation");
        assert_eq!(parsed, path);
    }

    #[test]
    fn enums_round_trip_through_display_and_from_str() {
        assert_eq!(DomainV1::from_str(&DomainV1::Project.to_string()), Ok(DomainV1::Project));
        assert_eq!(TopicV1::from_str(&TopicV1::Retrieval.to_string()), Ok(TopicV1::Retrieval));
        assert_eq!(AspectV1::from_str(&AspectV1::Behavior.to_string()), Ok(AspectV1::Behavior));
        assert_eq!(KindV1::from_str(&KindV1::Decision.to_string()), Ok(KindV1::Decision));
    }
}
