use rusqlite::Connection;
use thiserror::Error;

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
    search::{SearchError, SearchFilters, SearchRequest, SearchService},
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

    pub fn bounded_limit(&self) -> usize {
        self.limit.clamp(1, SearchRequest::DEFAULT_LIMIT.max(self.limit))
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
    fn snapshot(&self, request: &WorkingMemoryRequest, truths: &[TruthRecord]) -> SelfStateSnapshot;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MinimalSelfStateProvider;

impl SelfStateProvider for MinimalSelfStateProvider {
    fn snapshot(&self, request: &WorkingMemoryRequest, truths: &[TruthRecord]) -> SelfStateSnapshot {
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
    fn snapshot(&self, request: &WorkingMemoryRequest, truths: &[TruthRecord]) -> SelfStateSnapshot {
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
    #[error("action seed references record {record_id}, but it was not present in retrieved fragments")]
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
            request.clone().with_local_adaptation_entries(
                self.repository.list_local_adaptation_entries(subject_ref)?,
            )
        } else {
            request.clone()
        };
        let search_request = SearchRequest::new(overlay_request.query.clone())
            .with_limit(overlay_request.limit)
            .with_filters(overlay_request.filters.clone());
        let search_response = self.search.search(&search_request)?;

        let mut truths = Vec::with_capacity(search_response.results.len());
        let mut world_fragments = Vec::with_capacity(search_response.results.len());
        for result in search_response.results {
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
                truth_context: TruthContext::from_truth_record(&truth),
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
        let mut evidence = Vec::with_capacity(seed.supporting_record_ids.len());
        for record_id in &seed.supporting_record_ids {
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
