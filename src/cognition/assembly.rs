use rusqlite::Connection;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::{
    cognition::{
        action::{ActionBranch, ActionCandidate},
        working_memory::{
            ActiveGoal, EvidenceFragment, MetacognitiveFlag, PresentFrame, SelfStateFact,
            SelfStateSnapshot, TruthContext, WorkingMemory, WorkingMemoryBuildError,
        },
    },
    memory::{
        repository::{
            LocalAdaptationEntry, LocalAdaptationTargetKind, MemoryRepository, RepositoryError,
        },
        truth::TruthRecord,
    },
    search::{SearchError, SearchFilters, SearchRequest, SearchResult, SearchService},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionSeed {
    pub candidate: ActionCandidate,
    pub supporting_record_ids: Vec<String>,
    pub risk_markers: Vec<String>,
}

impl ActionSeed {
    pub fn new(candidate: ActionCandidate) -> Self {
        Self {
            candidate,
            supporting_record_ids: Vec::new(),
            risk_markers: Vec::new(),
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
    pub local_adaptation_entries: Vec<LocalAdaptationEntry>,
    pub integrated_results: Vec<SearchResult>,
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
            local_adaptation_entries: Vec::new(),
            integrated_results: Vec::new(),
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

    pub fn with_local_adaptation_entries(
        mut self,
        local_adaptation_entries: Vec<LocalAdaptationEntry>,
    ) -> Self {
        self.local_adaptation_entries = local_adaptation_entries;
        self
    }

    pub fn with_integrated_results(mut self, integrated_results: Vec<SearchResult>) -> Self {
        self.integrated_results = integrated_results;
        self
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
    fn snapshot(&self, request: &WorkingMemoryRequest, truths: &[TruthRecord])
    -> SelfStateSnapshot;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MinimalSelfStateProvider;

impl SelfStateProvider for MinimalSelfStateProvider {
    fn snapshot(
        &self,
        request: &WorkingMemoryRequest,
        truths: &[TruthRecord],
    ) -> SelfStateSnapshot {
        SelfStateSnapshot {
            task_context: request.task_context.clone(),
            capability_flags: request.capability_flags.clone(),
            readiness_flags: request.readiness_flags.clone(),
            facts: request.selected_truth_facts(truths),
        }
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
    fn snapshot(
        &self,
        request: &WorkingMemoryRequest,
        truths: &[TruthRecord],
    ) -> SelfStateSnapshot {
        let mut snapshot = self.base.snapshot(request, truths);
        snapshot.facts.extend(
            request
                .local_adaptation_entries
                .iter()
                .map(local_adaptation_fact),
        );
        snapshot
    }
}

#[derive(Debug, Error)]
pub enum WorkingMemoryAssemblyError {
    #[error(transparent)]
    Search(#[from] SearchError),
    #[error(transparent)]
    Repository(#[from] RepositoryError),
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
        let overlay_request = if let Some(subject_ref) = request.subject_ref.as_deref() {
            let mut local_adaptation_entries =
                self.repository.list_local_adaptation_entries(subject_ref)?;
            local_adaptation_entries.extend(request.local_adaptation_entries.iter().cloned());
            request
                .clone()
                .with_local_adaptation_entries(local_adaptation_entries)
        } else {
            request.clone()
        };
        let mut merged_results = if overlay_request.integrated_results.is_empty() {
            let search_request = SearchRequest::new(overlay_request.query.clone())
                .with_limit(overlay_request.limit)
                .with_filters(overlay_request.filters.clone());
            self.search.search(&search_request)?.results
        } else {
            overlay_request.integrated_results.clone()
        };

        if !overlay_request.integrated_results.is_empty() {
            let search_request = SearchRequest::new(overlay_request.query.clone())
                .with_limit(overlay_request.limit)
                .with_filters(overlay_request.filters.clone());
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
        let mut world_fragments = Vec::with_capacity(merged_results.len());
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
            integrated_result_matches_filters(result, &overlay_request.filters, &layered_records)
        });
        for result in merged_results {
            let truth = self
                .repository
                .get_truth_record(&result.record.id)?
                .ok_or_else(|| WorkingMemoryAssemblyError::MissingTruthProjection {
                    record_id: result.record.id.clone(),
                })?;
            let fragment = EvidenceFragment {
                record_id: result.record.id.clone(),
                snippet: result.snippet,
                citation: result.citation,
                provenance: result.record.provenance.clone(),
                truth_context: TruthContext::from_truth_record(&truth),
                dsl: result.dsl.clone().or_else(|| {
                    layered_records
                        .get(&result.record.id)
                        .and_then(|record| record.dsl.as_ref().map(|dsl| dsl.payload.clone()))
                }),
                trace: result.trace,
                score: result.score,
            };
            truths.push(truth);
            world_fragments.push(fragment);
        }

        let self_state = self.self_state_provider.snapshot(&overlay_request, &truths);
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

        let branches = overlay_request
            .action_seeds
            .iter()
            .map(|seed| materialize_branch(seed, &world_fragments))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(WorkingMemory::builder()
            .present(present)
            .extend_branches(branches)
            .build()?)
    }
}

fn local_adaptation_fact(entry: &LocalAdaptationEntry) -> SelfStateFact {
    let key = match entry.target_kind {
        LocalAdaptationTargetKind::SelfState => format!("self_state:{}", entry.key),
        LocalAdaptationTargetKind::RiskBoundary => format!("risk_boundary:{}", entry.key),
        LocalAdaptationTargetKind::PrivateT3 => format!("private_t3:{}", entry.key),
    };

    SelfStateFact {
        key,
        value: entry.payload.display_value(),
        source_record_id: None,
    }
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
