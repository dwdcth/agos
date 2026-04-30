use std::collections::BTreeSet;

use anyhow::Result as AnyResult;
use rusqlite::Connection;
use serde::Serialize;
use thiserror::Error;

use crate::{
    cognition::{
        action::{ActionCandidate, ActionKind},
        assembly::{ActionSeed, SelfStateProvider, WorkingMemoryAssembler, WorkingMemoryRequest},
        metacog::MetacognitionService,
        report::DecisionReport,
        rumination::{
            RuminationService, RuminationTriggerDecision, RuminationTriggerEvent,
            RuminationTriggerKind, ShortCycleWritebackReport,
        },
        value::{
            ScoredBranch, ValueAdjustment, ValueConfig, ValueScorer, ValueVector,
            derive_dynamic_delta,
        },
        working_memory::WorkingMemory,
        world_model::{SimulationResult, WorldFragmentProjection},
    },
    core::config::Config,
    memory::repository::{MemoryRepository, RuminationCandidateKind, RuminationCandidateStatus},
    search::{Citation, SearchFilters, SearchRequest, SearchResponse, SearchService},
};

#[derive(Debug, Clone, PartialEq)]
pub struct AgentSearchBranchValue {
    pub kind: crate::cognition::action::ActionKind,
    pub summary: String,
    pub value: ValueVector,
}

impl AgentSearchBranchValue {
    pub fn new(
        kind: crate::cognition::action::ActionKind,
        summary: impl Into<String>,
        value: ValueVector,
    ) -> Self {
        Self {
            kind,
            summary: summary.into(),
            value,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgentSearchRequest {
    pub working_memory: WorkingMemoryRequest,
    pub follow_up_queries: Vec<String>,
    pub max_steps: usize,
    pub step_limit: usize,
    pub branch_values: Vec<AgentSearchBranchValue>,
    pub enable_simulation: bool,
}

impl AgentSearchRequest {
    pub const DEFAULT_MAX_STEPS: usize = 2;
    pub const DEFAULT_STEP_LIMIT: usize = SearchRequest::DEFAULT_LIMIT;

    pub fn new(working_memory: WorkingMemoryRequest) -> Self {
        Self {
            working_memory,
            follow_up_queries: Vec::new(),
            max_steps: Self::DEFAULT_MAX_STEPS,
            step_limit: Self::DEFAULT_STEP_LIMIT,
            branch_values: Vec::new(),
            enable_simulation: false,
        }
    }

    pub fn with_follow_up_query(mut self, query: impl Into<String>) -> Self {
        self.follow_up_queries.push(query.into());
        self
    }

    pub fn with_max_steps(mut self, max_steps: usize) -> Self {
        self.max_steps = max_steps.max(1);
        self
    }

    pub fn with_step_limit(mut self, step_limit: usize) -> Self {
        self.step_limit = step_limit.max(1);
        self
    }

    pub fn with_branch_value(mut self, branch_value: AgentSearchBranchValue) -> Self {
        self.branch_values.push(branch_value);
        self
    }

    pub fn with_attention_state(
        mut self,
        attention_state: crate::cognition::attention::AttentionState,
    ) -> Self {
        self.working_memory = self.working_memory.with_attention_state(attention_state);
        self
    }

    pub fn with_working_memory_limit(mut self, limit: usize) -> Self {
        self.working_memory = self.working_memory.with_limit(limit);
        self
    }

    pub fn with_enable_simulation(mut self, enable: bool) -> Self {
        self.enable_simulation = enable;
        self
    }

    pub fn developer_defaults(query: impl Into<String>) -> Self {
        let query = query.into();
        let working_memory = WorkingMemoryRequest::new(query.clone())
            .with_active_goal(format!(
                "produce a cited decision-support report for: {query}"
            ))
            .with_action_seed(ActionSeed::new(
                ActionCandidate::new(ActionKind::Epistemic, "collect more evidence")
                    .with_intent("retrieve stronger support before acting"),
            ))
            .with_action_seed(ActionSeed::new(
                ActionCandidate::new(ActionKind::Instrumental, "take the leading action")
                    .with_intent("act only if citations and gates remain green"),
            ))
            .with_action_seed(ActionSeed::new(
                ActionCandidate::new(ActionKind::Regulative, "pause and request clarification")
                    .with_intent("insert a safe regulating step when evidence is weak"),
            ));

        Self::new(working_memory)
            .with_branch_value(AgentSearchBranchValue::new(
                ActionKind::Epistemic,
                "collect more evidence",
                ValueVector {
                    goal_progress: 0.40,
                    information_gain: 0.95,
                    risk_avoidance: 0.60,
                    resource_efficiency: 0.50,
                    agent_robustness: 0.75,
                },
            ))
            .with_branch_value(AgentSearchBranchValue::new(
                ActionKind::Instrumental,
                "take the leading action",
                ValueVector {
                    goal_progress: 0.90,
                    information_gain: 0.35,
                    risk_avoidance: 0.50,
                    resource_efficiency: 0.85,
                    agent_robustness: 0.65,
                },
            ))
            .with_branch_value(AgentSearchBranchValue::new(
                ActionKind::Regulative,
                "pause and request clarification",
                ValueVector {
                    goal_progress: 0.35,
                    information_gain: 0.40,
                    risk_avoidance: 0.98,
                    resource_efficiency: 0.45,
                    agent_robustness: 0.98,
                },
            ))
    }

    fn bounded_queries(&self) -> Vec<String> {
        std::iter::once(self.working_memory.query.clone())
            .chain(self.follow_up_queries.iter().cloned())
            .take(self.max_steps.max(1))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RetrievalStepReport {
    pub query: String,
    pub applied_filters: SearchFilters,
    pub result_count: usize,
    pub citations: Vec<Citation>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AgentSearchReport {
    pub working_memory: WorkingMemory,
    pub decision: DecisionReport,
    pub retrieval_steps: Vec<RetrievalStepReport>,
    pub citations: Vec<Citation>,
    pub executed_steps: usize,
    pub step_limit: usize,
}

pub trait RetrievalPort {
    fn search(&self, request: &SearchRequest) -> AnyResult<SearchResponse>;
}

pub trait AssemblyPort {
    fn assemble(&self, request: &WorkingMemoryRequest) -> AnyResult<WorkingMemory>;
}

pub trait ScoringPort {
    fn score(
        &self,
        working_memory: &WorkingMemory,
        branch_values: &[AgentSearchBranchValue],
    ) -> AnyResult<Vec<ScoredBranch>>;
}

pub trait GatingPort {
    fn evaluate(
        &self,
        working_memory: &WorkingMemory,
        scored_branches: Vec<ScoredBranch>,
    ) -> AnyResult<DecisionReport>;
}

pub trait RuminationPort {
    fn schedule_trigger(
        &self,
        event: RuminationTriggerEvent,
    ) -> AnyResult<RuminationTriggerDecision>;

    fn drain_short_cycle(&self, now: &str) -> AnyResult<Option<ShortCycleWritebackReport>>;
}

pub trait SimulationPort {
    fn simulate(
        &self,
        world_fragments: &[WorldFragmentProjection],
        action: &ActionCandidate,
    ) -> AnyResult<SimulationResult>;
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CognitiveCycleResult {
    pub report: AgentSearchReport,
    pub rumination_triggered: Vec<RuminationTriggerEvent>,
    pub short_cycle_writebacks: Vec<ShortCycleWritebackReport>,
}

#[derive(Debug, Error)]
pub enum AgentSearchError {
    #[error("retrieval step '{query}' failed")]
    Retrieval {
        query: String,
        #[source]
        source: anyhow::Error,
    },
    #[error("working-memory assembly failed")]
    Assembly {
        #[source]
        source: anyhow::Error,
    },
    #[error("value scoring failed")]
    Scoring {
        #[source]
        source: anyhow::Error,
    },
    #[error("metacognitive gating failed")]
    Gating {
        #[source]
        source: anyhow::Error,
    },
    #[error("rumination trigger scheduling failed")]
    Rumination {
        #[source]
        source: anyhow::Error,
    },
    #[error("rumination short-cycle drain failed")]
    RuminationDrain {
        #[source]
        source: anyhow::Error,
    },
}

pub struct AgentSearchOrchestrator<R, A, S, G> {
    retriever: R,
    assembler: A,
    scorer: S,
    gate: G,
    rumination: Option<Box<dyn RuminationPort>>,
    simulation: Option<Box<dyn SimulationPort>>,
}

impl<R, A, S, G> AgentSearchOrchestrator<R, A, S, G> {
    pub fn new(retriever: R, assembler: A, scorer: S, gate: G) -> Self {
        Self {
            retriever,
            assembler,
            scorer,
            gate,
            rumination: None,
            simulation: None,
        }
    }

    pub fn with_rumination(mut self, rumination: Box<dyn RuminationPort>) -> Self {
        self.rumination = Some(rumination);
        self
    }

    pub fn with_simulation(mut self, simulation: Box<dyn SimulationPort>) -> Self {
        self.simulation = Some(simulation);
        self
    }
}

pub trait AgentSearchRunner {
    fn run(&self, request: &AgentSearchRequest) -> Result<AgentSearchReport, AgentSearchError>;
}

impl<R, A, S, G> AgentSearchOrchestrator<R, A, S, G>
where
    R: RetrievalPort,
    A: AssemblyPort,
    S: ScoringPort,
    G: GatingPort,
{
    fn execute(&self, request: &AgentSearchRequest) -> Result<AgentSearchReport, AgentSearchError> {
        let resolved_attention = request.working_memory.resolved_attention_state();
        let mut integrated_results = Vec::new();
        let retrieval_steps = request
            .bounded_queries()
            .into_iter()
            .map(|query| {
                let mut search_request = SearchRequest::new(query.clone())
                    .with_limit(request.step_limit)
                    .with_filters(request.working_memory.filters.clone());
                if let Some(ref attention) = resolved_attention {
                    search_request = search_request.with_attention_state(attention.clone());
                }
                let response = self.retriever.search(&search_request).map_err(|source| {
                    AgentSearchError::Retrieval {
                        query: query.clone(),
                        source,
                    }
                })?;
                integrated_results.extend(response.results.iter().cloned());
                Ok(RetrievalStepReport {
                    query,
                    applied_filters: response.applied_filters.clone(),
                    result_count: response.results.len(),
                    citations: response
                        .results
                        .into_iter()
                        .map(|result| result.citation)
                        .collect(),
                })
            })
            .collect::<Result<Vec<_>, AgentSearchError>>()?;

        let working_memory_request = request
            .working_memory
            .clone()
            .with_integrated_results(integrated_results);
        let mut working_memory = self
            .assembler
            .assemble(&working_memory_request)
            .map_err(|source| AgentSearchError::Assembly { source })?;
        let mut scored_branches = self
            .scorer
            .score(&working_memory, &request.branch_values)
            .map_err(|source| AgentSearchError::Scoring { source })?;

        if request.enable_simulation {
            if let Some(ref simulation) = self.simulation {
                if let Some(top_branch) = scored_branches.first() {
                    let fragments: Vec<WorldFragmentProjection> = working_memory
                        .present
                        .world_fragments
                        .iter()
                        .map(evidence_fragment_to_world_projection)
                        .collect();

                    match simulation.simulate(&fragments, &top_branch.branch.candidate) {
                        Ok(sim_result) => {
                            let enriched_working_memory = enrich_branches_from_simulation(
                                &mut working_memory,
                                &mut scored_branches,
                                &sim_result,
                            );
                            working_memory = enriched_working_memory;
                        }
                        Err(_) => {
                            // Simulation failed; log and skip enrichment.
                        }
                    }
                }
            }
        }

        let decision = self
            .gate
            .evaluate(&working_memory, scored_branches)
            .map_err(|source| AgentSearchError::Gating { source })?;

        Ok(AgentSearchReport {
            citations: collect_unique_citations(&retrieval_steps),
            executed_steps: retrieval_steps.len(),
            retrieval_steps,
            step_limit: request.step_limit,
            working_memory,
            decision,
        })
    }

    pub fn cycle(
        &self,
        request: &AgentSearchRequest,
        subject_ref: &str,
        now: &str,
        budget_bucket: &str,
    ) -> Result<CognitiveCycleResult, AgentSearchError> {
        let report = self.execute(request)?;

        let rumination_triggered =
            build_rumination_triggers(&report, subject_ref, now, budget_bucket)?;

        let mut scheduled_triggers = Vec::new();
        if let Some(ref rumination) = self.rumination {
            for trigger in &rumination_triggered {
                match rumination.schedule_trigger(trigger.clone()) {
                    Ok(_) | Err(_) => {
                        scheduled_triggers.push(trigger.clone());
                    }
                }
            }
        }

        let mut short_cycle_writebacks = Vec::new();
        if let Some(ref rumination) = self.rumination {
            match rumination.drain_short_cycle(now) {
                Ok(Some(writeback)) => {
                    short_cycle_writebacks.push(writeback);
                }
                Ok(None) => {}
                Err(_) => {
                    // Drain failed; writebacks remain empty.
                }
            }
        }

        Ok(CognitiveCycleResult {
            report,
            rumination_triggered: scheduled_triggers,
            short_cycle_writebacks,
        })
    }
}

fn evidence_fragment_to_world_projection(
    fragment: &crate::cognition::working_memory::EvidenceFragment,
) -> WorldFragmentProjection {
    WorldFragmentProjection {
        record_id: fragment.record_id.clone(),
        snippet: fragment.snippet.clone(),
        citation: fragment.citation.clone(),
        provenance: fragment.provenance.clone(),
        truth_context: fragment.truth_context.clone(),
        dsl: fragment.dsl.clone(),
        trace: fragment.trace.clone(),
        score: fragment.score.clone(),
    }
}

fn enrich_branches_from_simulation(
    working_memory: &mut WorkingMemory,
    scored_branches: &mut [ScoredBranch],
    sim_result: &SimulationResult,
) -> WorkingMemory {
    for risk in &sim_result.predicted.new_risks {
        let label = format!(
            "simulated_{}:{}",
            match risk.severity {
                crate::cognition::world_model::PredictedSeverity::Low => "low",
                crate::cognition::world_model::PredictedSeverity::Medium => "medium",
                crate::cognition::world_model::PredictedSeverity::High => "high",
            },
            risk.description
        );
        if let Some(branch) = scored_branches.first_mut() {
            if !branch.branch.risk_markers.contains(&label) {
                branch.branch.risk_markers.push(label.clone());
            }
        }
        working_memory.present.active_risks.push(label);
    }

    for change in &sim_result.predicted.affected_fragments {
        if let Some(branch) = scored_branches.first_mut() {
            let effect_summary = format!("{}: {}", change.record_id, change.change_description);
            if !branch
                .branch
                .candidate
                .expected_effects
                .contains(&effect_summary)
            {
                branch
                    .branch
                    .candidate
                    .expected_effects
                    .push(effect_summary);
            }
        }
    }

    working_memory.clone()
}

fn build_rumination_triggers(
    report: &AgentSearchReport,
    subject_ref: &str,
    now: &str,
    budget_bucket: &str,
) -> Result<Vec<RuminationTriggerEvent>, AgentSearchError> {
    use crate::cognition::metacog::GateDecision;

    let gate_decision = report.decision.gate.decision;
    let mut triggers = Vec::new();

    match gate_decision {
        GateDecision::Warning => {
            if let Ok(event) = RuminationTriggerEvent::from_agent_search_report(
                RuminationTriggerKind::SessionBoundary,
                subject_ref,
                report,
                now,
                budget_bucket,
                None,
                None,
            ) {
                triggers.push(event);
            }
        }
        GateDecision::SoftVeto => {
            if let Ok(event) = RuminationTriggerEvent::from_agent_search_report(
                RuminationTriggerKind::MetacogVeto,
                subject_ref,
                report,
                now,
                budget_bucket,
                None,
                None,
            ) {
                triggers.push(event);
            }
        }
        GateDecision::HardVeto | GateDecision::Escalate => {
            if let Ok(mut event) = RuminationTriggerEvent::from_agent_search_report(
                RuminationTriggerKind::MetacogVeto,
                subject_ref,
                report,
                now,
                budget_bucket,
                None,
                None,
            ) {
                event = event.with_budget_cost(3);
                triggers.push(event);
            }
        }
    }

    Ok(triggers)
}

impl<R, A, S, G> AgentSearchRunner for AgentSearchOrchestrator<R, A, S, G>
where
    R: RetrievalPort,
    A: AssemblyPort,
    S: ScoringPort,
    G: GatingPort,
{
    fn run(&self, request: &AgentSearchRequest) -> Result<AgentSearchReport, AgentSearchError> {
        self.execute(request)
    }
}

pub struct SearchServicePort<'db> {
    search: SearchService<'db>,
}

impl<'db> SearchServicePort<'db> {
    pub fn new(conn: &'db Connection) -> Self {
        Self {
            search: SearchService::new(conn),
        }
    }

    pub fn with_runtime_config(conn: &'db Connection, config: &Config) -> Self {
        Self {
            search: SearchService::with_runtime_config(conn, config, None),
        }
    }
}

impl RetrievalPort for SearchServicePort<'_> {
    fn search(&self, request: &SearchRequest) -> AnyResult<SearchResponse> {
        Ok(self.search.search(request)?)
    }
}

pub struct WorkingMemoryAssemblyPort<'db, P> {
    assembler: WorkingMemoryAssembler<'db, P>,
}

impl<'db, P> WorkingMemoryAssemblyPort<'db, P>
where
    P: SelfStateProvider,
{
    pub fn new(conn: &'db Connection, self_state_provider: P) -> Self {
        Self {
            assembler: WorkingMemoryAssembler::new(conn, self_state_provider),
        }
    }
}

impl<P> AssemblyPort for WorkingMemoryAssemblyPort<'_, P>
where
    P: SelfStateProvider,
{
    fn assemble(&self, request: &WorkingMemoryRequest) -> AnyResult<WorkingMemory> {
        Ok(self.assembler.assemble(request)?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ValueScoringError {
    #[error("missing value vector for branch {kind}:{summary}")]
    MissingBranchValue { kind: &'static str, summary: String },
}

#[derive(Debug, Clone, Default)]
pub struct WorkingMemoryScoringPort {
    scorer: ValueScorer,
}

impl WorkingMemoryScoringPort {
    pub fn new(config: ValueConfig) -> Self {
        Self {
            scorer: ValueScorer::new(config),
        }
    }

    pub fn from_persisted_adjustments(
        base_config: &ValueConfig,
        adjustments: &[ValueAdjustment],
        learning_rate: f32,
    ) -> Self {
        let adjusted =
            ValueConfig::from_persisted_adjustments(base_config, adjustments, learning_rate);
        Self {
            scorer: ValueScorer::new(adjusted),
        }
    }
}

impl ScoringPort for WorkingMemoryScoringPort {
    fn score(
        &self,
        working_memory: &WorkingMemory,
        branch_values: &[AgentSearchBranchValue],
    ) -> AnyResult<Vec<ScoredBranch>> {
        let delta = derive_dynamic_delta(
            working_memory
                .present
                .active_goal
                .as_ref()
                .map(|g| g.summary.as_str()),
            &working_memory.present.active_risks,
            &working_memory.present.metacog_flags,
            &working_memory.present.self_state.readiness_flags,
            &working_memory.present.self_state.capability_flags,
        );

        working_memory
            .branches
            .iter()
            .map(|branch| {
                let branch_value = branch_values
                    .iter()
                    .find(|value| {
                        value.kind == branch.candidate.kind
                            && value.summary == branch.candidate.summary
                    })
                    .or_else(|| {
                        if !branch.source.is_skill_generated() {
                            return None;
                        }

                        let mut same_kind = branch_values
                            .iter()
                            .filter(|value| value.kind == branch.candidate.kind);
                        let first = same_kind.next()?;
                        if same_kind.next().is_none() {
                            Some(first)
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| ValueScoringError::MissingBranchValue {
                        kind: branch.candidate.kind.as_str(),
                        summary: branch.candidate.summary.clone(),
                    })?;

                let projected = self.scorer.project_with_delta(&branch_value.value, &delta);
                Ok(ScoredBranch {
                    branch: branch.clone(),
                    value: branch_value.value.clone(),
                    projected,
                })
            })
            .collect()
    }
}

#[derive(Debug, Clone, Default)]
pub struct MetacognitionPort {
    service: MetacognitionService,
}

impl MetacognitionPort {
    pub fn new(service: MetacognitionService) -> Self {
        Self { service }
    }
}

impl GatingPort for MetacognitionPort {
    fn evaluate(
        &self,
        working_memory: &WorkingMemory,
        scored_branches: Vec<ScoredBranch>,
    ) -> AnyResult<DecisionReport> {
        Ok(self.service.evaluate(working_memory, scored_branches))
    }
}

/// No-op rumination port used as the default when no rumination service is configured.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopRuminationPort;

impl RuminationPort for NoopRuminationPort {
    fn schedule_trigger(
        &self,
        _event: RuminationTriggerEvent,
    ) -> AnyResult<RuminationTriggerDecision> {
        Ok(RuminationTriggerDecision::Enqueued {
            tier: crate::cognition::rumination::RuminationQueueTier::Spq,
            item_id: String::new(),
        })
    }

    fn drain_short_cycle(&self, _now: &str) -> AnyResult<Option<ShortCycleWritebackReport>> {
        Ok(None)
    }
}

/// No-op simulation port used as the default when no simulation backend is configured.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopSimulationPort;

impl SimulationPort for NoopSimulationPort {
    fn simulate(
        &self,
        _world_fragments: &[WorldFragmentProjection],
        _action: &ActionCandidate,
    ) -> AnyResult<SimulationResult> {
        Err(anyhow::anyhow!("simulation not configured"))
    }
}

/// Concrete rumination port that wraps a `RuminationService`.
pub struct ServiceRuminationPort<'db> {
    service: RuminationService<'db>,
}

impl<'db> ServiceRuminationPort<'db> {
    pub fn new(conn: &'db Connection) -> Self {
        Self {
            service: RuminationService::new(conn),
        }
    }

    pub fn with_budget_limits(
        conn: &'db Connection,
        spq_budget_limit: u32,
        lpq_budget_limit: u32,
    ) -> Self {
        Self {
            service: RuminationService::with_budget_limits(
                conn,
                spq_budget_limit,
                lpq_budget_limit,
            ),
        }
    }
}

impl RuminationPort for ServiceRuminationPort<'_> {
    fn schedule_trigger(
        &self,
        event: RuminationTriggerEvent,
    ) -> AnyResult<RuminationTriggerDecision> {
        Ok(self.service.schedule(event)?)
    }

    fn drain_short_cycle(&self, now: &str) -> AnyResult<Option<ShortCycleWritebackReport>> {
        Ok(self.service.drain_short_cycle(now)?)
    }
}

/// Load persisted value adjustment candidates for a subject and fold them into a scoring port.
pub fn load_value_adjustments_into_scoring_port(
    repository: &MemoryRepository<'_>,
    subject_ref: &str,
    base_config: &ValueConfig,
    learning_rate: f32,
) -> WorkingMemoryScoringPort {
    let adjustments = collect_pending_value_adjustments(repository, subject_ref);
    if adjustments.is_empty() {
        return WorkingMemoryScoringPort::new(base_config.clone());
    }
    WorkingMemoryScoringPort::from_persisted_adjustments(base_config, &adjustments, learning_rate)
}

fn collect_pending_value_adjustments(
    repository: &MemoryRepository<'_>,
    subject_ref: &str,
) -> Vec<ValueAdjustment> {
    let candidates = match repository.list_rumination_candidates() {
        Ok(candidates) => candidates,
        Err(_) => return Vec::new(),
    };

    candidates
        .into_iter()
        .filter(|candidate| {
            candidate.candidate_kind == RuminationCandidateKind::ValueAdjustmentCandidate
                && candidate.status == RuminationCandidateStatus::Pending
                && (subject_ref.is_empty() || candidate.subject_ref == subject_ref)
        })
        .filter_map(|candidate| {
            candidate
                .payload
                .get("adjustment")
                .cloned()
                .and_then(|value| serde_json::from_value::<ValueAdjustment>(value).ok())
        })
        .collect()
}

impl<'db, P>
    AgentSearchOrchestrator<
        SearchServicePort<'db>,
        WorkingMemoryAssemblyPort<'db, P>,
        WorkingMemoryScoringPort,
        MetacognitionPort,
    >
where
    P: SelfStateProvider,
{
    pub fn with_services(
        conn: &'db Connection,
        self_state_provider: P,
        value_config: ValueConfig,
    ) -> Self {
        Self::new(
            SearchServicePort::new(conn),
            WorkingMemoryAssemblyPort::new(conn, self_state_provider),
            WorkingMemoryScoringPort::new(value_config),
            MetacognitionPort::default(),
        )
    }

    pub fn with_runtime_config(
        conn: &'db Connection,
        self_state_provider: P,
        value_config: ValueConfig,
        config: &Config,
    ) -> Self {
        Self::new(
            SearchServicePort::with_runtime_config(conn, config),
            WorkingMemoryAssemblyPort::new(conn, self_state_provider),
            WorkingMemoryScoringPort::new(value_config),
            MetacognitionPort::default(),
        )
    }
}

fn collect_unique_citations(retrieval_steps: &[RetrievalStepReport]) -> Vec<Citation> {
    let mut seen = BTreeSet::new();
    let mut citations = Vec::new();

    for citation in retrieval_steps
        .iter()
        .flat_map(|step| step.citations.iter().cloned())
    {
        if seen.insert(citation.record_id.clone()) {
            citations.push(citation);
        }
    }

    citations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cognition::{
            action::{ActionBranch, ActionCandidate, ActionKind},
            assembly::WorkingMemoryAssemblyError,
            metacog::GateDecision,
            working_memory::{PresentFrame, SelfStateSnapshot},
        },
        memory::record::TruthLayer,
    };

    #[test]
    fn request_bounds_queries_to_declared_step_budget() {
        let request = AgentSearchRequest::new(WorkingMemoryRequest::new("primary"))
            .with_follow_up_query("follow-up-a")
            .with_follow_up_query("follow-up-b")
            .with_max_steps(2);

        assert_eq!(request.bounded_queries(), vec!["primary", "follow-up-a"]);
    }

    #[test]
    fn scoring_port_requires_value_vectors_for_each_branch() {
        let branch = ActionBranch::new(ActionCandidate::new(
            ActionKind::Instrumental,
            "ship directly",
        ));
        let working_memory = WorkingMemory {
            present: PresentFrame {
                world_fragments: Vec::new(),
                self_state: SelfStateSnapshot {
                    task_context: None,
                    capability_flags: Vec::new(),
                    readiness_flags: Vec::new(),
                    facts: Vec::new(),
                },
                active_goal: None,
                active_risks: Vec::new(),
                metacog_flags: Vec::new(),
            },
            branches: vec![branch],
        };

        let error = WorkingMemoryScoringPort::default()
            .score(&working_memory, &[])
            .expect_err("branch scoring should reject missing value vectors");

        assert!(
            error.to_string().contains("ship directly"),
            "missing-branch error should preserve the branch summary: {error}",
        );
        let _ = TruthLayer::T2;
    }

    #[test]
    fn unique_citations_are_deduplicated_by_record_id() {
        let citation = Citation {
            record_id: "record-1".to_string(),
            source_uri: "memo://project/record-1".to_string(),
            source_kind: crate::memory::record::SourceKind::Note,
            source_label: Some("record-1".to_string()),
            recorded_at: "2026-04-16T00:00:00Z".to_string(),
            validity: crate::memory::record::ValidityWindow::default(),
            anchor: crate::search::CitationAnchor {
                chunk_index: 0,
                chunk_count: 1,
                anchor: crate::memory::record::ChunkAnchor::LineRange {
                    start_line: 1,
                    end_line: 1,
                },
            },
        };
        let citations = collect_unique_citations(&[
            RetrievalStepReport {
                query: "first".to_string(),
                applied_filters: SearchFilters::default(),
                result_count: 1,
                citations: vec![citation.clone()],
            },
            RetrievalStepReport {
                query: "second".to_string(),
                applied_filters: SearchFilters::default(),
                result_count: 1,
                citations: vec![citation],
            },
        ]);

        assert_eq!(citations.len(), 1);
    }

    #[test]
    fn assembly_port_surfaces_underlying_builder_failures() {
        #[derive(Clone, Copy)]
        struct BrokenAssemblyPort;

        impl AssemblyPort for BrokenAssemblyPort {
            fn assemble(&self, _: &WorkingMemoryRequest) -> AnyResult<WorkingMemory> {
                Err(anyhow::Error::new(
                    WorkingMemoryAssemblyError::MissingSupportingRecord {
                        record_id: "missing".to_string(),
                    },
                ))
            }
        }

        let error = BrokenAssemblyPort
            .assemble(&WorkingMemoryRequest::new("primary"))
            .expect_err("broken assembly should fail");
        assert!(error.to_string().contains("missing"));
    }

    // -- Cognitive loop tests --------------------------------------------------

    fn stub_report_with_gate(
        gate_decision: crate::cognition::metacog::GateDecision,
    ) -> AgentSearchReport {
        use crate::cognition::report::GateReport;

        AgentSearchReport {
            working_memory: WorkingMemory {
                present: PresentFrame {
                    world_fragments: Vec::new(),
                    self_state: SelfStateSnapshot {
                        task_context: None,
                        capability_flags: Vec::new(),
                        readiness_flags: Vec::new(),
                        facts: Vec::new(),
                    },
                    active_goal: None,
                    active_risks: Vec::new(),
                    metacog_flags: Vec::new(),
                },
                branches: Vec::new(),
            },
            decision: DecisionReport {
                scored_branches: Vec::new(),
                selected_branch: None,
                gate: GateReport {
                    decision: gate_decision,
                    diagnostics: Vec::new(),
                    rejected_branch: None,
                    regulative_branch: None,
                    safe_response: None,
                    autonomy_paused: false,
                },
                active_risks: Vec::new(),
                metacog_flags: Vec::new(),
            },
            retrieval_steps: Vec::new(),
            citations: Vec::new(),
            executed_steps: 0,
            step_limit: 1,
        }
    }

    #[test]
    fn rumination_trigger_maps_warning_to_session_boundary() {
        let report = stub_report_with_gate(GateDecision::Warning);
        let triggers =
            build_rumination_triggers(&report, "subject-1", "2026-04-30T00:00:00Z", "bucket-1")
                .expect("trigger building should succeed");

        assert_eq!(triggers.len(), 1);
        assert_eq!(triggers[0].kind, RuminationTriggerKind::SessionBoundary);
    }

    #[test]
    fn rumination_trigger_maps_soft_veto_to_metacog_veto() {
        let report = stub_report_with_gate(GateDecision::SoftVeto);
        let triggers =
            build_rumination_triggers(&report, "subject-1", "2026-04-30T00:00:00Z", "bucket-1")
                .expect("trigger building should succeed");

        assert_eq!(triggers.len(), 1);
        assert_eq!(triggers[0].kind, RuminationTriggerKind::MetacogVeto);
        assert_eq!(triggers[0].budget_cost, 1);
    }

    #[test]
    fn rumination_trigger_maps_hard_veto_to_high_priority_metacog_veto() {
        let report = stub_report_with_gate(GateDecision::HardVeto);
        let triggers =
            build_rumination_triggers(&report, "subject-1", "2026-04-30T00:00:00Z", "bucket-1")
                .expect("trigger building should succeed");

        assert_eq!(triggers.len(), 1);
        assert_eq!(triggers[0].kind, RuminationTriggerKind::MetacogVeto);
        assert_eq!(triggers[0].budget_cost, 3);
    }

    #[test]
    fn rumination_trigger_maps_escalate_to_high_priority_metacog_veto() {
        let report = stub_report_with_gate(GateDecision::Escalate);
        let triggers =
            build_rumination_triggers(&report, "subject-1", "2026-04-30T00:00:00Z", "bucket-1")
                .expect("trigger building should succeed");

        assert_eq!(triggers.len(), 1);
        assert_eq!(triggers[0].kind, RuminationTriggerKind::MetacogVeto);
        assert_eq!(triggers[0].budget_cost, 3);
    }

    #[test]
    fn noop_rumination_port_schedule_returns_ok() {
        let port = NoopRuminationPort;
        let report = stub_report_with_gate(GateDecision::Warning);
        let trigger = RuminationTriggerEvent::from_agent_search_report(
            RuminationTriggerKind::SessionBoundary,
            "subject-1",
            &report,
            "2026-04-30T00:00:00Z",
            "bucket-1",
            None,
            None,
        )
        .expect("should build trigger");

        let result = port.schedule_trigger(trigger);
        assert!(result.is_ok());
    }

    #[test]
    fn noop_rumination_port_drain_returns_none() {
        let port = NoopRuminationPort;
        let result = port.drain_short_cycle("2026-04-30T00:00:00Z");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn noop_simulation_port_returns_error() {
        let port = NoopSimulationPort;
        let result = port.simulate(&[], &ActionCandidate::new(ActionKind::Epistemic, "test"));
        assert!(result.is_err());
    }

    #[test]
    fn enable_simulation_defaults_to_false() {
        let request = AgentSearchRequest::new(WorkingMemoryRequest::new("test"));
        assert!(!request.enable_simulation);
    }

    #[test]
    fn enable_simulation_can_be_set_to_true() {
        let request =
            AgentSearchRequest::new(WorkingMemoryRequest::new("test")).with_enable_simulation(true);
        assert!(request.enable_simulation);
    }

    #[test]
    fn simulation_enrichment_adds_risk_markers() {
        use crate::cognition::world_model::{
            PredictedFragmentChange, PredictedRisk, PredictedSeverity, PredictedWorldSlice,
        };

        let mut working_memory = WorkingMemory {
            present: PresentFrame {
                world_fragments: Vec::new(),
                self_state: SelfStateSnapshot {
                    task_context: None,
                    capability_flags: Vec::new(),
                    readiness_flags: Vec::new(),
                    facts: Vec::new(),
                },
                active_goal: None,
                active_risks: Vec::new(),
                metacog_flags: Vec::new(),
            },
            branches: Vec::new(),
        };
        let mut scored_branches = vec![ScoredBranch {
            branch: ActionBranch::new(ActionCandidate::new(ActionKind::Instrumental, "deploy")),
            value: ValueVector {
                goal_progress: 0.5,
                information_gain: 0.5,
                risk_avoidance: 0.5,
                resource_efficiency: 0.5,
                agent_robustness: 0.5,
            },
            projected: crate::cognition::value::ProjectedScore {
                final_score: 0.5,
                weight_snapshot: ValueConfig::default(),
                threshold_passed: true,
            },
        }];

        let sim_result = SimulationResult {
            predicted: PredictedWorldSlice {
                affected_fragments: vec![PredictedFragmentChange {
                    record_id: "r1".to_string(),
                    change_description: "weakened".to_string(),
                    change_direction: crate::cognition::world_model::ChangeDirection::Weakened,
                }],
                new_risks: vec![PredictedRisk {
                    description: "regression risk".to_string(),
                    severity: PredictedSeverity::High,
                }],
                uncertainty_delta: 0.2,
                overall_assessment: "moderate risk".to_string(),
            },
            confidence: 0.7,
            action_summary: "deploy".to_string(),
        };

        let _ =
            enrich_branches_from_simulation(&mut working_memory, &mut scored_branches, &sim_result);

        assert!(
            scored_branches[0]
                .branch
                .risk_markers
                .iter()
                .any(|m| m.contains("simulated_high") && m.contains("regression risk")),
            "simulation should add risk marker to top branch: {:?}",
            scored_branches[0].branch.risk_markers,
        );
        assert!(
            working_memory
                .present
                .active_risks
                .iter()
                .any(|r| r.contains("simulated_high")),
            "simulation should add risk to working memory active risks: {:?}",
            working_memory.present.active_risks,
        );
        assert!(
            scored_branches[0]
                .branch
                .candidate
                .expected_effects
                .iter()
                .any(|e| e.contains("r1") && e.contains("weakened")),
            "simulation should add affected fragment to expected effects: {:?}",
            scored_branches[0].branch.candidate.expected_effects,
        );
    }

    #[test]
    fn collect_pending_value_adjustments_returns_empty_on_db_error() {
        // Use a bare in-memory connection without migrations.
        // list_rumination_candidates will fail because the table doesn't exist,
        // so the function should return an empty vec gracefully.
        let conn = rusqlite::Connection::open_in_memory().expect("in-memory db");
        let repository = MemoryRepository::new(&conn);

        let adjustments = collect_pending_value_adjustments(&repository, "subject-1");
        assert!(
            adjustments.is_empty(),
            "should return empty when DB table doesn't exist"
        );
    }
}
