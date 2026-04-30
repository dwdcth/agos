use rusqlite::Connection;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::{
    cognition::{
        action::{ActionBranch, ActionCandidate, ActionSource},
        attention::{AttentionBaseline, AttentionState},
        self_model::{
            ProjectedSelfModel, RuntimeSelfState, SelfModelReadModel, StableSelfKnowledge,
        },
        skill_memory::{
            ProjectedSkillCandidate, RuntimeSkillTemplateLoadError, SkillMemoryTemplate,
            SkillProjectionContext, load_runtime_skill_templates_for_subject,
            merge_runtime_skill_templates,
        },
        working_memory::{
            ActiveGoal, EvidenceFragment, MetacognitiveFlag, PresentFrame, SelfStateFact,
            SelfStateSnapshot, WorkingMemory, WorkingMemoryBuildError,
        },
        world_model::{
            CurrentWorldSlice, ProjectedWorldModel, WorldFragmentProjection,
            load_runtime_current_world_model,
        },
    },
    memory::{
        repository::{LocalAdaptationEntry, MemoryRepository, RepositoryError},
        truth::TruthRecord,
    },
    search::{SearchError, SearchFilters, SearchRequest, SearchResult, SearchService},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionSeed {
    pub candidate: ActionCandidate,
    pub supporting_record_ids: Vec<String>,
    pub risk_markers: Vec<String>,
    pub source: ActionSource,
}

impl ActionSeed {
    pub fn new(candidate: ActionCandidate) -> Self {
        Self {
            candidate,
            supporting_record_ids: Vec::new(),
            risk_markers: Vec::new(),
            source: ActionSource::Manual,
        }
    }

    pub fn with_supporting_record_ids(mut self, supporting_record_ids: Vec<String>) -> Self {
        self.supporting_record_ids = supporting_record_ids;
        self
    }

    pub fn with_risk_marker(mut self, risk_marker: impl Into<String>) -> Self {
        self.risk_markers.push(risk_marker.into());
        self
    }

    pub fn with_source(mut self, source: ActionSource) -> Self {
        self.source = source;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkingMemoryRequest {
    pub query: String,
    pub limit: usize,
    pub filters: SearchFilters,
    pub subject_ref: Option<String>,
    pub task_context: Option<String>,
    pub active_goal: Option<String>,
    pub active_risks: Vec<String>,
    pub metacog_flags: Vec<MetacognitiveFlag>,
    pub capability_flags: Vec<String>,
    pub readiness_flags: Vec<String>,
    pub action_seeds: Vec<ActionSeed>,
    pub skill_templates: Vec<SkillMemoryTemplate>,
    pub local_adaptation_entries: Vec<LocalAdaptationEntry>,
    pub persisted_self_model: Option<SelfModelReadModel>,
    pub integrated_results: Vec<SearchResult>,
    pub attention_state: Option<AttentionState>,
}

impl WorkingMemoryRequest {
    pub const DEFAULT_LIMIT: usize = SearchRequest::DEFAULT_LIMIT;

    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            limit: Self::DEFAULT_LIMIT,
            filters: SearchFilters::default(),
            subject_ref: None,
            task_context: None,
            active_goal: None,
            active_risks: Vec::new(),
            metacog_flags: Vec::new(),
            capability_flags: Vec::new(),
            readiness_flags: Vec::new(),
            action_seeds: Vec::new(),
            skill_templates: Vec::new(),
            local_adaptation_entries: Vec::new(),
            persisted_self_model: None,
            integrated_results: Vec::new(),
            attention_state: None,
        }
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn with_filters(mut self, filters: SearchFilters) -> Self {
        self.filters = filters;
        self
    }

    pub fn with_subject_ref(mut self, subject_ref: impl Into<String>) -> Self {
        self.subject_ref = Some(subject_ref.into());
        self
    }

    pub fn with_task_context(mut self, task_context: impl Into<String>) -> Self {
        self.task_context = Some(task_context.into());
        self
    }

    pub fn with_active_goal(mut self, active_goal: impl Into<String>) -> Self {
        self.active_goal = Some(active_goal.into());
        self
    }

    pub fn with_active_risk(mut self, active_risk: impl Into<String>) -> Self {
        self.active_risks.push(active_risk.into());
        self
    }

    pub fn with_metacog_flag(mut self, metacog_flag: MetacognitiveFlag) -> Self {
        self.metacog_flags.push(metacog_flag);
        self
    }

    pub fn with_capability_flag(mut self, capability_flag: impl Into<String>) -> Self {
        self.capability_flags.push(capability_flag.into());
        self
    }

    pub fn with_readiness_flag(mut self, readiness_flag: impl Into<String>) -> Self {
        self.readiness_flags.push(readiness_flag.into());
        self
    }

    pub fn with_action_seed(mut self, action_seed: ActionSeed) -> Self {
        self.action_seeds.push(action_seed);
        self
    }

    pub fn with_skill_template(mut self, skill_template: SkillMemoryTemplate) -> Self {
        self.skill_templates.push(skill_template);
        self
    }

    pub fn with_local_adaptation_entries(
        mut self,
        local_adaptation_entries: Vec<LocalAdaptationEntry>,
    ) -> Self {
        self.local_adaptation_entries = local_adaptation_entries;
        self
    }

    pub fn with_persisted_self_model(mut self, persisted_self_model: SelfModelReadModel) -> Self {
        self.persisted_self_model = Some(persisted_self_model);
        self
    }

    pub fn with_integrated_results(mut self, integrated_results: Vec<SearchResult>) -> Self {
        self.integrated_results = integrated_results;
        self
    }

    pub fn with_attention_state(mut self, attention_state: AttentionState) -> Self {
        self.attention_state = Some(attention_state);
        self
    }

    /// Resolve the effective attention state for this request.
    ///
    /// - If an explicit `attention_state` was set, use it (even if empty,
    ///   which disables derived fallback).
    /// - If no explicit state was set, derive from request metadata fields
    ///   (active_goal, active_risks, metacog_flags, readiness_flags, capability_flags).
    /// - If nothing can be derived, return `None`.
    pub fn resolved_attention_state(&self) -> Option<AttentionState> {
        if let Some(ref state) = self.attention_state {
            // Explicit state was set -- use it as-is (even if empty, meaning "no attention").
            return Some(state.clone());
        }

        // No explicit state -- try to derive from metadata.
        let delta = AttentionState::derive_delta(
            self.active_goal.as_deref(),
            &self.active_risks,
            &self.metacog_flags,
            &self.readiness_flags,
            &self.capability_flags,
        );

        if delta.contributions.is_empty() {
            return None;
        }

        let inhibition_constraints = AttentionState::derive_inhibition_constraints(
            &self.capability_flags,
            &self.readiness_flags,
        );
        let metacog_modifier =
            crate::cognition::attention::MetacogModifier::from_flags(&self.metacog_flags);

        Some(AttentionState {
            baseline: AttentionBaseline::default(),
            emotion: crate::cognition::attention::EmotionModulator::default(),
            metacog_modifier,
            delta,
            inhibition_constraints,
        })
    }

    pub fn bounded_limit(&self) -> usize {
        self.limit
            .clamp(1, crate::search::lexical::MAX_RECALL_LIMIT)
    }

    pub fn selected_truth_facts(&self, truths: &[TruthRecord]) -> Vec<SelfStateFact> {
        truths
            .iter()
            .map(|truth| SelfStateFact {
                key: format!("truth_record:{}", truth.record().id),
                value: truth.truth_layer().as_str().to_string(),
                source_record_id: Some(truth.record().id.clone()),
            })
            .collect()
    }
}

pub trait SelfStateProvider {
    fn project(&self, request: &WorkingMemoryRequest, truths: &[TruthRecord])
    -> ProjectedSelfModel;

    fn snapshot(
        &self,
        request: &WorkingMemoryRequest,
        truths: &[TruthRecord],
    ) -> SelfStateSnapshot {
        self.project(request, truths).project_snapshot()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MinimalSelfStateProvider;

impl SelfStateProvider for MinimalSelfStateProvider {
    fn project(
        &self,
        request: &WorkingMemoryRequest,
        truths: &[TruthRecord],
    ) -> ProjectedSelfModel {
        ProjectedSelfModel::new(
            StableSelfKnowledge {
                capability_flags: request.capability_flags.clone(),
                facts: request.selected_truth_facts(truths),
            },
            RuntimeSelfState {
                task_context: request.task_context.clone(),
                readiness_flags: request.readiness_flags.clone(),
                facts: Vec::new(),
            },
        )
    }
}

#[derive(Debug, Clone)]
pub struct AdaptiveSelfStateProvider<P> {
    base: P,
}

impl<P> AdaptiveSelfStateProvider<P> {
    pub fn new(base: P) -> Self {
        Self { base }
    }
}

impl<P> SelfStateProvider for AdaptiveSelfStateProvider<P>
where
    P: SelfStateProvider,
{
    fn project(
        &self,
        request: &WorkingMemoryRequest,
        truths: &[TruthRecord],
    ) -> ProjectedSelfModel {
        let mut model = self.base.project(request, truths);
        let read_model = request.persisted_self_model.clone().unwrap_or_else(|| {
            SelfModelReadModel::from_overlay_entries(&request.local_adaptation_entries)
        });
        model.runtime.facts.extend(read_model.active_facts());
        model
    }
}

#[derive(Debug, Error)]
pub enum WorkingMemoryAssemblyError {
    #[error(transparent)]
    Search(#[from] SearchError),
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error(transparent)]
    RuntimeSkillTemplates(#[from] RuntimeSkillTemplateLoadError),
    #[error(transparent)]
    Build(#[from] WorkingMemoryBuildError),
    #[error("missing truth projection for retrieved record {record_id}")]
    MissingTruthProjection { record_id: String },
    #[error(
        "action seed references record {record_id}, but it was not present in retrieved fragments"
    )]
    MissingSupportingRecord { record_id: String },
}

pub struct WorkingMemoryAssembler<'db, P> {
    search: SearchService<'db>,
    repository: MemoryRepository<'db>,
    self_state_provider: P,
}

impl<'db, P> WorkingMemoryAssembler<'db, P>
where
    P: SelfStateProvider,
{
    pub fn new(conn: &'db Connection, self_state_provider: P) -> Self {
        Self {
            search: SearchService::new(conn),
            repository: MemoryRepository::new(conn),
            self_state_provider,
        }
    }

    pub fn assemble(
        &self,
        request: &WorkingMemoryRequest,
    ) -> Result<WorkingMemory, WorkingMemoryAssemblyError> {
        let persisted_skill_templates = request
            .subject_ref
            .as_deref()
            .map(|subject_ref| {
                load_runtime_skill_templates_for_subject(&self.repository, subject_ref)
            })
            .transpose()?
            .unwrap_or_default();
        let persisted_self_model =
            if let Some(persisted_self_model) = request.persisted_self_model.clone() {
                persisted_self_model
            } else if let Some(subject_ref) = request.subject_ref.as_deref() {
                let persisted_state = self.repository.load_self_model_state(subject_ref)?;
                SelfModelReadModel::from_persisted_state(
                    persisted_state.snapshot.as_ref(),
                    &persisted_state.tail_entries,
                    &request.local_adaptation_entries,
                )
            } else {
                SelfModelReadModel::from_overlay_entries(&request.local_adaptation_entries)
            };
        let mut overlay_request = request
            .clone()
            .with_persisted_self_model(persisted_self_model);
        overlay_request.skill_templates =
            merge_runtime_skill_templates(&request.skill_templates, persisted_skill_templates);
        let (world_model, truths) = self.resolve_world_model(&overlay_request)?;
        let world_fragments = world_model.project_fragments();

        let self_model = self.self_state_provider.project(&overlay_request, &truths);
        let self_state = self_model.project_snapshot();
        let present = PresentFrame {
            world_fragments: world_fragments.clone(),
            self_state,
            active_goal: overlay_request
                .active_goal
                .clone()
                .map(|summary| ActiveGoal { summary }),
            active_risks: overlay_request.active_risks.clone(),
            metacog_flags: overlay_request.metacog_flags.clone(),
        };

        let skill_seeds = project_skill_action_seeds(&overlay_request, &world_fragments);
        let action_seeds = overlay_request
            .action_seeds
            .iter()
            .cloned()
            .chain(skill_seeds)
            .collect::<Vec<_>>();
        let branches = action_seeds
            .iter()
            .map(|seed| materialize_branch(seed, &world_fragments))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(WorkingMemory::builder()
            .present(present)
            .extend_branches(branches)
            .build()?)
    }

    fn resolve_world_model(
        &self,
        request: &WorkingMemoryRequest,
    ) -> Result<(ProjectedWorldModel, Vec<TruthRecord>), WorkingMemoryAssemblyError> {
        if request.integrated_results.is_empty()
            && let Some(subject_ref) = request.subject_ref.as_deref()
            && let Some(world_model) =
                load_runtime_current_world_model(&self.repository, subject_ref)?
        {
            let truths = self.load_truths_for_world_model(&world_model)?;
            return Ok((world_model, truths));
        }

        self.project_live_world_model(request)
    }

    fn project_live_world_model(
        &self,
        request: &WorkingMemoryRequest,
    ) -> Result<(ProjectedWorldModel, Vec<TruthRecord>), WorkingMemoryAssemblyError> {
        let resolved_attention = request.resolved_attention_state();
        let mut merged_results = if request.integrated_results.is_empty() {
            let mut search_request = SearchRequest::new(request.query.clone())
                .with_limit(request.limit)
                .with_filters(request.filters.clone());
            if let Some(ref attention) = resolved_attention {
                search_request = search_request.with_attention_state(attention.clone());
            }
            self.search.search(&search_request)?.results
        } else {
            request.integrated_results.clone()
        };

        if !request.integrated_results.is_empty() {
            let mut search_request = SearchRequest::new(request.query.clone())
                .with_limit(request.limit)
                .with_filters(request.filters.clone());
            if let Some(ref attention) = resolved_attention {
                search_request = search_request.with_attention_state(attention.clone());
            }
            for result in self.search.search(&search_request)?.results {
                if !merged_results
                    .iter()
                    .any(|existing| existing.record.id == result.record.id)
                {
                    merged_results.push(result);
                }
            }
        }
        let mut seen_record_ids = BTreeSet::new();
        merged_results.retain(|result| seen_record_ids.insert(result.record.id.clone()));

        let mut truths = Vec::with_capacity(merged_results.len());
        let mut world_fragment_projections = Vec::with_capacity(merged_results.len());
        let layered_records = merged_results
            .iter()
            .any(|result| result.dsl.is_none())
            .then(|| {
                self.repository.list_layered_records_for_ids(
                    &merged_results
                        .iter()
                        .map(|result| result.record.id.clone())
                        .collect::<Vec<_>>(),
                )
            })
            .transpose()?
            .unwrap_or_default()
            .into_iter()
            .map(|record| (record.record.id.clone(), record))
            .collect::<BTreeMap<_, _>>();
        merged_results.retain(|result| {
            integrated_result_matches_filters(result, &request.filters, &layered_records)
        });
        for result in merged_results {
            let record_id = result.record.id.clone();
            let truth = self
                .repository
                .get_truth_record(&record_id)?
                .ok_or_else(|| WorkingMemoryAssemblyError::MissingTruthProjection {
                    record_id: record_id.clone(),
                })?;
            let projection = WorldFragmentProjection::from_search_result(
                result,
                &truth,
                layered_records
                    .get(&record_id)
                    .and_then(|record| record.dsl.as_ref().map(|dsl| &dsl.payload)),
            );
            truths.push(truth);
            world_fragment_projections.push(projection);
        }

        Ok((
            ProjectedWorldModel::new(CurrentWorldSlice::new(world_fragment_projections)),
            truths,
        ))
    }

    fn load_truths_for_world_model(
        &self,
        world_model: &ProjectedWorldModel,
    ) -> Result<Vec<TruthRecord>, WorkingMemoryAssemblyError> {
        let mut truths = Vec::with_capacity(world_model.current.fragments.len());
        let mut seen_record_ids = BTreeSet::new();

        for fragment in &world_model.current.fragments {
            if !seen_record_ids.insert(fragment.record_id.clone()) {
                continue;
            }

            let truth = self
                .repository
                .get_truth_record(&fragment.record_id)?
                .ok_or_else(|| WorkingMemoryAssemblyError::MissingTruthProjection {
                    record_id: fragment.record_id.clone(),
                })?;
            truths.push(truth);
        }

        Ok(truths)
    }
}

pub fn project_skill_action_seeds(
    request: &WorkingMemoryRequest,
    world_fragments: &[EvidenceFragment],
) -> Vec<ActionSeed> {
    let context = SkillProjectionContext {
        request,
        world_fragments,
    };

    request
        .skill_templates
        .iter()
        .filter_map(|template| template.project(&context))
        .map(ProjectedSkillCandidate::into_action_seed)
        .collect()
}

fn materialize_branch(
    seed: &ActionSeed,
    world_fragments: &[EvidenceFragment],
) -> Result<ActionBranch, WorkingMemoryAssemblyError> {
    let supporting_evidence = if seed.supporting_record_ids.is_empty() {
        world_fragments.to_vec()
    } else {
        let mut seen_record_ids = BTreeSet::new();
        let mut evidence = Vec::with_capacity(seed.supporting_record_ids.len());
        for record_id in &seed.supporting_record_ids {
            if !seen_record_ids.insert(record_id.clone()) {
                continue;
            }
            let fragment = world_fragments
                .iter()
                .find(|fragment| fragment.record_id == *record_id)
                .cloned()
                .ok_or_else(|| WorkingMemoryAssemblyError::MissingSupportingRecord {
                    record_id: record_id.clone(),
                })?;
            evidence.push(fragment);
        }
        evidence
    };

    Ok(ActionBranch {
        candidate: seed.candidate.clone(),
        supporting_evidence,
        risk_markers: seed.risk_markers.clone(),
        source: seed.source.clone(),
    })
}

fn integrated_result_matches_filters(
    result: &SearchResult,
    filters: &SearchFilters,
    layered_records: &BTreeMap<String, crate::memory::repository::LayeredMemoryRecord>,
) -> bool {
    if let Some(scope) = filters.scope
        && result.record.scope != scope
    {
        return false;
    }
    if let Some(record_type) = filters.record_type
        && result.record.record_type != record_type
    {
        return false;
    }
    if let Some(truth_layer) = filters.truth_layer
        && result.record.truth_layer != truth_layer
    {
        return false;
    }
    if let Some(valid_at) = filters.valid_at.as_deref() {
        if let Some(valid_from) = result.record.validity.valid_from.as_deref()
            && !matches_required_timestamp(valid_from, valid_at, TimestampComparison::LessOrEqual)
        {
            return false;
        }
        if let Some(valid_to) = result.record.validity.valid_to.as_deref()
            && !matches_required_timestamp(valid_to, valid_at, TimestampComparison::GreaterOrEqual)
        {
            return false;
        }
    }
    if let Some(recorded_from) = filters.recorded_from.as_deref()
        && !matches_required_timestamp(
            result.record.timestamp.recorded_at.as_str(),
            recorded_from,
            TimestampComparison::GreaterOrEqual,
        )
    {
        return false;
    }
    if let Some(recorded_to) = filters.recorded_to.as_deref()
        && !matches_required_timestamp(
            result.record.timestamp.recorded_at.as_str(),
            recorded_to,
            TimestampComparison::LessOrEqual,
        )
    {
        return false;
    }
    if filters.domain.is_some()
        || filters.topic.is_some()
        || filters.aspect.is_some()
        || filters.kind.is_some()
    {
        let dsl = result.dsl.as_ref().or_else(|| {
            layered_records
                .get(&result.record.id)
                .and_then(|record| record.dsl.as_ref().map(|dsl| &dsl.payload))
        });
        let Some(dsl) = dsl else {
            return false;
        };
        for (expected, actual) in [
            (filters.domain.as_deref(), dsl.domain.as_str()),
            (filters.topic.as_deref(), dsl.topic.as_str()),
            (filters.aspect.as_deref(), dsl.aspect.as_str()),
            (filters.kind.as_deref(), dsl.kind.as_str()),
        ] {
            if let Some(expected) = expected
                && !expected.eq_ignore_ascii_case(actual)
            {
                return false;
            }
        }
    }

    true
}

#[derive(Clone, Copy)]
enum TimestampComparison {
    LessOrEqual,
    GreaterOrEqual,
}

fn matches_required_timestamp(
    candidate: &str,
    filter: &str,
    comparison: TimestampComparison,
) -> bool {
    match (parse_rfc3339(candidate), parse_rfc3339(filter)) {
        (Some(candidate), Some(filter)) => compare_timestamps(candidate, filter, comparison),
        _ => compare_timestamp_strings(candidate, filter, comparison),
    }
}

fn parse_rfc3339(value: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(value, &Rfc3339).ok()
}

fn compare_timestamps(
    candidate: OffsetDateTime,
    filter: OffsetDateTime,
    comparison: TimestampComparison,
) -> bool {
    match comparison {
        TimestampComparison::LessOrEqual => candidate <= filter,
        TimestampComparison::GreaterOrEqual => candidate >= filter,
    }
}

fn compare_timestamp_strings(
    candidate: &str,
    filter: &str,
    comparison: TimestampComparison,
) -> bool {
    match comparison {
        TimestampComparison::LessOrEqual => candidate <= filter,
        TimestampComparison::GreaterOrEqual => candidate >= filter,
    }
}
