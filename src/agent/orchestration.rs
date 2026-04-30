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
        value::{ScoredBranch, ValueAdjustment, ValueConfig, ValueScorer, ValueVector},
        working_memory::WorkingMemory,
    },
    core::config::Config,
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
}

pub struct AgentSearchOrchestrator<R, A, S, G> {
    retriever: R,
    assembler: A,
    scorer: S,
    gate: G,
}

impl<R, A, S, G> AgentSearchOrchestrator<R, A, S, G> {
    pub fn new(retriever: R, assembler: A, scorer: S, gate: G) -> Self {
        Self {
            retriever,
            assembler,
            scorer,
            gate,
        }
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
        let working_memory = self
            .assembler
            .assemble(&working_memory_request)
            .map_err(|source| AgentSearchError::Assembly { source })?;
        let scored_branches = self
            .scorer
            .score(&working_memory, &request.branch_values)
            .map_err(|source| AgentSearchError::Scoring { source })?;
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

                Ok(self
                    .scorer
                    .score_branch(crate::cognition::value::BranchValueInput::new(
                        branch.clone(),
                        branch_value.value.clone(),
                    )))
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
}
