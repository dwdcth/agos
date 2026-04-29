use std::collections::BTreeSet;

use crate::cognition::{
    action::{ActionCandidate, ActionKind, ActionSource},
    assembly::{ActionSeed, WorkingMemoryRequest},
    working_memory::EvidenceFragment,
};
use crate::memory::repository::{
    MemoryRepository, PersistedSkillMemoryTemplateAction, PersistedSkillMemoryTemplateBoundaries,
    PersistedSkillMemoryTemplateCandidate, PersistedSkillMemoryTemplateExpectedOutcome,
    PersistedSkillMemoryTemplatePayload, PersistedSkillMemoryTemplatePreconditions,
    RepositoryError, SKILL_TEMPLATE_PAYLOAD_VERSION,
};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Preconditions {
    pub required_goal_terms: Vec<String>,
    pub required_task_context_terms: Vec<String>,
    pub required_capability_flags: Vec<String>,
    pub required_readiness_flags: Vec<String>,
    pub required_metacog_flag_codes: Vec<String>,
}

impl Preconditions {
    pub fn matches(&self, context: &SkillProjectionContext<'_>) -> bool {
        match_all_terms(
            context.request.active_goal.as_deref(),
            &self.required_goal_terms,
        ) && match_all_terms(
            context.request.task_context.as_deref(),
            &self.required_task_context_terms,
        ) && contains_all(
            &context.request.capability_flags,
            &self.required_capability_flags,
        ) && contains_all(
            &context.request.readiness_flags,
            &self.required_readiness_flags,
        ) && self
            .required_metacog_flag_codes
            .iter()
            .all(|required_code| {
                context
                    .request
                    .metacog_flags
                    .iter()
                    .any(|flag| flag.code == *required_code)
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionTemplate {
    pub kind: ActionKind,
    pub summary: String,
    pub intent: Option<String>,
    pub parameters: Vec<String>,
}

impl ActionTemplate {
    pub fn new(kind: ActionKind, summary: impl Into<String>) -> Self {
        Self {
            kind,
            summary: summary.into(),
            intent: None,
            parameters: Vec::new(),
        }
    }

    pub fn with_intent(mut self, intent: impl Into<String>) -> Self {
        self.intent = Some(intent.into());
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExpectedOutcome {
    pub effects: Vec<String>,
}

impl ExpectedOutcome {
    pub fn from_effect(effect: impl Into<String>) -> Self {
        Self {
            effects: vec![effect.into()],
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Boundaries {
    pub risk_markers: Vec<String>,
    pub supporting_record_ids: Vec<String>,
    pub blocked_active_risks: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillMemoryTemplate {
    pub template_id: String,
    pub preconditions: Preconditions,
    pub action: ActionTemplate,
    pub expected_outcome: ExpectedOutcome,
    pub boundaries: Boundaries,
}

impl SkillMemoryTemplate {
    pub fn new(template_id: impl Into<String>, action: ActionTemplate) -> Self {
        Self {
            template_id: template_id.into(),
            preconditions: Preconditions::default(),
            action,
            expected_outcome: ExpectedOutcome::default(),
            boundaries: Boundaries::default(),
        }
    }

    pub fn project(&self, context: &SkillProjectionContext<'_>) -> Option<ProjectedSkillCandidate> {
        if !self.preconditions.matches(context) {
            return None;
        }
        if self.boundaries.blocked_active_risks.iter().any(|risk| {
            context
                .request
                .active_risks
                .iter()
                .any(|active| active == risk)
        }) {
            return None;
        }
        if !self.boundaries.supporting_record_ids.is_empty()
            && !self
                .boundaries
                .supporting_record_ids
                .iter()
                .all(|record_id| {
                    context
                        .world_fragments
                        .iter()
                        .any(|fragment| fragment.record_id == *record_id)
                })
        {
            return None;
        }

        Some(ProjectedSkillCandidate {
            template_id: self.template_id.clone(),
            action_seed: ActionSeed {
                candidate: ActionCandidate {
                    kind: self.action.kind,
                    summary: self.action.summary.clone(),
                    intent: self.action.intent.clone(),
                    parameters: self.action.parameters.clone(),
                    expected_effects: self.expected_outcome.effects.clone(),
                },
                supporting_record_ids: self.boundaries.supporting_record_ids.clone(),
                risk_markers: self.boundaries.risk_markers.clone(),
                source: ActionSource::SkillTemplate {
                    template_id: self.template_id.clone(),
                },
            },
        })
    }

    pub fn to_candidate_payload(
        &self,
        trigger_kind: impl Into<String>,
        source_report: Value,
        evidence_count: usize,
    ) -> PersistedSkillMemoryTemplatePayload {
        PersistedSkillMemoryTemplatePayload {
            payload_version: SKILL_TEMPLATE_PAYLOAD_VERSION,
            template_id: self.template_id.clone(),
            template_summary: self.action.summary.clone(),
            preconditions: PersistedSkillMemoryTemplatePreconditions {
                required_goal_terms: self.preconditions.required_goal_terms.clone(),
                required_task_context_terms: self.preconditions.required_task_context_terms.clone(),
                required_capability_flags: self.preconditions.required_capability_flags.clone(),
                required_readiness_flags: self.preconditions.required_readiness_flags.clone(),
                required_metacog_flag_codes: self.preconditions.required_metacog_flag_codes.clone(),
            },
            action: PersistedSkillMemoryTemplateAction {
                kind: self.action.kind.as_str().to_string(),
                summary: self.action.summary.clone(),
                intent: self.action.intent.clone(),
                parameters: self.action.parameters.clone(),
            },
            expected_outcome: PersistedSkillMemoryTemplateExpectedOutcome {
                effects: self.expected_outcome.effects.clone(),
            },
            boundaries: PersistedSkillMemoryTemplateBoundaries {
                risk_markers: self.boundaries.risk_markers.clone(),
                supporting_record_ids: self.boundaries.supporting_record_ids.clone(),
                blocked_active_risks: self.boundaries.blocked_active_risks.clone(),
            },
            trigger_kind: trigger_kind.into(),
            source_report,
            evidence_count,
        }
    }

    pub fn from_rumination_candidate(
        candidate: &PersistedSkillMemoryTemplateCandidate,
    ) -> Result<Self, SkillMemoryTemplateDecodeError> {
        if candidate.payload.payload_version != SKILL_TEMPLATE_PAYLOAD_VERSION {
            return Err(SkillMemoryTemplateDecodeError::UnsupportedPayloadVersion {
                candidate_id: candidate.candidate.candidate_id.clone(),
                value: candidate.payload.payload_version,
            });
        }

        let action_kind = ActionKind::parse(&candidate.payload.action.kind).ok_or_else(|| {
            SkillMemoryTemplateDecodeError::InvalidActionKind {
                candidate_id: candidate.candidate.candidate_id.clone(),
                value: candidate.payload.action.kind.clone(),
            }
        })?;

        Ok(Self {
            template_id: candidate.payload.template_id.clone(),
            preconditions: Preconditions {
                required_goal_terms: candidate.payload.preconditions.required_goal_terms.clone(),
                required_task_context_terms: candidate
                    .payload
                    .preconditions
                    .required_task_context_terms
                    .clone(),
                required_capability_flags: candidate
                    .payload
                    .preconditions
                    .required_capability_flags
                    .clone(),
                required_readiness_flags: candidate
                    .payload
                    .preconditions
                    .required_readiness_flags
                    .clone(),
                required_metacog_flag_codes: candidate
                    .payload
                    .preconditions
                    .required_metacog_flag_codes
                    .clone(),
            },
            action: ActionTemplate {
                kind: action_kind,
                summary: candidate.payload.action.summary.clone(),
                intent: candidate.payload.action.intent.clone(),
                parameters: candidate.payload.action.parameters.clone(),
            },
            expected_outcome: ExpectedOutcome {
                effects: candidate.payload.expected_outcome.effects.clone(),
            },
            boundaries: Boundaries {
                risk_markers: candidate.payload.boundaries.risk_markers.clone(),
                supporting_record_ids: candidate.payload.boundaries.supporting_record_ids.clone(),
                blocked_active_risks: candidate.payload.boundaries.blocked_active_risks.clone(),
            },
        })
    }
}

pub fn load_runtime_skill_templates_for_subject(
    repository: &MemoryRepository<'_>,
    subject_ref: &str,
) -> Result<Vec<SkillMemoryTemplate>, RuntimeSkillTemplateLoadError> {
    repository
        .list_consumed_skill_template_candidates_for_subject(subject_ref)?
        .iter()
        .map(SkillMemoryTemplate::from_rumination_candidate)
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

pub fn merge_runtime_skill_templates(
    explicit_templates: &[SkillMemoryTemplate],
    persisted_templates: Vec<SkillMemoryTemplate>,
) -> Vec<SkillMemoryTemplate> {
    let explicit_template_ids = explicit_templates
        .iter()
        .map(|template| template.template_id.clone())
        .collect::<BTreeSet<_>>();
    let mut merged = explicit_templates.to_vec();
    merged.extend(
        persisted_templates
            .into_iter()
            .filter(|template| !explicit_template_ids.contains(&template.template_id)),
    );
    merged
}

#[derive(Debug, Clone, Copy)]
pub struct SkillProjectionContext<'a> {
    pub request: &'a WorkingMemoryRequest,
    pub world_fragments: &'a [EvidenceFragment],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectedSkillCandidate {
    pub template_id: String,
    pub action_seed: ActionSeed,
}

impl ProjectedSkillCandidate {
    pub fn into_action_seed(self) -> ActionSeed {
        self.action_seed
    }
}

fn match_all_terms(haystack: Option<&str>, terms: &[String]) -> bool {
    let normalized = haystack.map(str::to_ascii_lowercase);
    terms.iter().all(|term| {
        normalized
            .as_deref()
            .is_some_and(|value| value.contains(&term.to_ascii_lowercase()))
    })
}

fn contains_all(values: &[String], required_values: &[String]) -> bool {
    required_values
        .iter()
        .all(|required| values.iter().any(|value| value == required))
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SkillMemoryTemplateDecodeError {
    #[error("skill template candidate {candidate_id} stored unsupported payload_version {value}")]
    UnsupportedPayloadVersion { candidate_id: String, value: u32 },
    #[error("skill template candidate {candidate_id} stored invalid action kind {value}")]
    InvalidActionKind { candidate_id: String, value: String },
}

#[derive(Debug, Error)]
pub enum RuntimeSkillTemplateLoadError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error(transparent)]
    Decode(#[from] SkillMemoryTemplateDecodeError),
}
