use std::future::Future;
use std::pin::Pin;

use rig::{client::CompletionClient, completion::TypedPrompt, providers::openai};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    cognition::action::ActionCandidate,
    cognition::working_memory::{EvidenceFragment, TruthContext},
    core::config::RootLlmConfig,
    memory::{
        dsl::FlatFactDslRecordV1,
        record::Provenance,
        repository::{
            MemoryRepository, PersistedWorldModelAppliedFilters,
            PersistedWorldModelChannelContribution, PersistedWorldModelCitation,
            PersistedWorldModelCitationAnchor, PersistedWorldModelQueryStrategy,
            PersistedWorldModelScore, PersistedWorldModelTrace, PersistedWorldModelTruthContext,
            RepositoryError,
        },
        truth::TruthRecord,
    },
    search::{
        AppliedFilters, ChannelContribution, Citation, CitationAnchor, QueryStrategy, ResultTrace,
        ScoreBreakdown, SearchFilters, SearchResult,
    },
};

pub use crate::memory::repository::{
    PersistedWorldModelSnapshot as WorldModelSnapshot,
    PersistedWorldModelSnapshotFragment as WorldModelSnapshotFragment,
};

pub const CURRENT_WORLD_KEY: &str = "current";

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ProjectedWorldModel {
    pub current: CurrentWorldSlice,
}

impl ProjectedWorldModel {
    pub fn new(current: CurrentWorldSlice) -> Self {
        Self { current }
    }

    pub fn from_snapshot(snapshot: &WorldModelSnapshot) -> Self {
        Self::new(CurrentWorldSlice::new(
            snapshot
                .fragments
                .iter()
                .map(WorldFragmentProjection::from_snapshot_fragment)
                .collect(),
        ))
    }

    pub fn project_fragments(&self) -> Vec<EvidenceFragment> {
        self.current
            .fragments
            .iter()
            .map(WorldFragmentProjection::project_fragment)
            .collect()
    }

    pub fn to_snapshot(
        &self,
        subject_ref: impl Into<String>,
        world_key: impl Into<String>,
        snapshot_id: impl Into<String>,
        created_at: impl Into<String>,
        updated_at: impl Into<String>,
    ) -> WorldModelSnapshot {
        WorldModelSnapshot {
            subject_ref: subject_ref.into(),
            world_key: world_key.into(),
            snapshot_id: snapshot_id.into(),
            fragments: self
                .current
                .fragments
                .iter()
                .map(WorldFragmentProjection::to_snapshot_fragment)
                .collect(),
            created_at: created_at.into(),
            updated_at: updated_at.into(),
        }
    }
}

pub fn load_runtime_current_world_model(
    repository: &MemoryRepository<'_>,
    subject_ref: &str,
) -> Result<Option<ProjectedWorldModel>, RepositoryError> {
    repository
        .load_world_model_snapshot(subject_ref, CURRENT_WORLD_KEY)
        .map(|snapshot| snapshot.map(|snapshot| ProjectedWorldModel::from_snapshot(&snapshot)))
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct CurrentWorldSlice {
    pub fragments: Vec<WorldFragmentProjection>,
}

impl CurrentWorldSlice {
    pub fn new(fragments: Vec<WorldFragmentProjection>) -> Self {
        Self { fragments }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WorldFragmentProjection {
    pub record_id: String,
    pub snippet: String,
    pub citation: Citation,
    pub provenance: Provenance,
    pub truth_context: TruthContext,
    pub dsl: Option<FlatFactDslRecordV1>,
    pub trace: ResultTrace,
    pub score: ScoreBreakdown,
}

impl WorldFragmentProjection {
    pub fn from_search_result(
        result: SearchResult,
        truth: &TruthRecord,
        repository_dsl: Option<&FlatFactDslRecordV1>,
    ) -> Self {
        let SearchResult {
            record,
            snippet,
            citation,
            dsl,
            score,
            trace,
        } = result;

        Self {
            record_id: record.id,
            snippet,
            citation,
            provenance: record.provenance,
            truth_context: TruthContext::from_truth_record(truth),
            dsl: dsl.or_else(|| repository_dsl.cloned()),
            trace,
            score,
        }
    }

    pub fn project_fragment(&self) -> EvidenceFragment {
        EvidenceFragment {
            record_id: self.record_id.clone(),
            snippet: self.snippet.clone(),
            citation: self.citation.clone(),
            provenance: self.provenance.clone(),
            truth_context: self.truth_context.clone(),
            dsl: self.dsl.clone(),
            trace: self.trace.clone(),
            score: self.score.clone(),
        }
    }

    pub fn from_snapshot_fragment(fragment: &WorldModelSnapshotFragment) -> Self {
        Self {
            record_id: fragment.record_id.clone(),
            snippet: fragment.snippet.clone(),
            citation: restore_citation(&fragment.citation),
            provenance: fragment.provenance.clone(),
            truth_context: restore_truth_context(&fragment.truth_context),
            dsl: fragment.dsl.clone(),
            trace: restore_trace(&fragment.trace),
            score: restore_score(&fragment.score),
        }
    }

    pub fn to_snapshot_fragment(&self) -> WorldModelSnapshotFragment {
        WorldModelSnapshotFragment {
            record_id: self.record_id.clone(),
            snippet: self.snippet.clone(),
            citation: persist_citation(&self.citation),
            provenance: self.provenance.clone(),
            truth_context: persist_truth_context(&self.truth_context),
            dsl: self.dsl.clone(),
            trace: persist_trace(&self.trace),
            score: persist_score(&self.score),
        }
    }
}

fn persist_citation(citation: &Citation) -> PersistedWorldModelCitation {
    PersistedWorldModelCitation {
        record_id: citation.record_id.clone(),
        source_uri: citation.source_uri.clone(),
        source_kind: citation.source_kind,
        source_label: citation.source_label.clone(),
        recorded_at: citation.recorded_at.clone(),
        validity: citation.validity.clone(),
        anchor: PersistedWorldModelCitationAnchor {
            chunk_index: citation.anchor.chunk_index,
            chunk_count: citation.anchor.chunk_count,
            anchor: citation.anchor.anchor.clone(),
        },
    }
}

fn restore_citation(citation: &PersistedWorldModelCitation) -> Citation {
    Citation {
        record_id: citation.record_id.clone(),
        source_uri: citation.source_uri.clone(),
        source_kind: citation.source_kind,
        source_label: citation.source_label.clone(),
        recorded_at: citation.recorded_at.clone(),
        validity: citation.validity.clone(),
        anchor: CitationAnchor {
            chunk_index: citation.anchor.chunk_index,
            chunk_count: citation.anchor.chunk_count,
            anchor: citation.anchor.anchor.clone(),
        },
    }
}

fn persist_truth_context(truth_context: &TruthContext) -> PersistedWorldModelTruthContext {
    PersistedWorldModelTruthContext {
        truth_layer: truth_context.truth_layer,
        t3_state: truth_context.t3_state.clone(),
        open_review_ids: truth_context.open_review_ids.clone(),
        open_candidate_ids: truth_context.open_candidate_ids.clone(),
    }
}

fn restore_truth_context(truth_context: &PersistedWorldModelTruthContext) -> TruthContext {
    TruthContext {
        truth_layer: truth_context.truth_layer,
        t3_state: truth_context.t3_state.clone(),
        open_review_ids: truth_context.open_review_ids.clone(),
        open_candidate_ids: truth_context.open_candidate_ids.clone(),
    }
}

fn persist_trace(trace: &ResultTrace) -> PersistedWorldModelTrace {
    PersistedWorldModelTrace {
        matched_query: trace.matched_query.clone(),
        query_strategies: trace
            .query_strategies
            .iter()
            .copied()
            .map(persist_query_strategy)
            .collect(),
        channel_contribution: persist_channel_contribution(trace.channel_contribution),
        applied_filters: persist_applied_filters(&trace.applied_filters),
    }
}

fn restore_trace(trace: &PersistedWorldModelTrace) -> ResultTrace {
    ResultTrace {
        matched_query: trace.matched_query.clone(),
        query_strategies: trace
            .query_strategies
            .iter()
            .copied()
            .map(restore_query_strategy)
            .collect(),
        channel_contribution: restore_channel_contribution(trace.channel_contribution),
        applied_filters: restore_applied_filters(&trace.applied_filters),
        attention: None,
    }
}

fn persist_query_strategy(strategy: QueryStrategy) -> PersistedWorldModelQueryStrategy {
    match strategy {
        QueryStrategy::Jieba => PersistedWorldModelQueryStrategy::Jieba,
        QueryStrategy::Simple => PersistedWorldModelQueryStrategy::Simple,
        QueryStrategy::Structured => PersistedWorldModelQueryStrategy::Structured,
        QueryStrategy::Embedding => PersistedWorldModelQueryStrategy::Embedding,
    }
}

fn restore_query_strategy(strategy: PersistedWorldModelQueryStrategy) -> QueryStrategy {
    match strategy {
        PersistedWorldModelQueryStrategy::Jieba => QueryStrategy::Jieba,
        PersistedWorldModelQueryStrategy::Simple => QueryStrategy::Simple,
        PersistedWorldModelQueryStrategy::Structured => QueryStrategy::Structured,
        PersistedWorldModelQueryStrategy::Embedding => QueryStrategy::Embedding,
    }
}

fn persist_channel_contribution(
    contribution: ChannelContribution,
) -> PersistedWorldModelChannelContribution {
    match contribution {
        ChannelContribution::LexicalOnly => PersistedWorldModelChannelContribution::LexicalOnly,
        ChannelContribution::EmbeddingOnly => PersistedWorldModelChannelContribution::EmbeddingOnly,
        ChannelContribution::Hybrid => PersistedWorldModelChannelContribution::Hybrid,
    }
}

fn restore_channel_contribution(
    contribution: PersistedWorldModelChannelContribution,
) -> ChannelContribution {
    match contribution {
        PersistedWorldModelChannelContribution::LexicalOnly => ChannelContribution::LexicalOnly,
        PersistedWorldModelChannelContribution::EmbeddingOnly => ChannelContribution::EmbeddingOnly,
        PersistedWorldModelChannelContribution::Hybrid => ChannelContribution::Hybrid,
    }
}

fn persist_applied_filters(filters: &AppliedFilters) -> PersistedWorldModelAppliedFilters {
    PersistedWorldModelAppliedFilters {
        scope: filters.scope,
        record_type: filters.record_type,
        truth_layer: filters.truth_layer,
        domain: filters.domain.clone(),
        topic: filters.topic.clone(),
        aspect: filters.aspect.clone(),
        kind: filters.kind.clone(),
        valid_at: filters.valid_at.clone(),
        recorded_from: filters.recorded_from.clone(),
        recorded_to: filters.recorded_to.clone(),
    }
}

fn restore_applied_filters(filters: &PersistedWorldModelAppliedFilters) -> AppliedFilters {
    SearchFilters {
        scope: filters.scope,
        record_type: filters.record_type,
        truth_layer: filters.truth_layer,
        domain: filters.domain.clone(),
        topic: filters.topic.clone(),
        aspect: filters.aspect.clone(),
        kind: filters.kind.clone(),
        valid_at: filters.valid_at.clone(),
        recorded_from: filters.recorded_from.clone(),
        recorded_to: filters.recorded_to.clone(),
    }
}

fn persist_score(score: &ScoreBreakdown) -> PersistedWorldModelScore {
    PersistedWorldModelScore {
        lexical_raw: score.lexical_raw,
        lexical_base: score.lexical_base,
        keyword_bonus: score.keyword_bonus,
        importance_bonus: score.importance_bonus,
        recency_bonus: score.recency_bonus,
        attention_bonus: score.attention_bonus,
        final_score: score.final_score,
    }
}

fn restore_score(score: &PersistedWorldModelScore) -> ScoreBreakdown {
    ScoreBreakdown {
        lexical_raw: score.lexical_raw,
        lexical_base: score.lexical_base,
        keyword_bonus: score.keyword_bonus,
        importance_bonus: score.importance_bonus,
        recency_bonus: score.recency_bonus,
        attention_bonus: score.attention_bonus,
        final_score: score.final_score,
    }
}

// ---------------------------------------------------------------------------
// World model prediction / simulation
// ---------------------------------------------------------------------------

/// Direction of a predicted change to a world fragment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChangeDirection {
    Strengthened,
    Weakened,
    Invalidated,
    Unchanged,
    NewRisk,
}

/// A single predicted change to a world fragment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PredictedFragmentChange {
    pub record_id: String,
    pub change_description: String,
    pub change_direction: ChangeDirection,
}

/// Severity level for a predicted risk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PredictedSeverity {
    Low,
    Medium,
    High,
}

/// A predicted risk emerging from the simulation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PredictedRisk {
    pub description: String,
    pub severity: PredictedSeverity,
}

/// The result of simulating an action against the current world state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredictedWorldSlice {
    pub affected_fragments: Vec<PredictedFragmentChange>,
    pub new_risks: Vec<PredictedRisk>,
    /// Delta in uncertainty: negative means less uncertain, positive means more.
    pub uncertainty_delta: f32,
    pub overall_assessment: String,
}

/// Complete simulation result combining the predicted slice with confidence metadata.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SimulationResult {
    pub predicted: PredictedWorldSlice,
    /// Confidence of the prediction, between 0.0 and 1.0.
    pub confidence: f32,
    pub action_summary: String,
}

#[derive(Debug, Error)]
pub enum SimulationError {
    #[error("{0}")]
    LlmUnconfigured(String),
    #[error("{0}")]
    LlmRequestFailed(String),
}

/// Structured output schema for the LLM simulation response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SimulationStructuredOutput {
    pub affected_fragments: Vec<PredictedFragmentChange>,
    pub new_risks: Vec<PredictedRisk>,
    pub uncertainty_delta: f32,
    pub overall_assessment: String,
    pub confidence: f32,
}

impl SimulationStructuredOutput {
    pub fn into_simulation_result(self, action_summary: String) -> SimulationResult {
        SimulationResult {
            predicted: PredictedWorldSlice {
                affected_fragments: self.affected_fragments,
                new_risks: self.new_risks,
                uncertainty_delta: self.uncertainty_delta,
                overall_assessment: self.overall_assessment,
            },
            confidence: self.confidence,
            action_summary,
        }
    }
}

const SIMULATION_PREAMBLE: &str = concat!(
    "You predict the consequences of an action on a world model. ",
    "Given current world fragments and an action description, predict which fragments ",
    "are affected, what new risks emerge, and how uncertainty changes. ",
    "Be concise and factual. Do not invent information. ",
    "Return structured JSON only."
);

/// Trait abstracting the LLM backend for world simulation.
pub trait RigWorldSimulationBackend: Send + Sync {
    fn complete<'a>(
        &'a self,
        prompt: String,
    ) -> Pin<
        Box<dyn Future<Output = Result<SimulationStructuredOutput, SimulationError>> + Send + 'a>,
    >;
}

/// OpenAI-compatible LLM backend for world simulation (same pattern as RigSummaryBackend).
#[derive(Debug, Clone)]
pub struct OpenAiCompatibleSimulationBackend {
    config: RootLlmConfig,
}

impl OpenAiCompatibleSimulationBackend {
    pub fn new(config: RootLlmConfig) -> Self {
        Self { config }
    }
}

impl RigWorldSimulationBackend for OpenAiCompatibleSimulationBackend {
    fn complete<'a>(
        &'a self,
        prompt: String,
    ) -> Pin<
        Box<dyn Future<Output = Result<SimulationStructuredOutput, SimulationError>> + Send + 'a>,
    > {
        Box::pin(async move {
            if !self.config.is_configured() {
                return Err(SimulationError::LlmUnconfigured(
                    "llm config is incomplete for world simulation".to_string(),
                ));
            }

            if self.config.provider != "openai" {
                return Err(SimulationError::LlmUnconfigured(format!(
                    "unsupported world simulation provider: {}",
                    self.config.provider
                )));
            }

            let api_key = self.config.api_key.as_deref().ok_or_else(|| {
                SimulationError::LlmUnconfigured("missing api_key for world simulation".to_string())
            })?;

            let mut builder = openai::Client::builder().api_key(api_key);
            if let Some(api_base) = self.config.api_base.as_deref() {
                builder = builder.base_url(api_base);
            }

            let client = builder.build().map_err(|error| {
                SimulationError::LlmRequestFailed(format!(
                    "failed to build openai-compatible rig client: {error}"
                ))
            })?;

            let mut agent = client
                .agent(self.config.model.clone())
                .preamble(SIMULATION_PREAMBLE);
            if let Some(temperature) = self.config.temperature {
                agent = agent.temperature(f64::from(temperature));
            }
            if let Some(max_tokens) = self.config.max_tokens {
                agent = agent.max_tokens(u64::from(max_tokens));
            }

            agent
                .build()
                .prompt_typed::<SimulationStructuredOutput>(prompt)
                .await
                .map_err(|error| {
                    SimulationError::LlmRequestFailed(format!(
                        "rig world simulation request failed: {error}"
                    ))
                })
        })
    }
}

/// High-level simulator that uses an LLM backend to predict action consequences.
#[derive(Debug, Clone)]
pub struct WorldSimulator<B> {
    backend: B,
}

impl<B> WorldSimulator<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }
}

impl WorldSimulator<OpenAiCompatibleSimulationBackend> {
    pub fn from_llm_config(config: RootLlmConfig) -> Self {
        Self::new(OpenAiCompatibleSimulationBackend::new(config))
    }
}

impl<B> WorldSimulator<B>
where
    B: RigWorldSimulationBackend,
{
    /// Run simulation asynchronously.
    pub async fn simulate_async(
        &self,
        world_fragments: &[WorldFragmentProjection],
        action: &ActionCandidate,
    ) -> Result<SimulationResult, SimulationError> {
        let prompt = build_simulation_prompt(world_fragments, action);
        let action_summary = action.summary.clone();
        let output = self.backend.complete(prompt).await?;
        Ok(output.into_simulation_result(action_summary))
    }

    /// Run simulation synchronously using tokio runtime.
    pub fn simulate(
        &self,
        world_fragments: &[WorldFragmentProjection],
        action: &ActionCandidate,
    ) -> Result<SimulationResult, SimulationError> {
        let handle = tokio::runtime::Handle::current();
        handle.block_on(self.simulate_async(world_fragments, action))
    }
}

/// Build the prompt sent to the LLM for world simulation.
pub fn build_simulation_prompt(
    world_fragments: &[WorldFragmentProjection],
    action: &ActionCandidate,
) -> String {
    let mut fragments_section = String::new();
    for (i, fragment) in world_fragments.iter().enumerate() {
        fragments_section.push_str(&format!(
            "  [{}] record_id: {}\n       snippet: {}\n",
            i + 1,
            fragment.record_id,
            fragment.snippet,
        ));
        if let Some(ref dsl) = fragment.dsl {
            fragments_section.push_str(&format!("       dsl_claim: {}\n", dsl.claim));
        }
    }

    let expected_effects = if action.expected_effects.is_empty() {
        "  (none specified)".to_string()
    } else {
        action
            .expected_effects
            .iter()
            .enumerate()
            .map(|(i, effect)| format!("  {}. {effect}", i + 1))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let intent = action.intent.as_deref().unwrap_or("(not specified)");

    format!(
        concat!(
            "Predict the consequences of the following action on the current world state.\n",
            "\n",
            "## Current World Fragments\n",
            "{fragments}\n",
            "## Action\n",
            "kind: {kind}\n",
            "summary: {summary}\n",
            "intent: {intent}\n",
            "expected_effects:\n{expected_effects}\n",
            "\n",
            "For each world fragment that would be affected, describe the change and direction.\n",
            "Identify any new risks that could emerge.\n",
            "Estimate the uncertainty delta (negative = less uncertain, positive = more).\n",
            "Provide an overall assessment and your confidence (0.0 to 1.0).\n"
        ),
        fragments = fragments_section,
        kind = action.kind.as_str(),
        summary = action.summary,
        intent = intent,
        expected_effects = expected_effects,
    )
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::cognition::action::ActionKind;
    use crate::memory::dsl::FlatFactDslRecordV1;
    use crate::memory::record::{ChunkAnchor, Provenance, SourceKind, TruthLayer, ValidityWindow};
    use crate::search::{Citation, CitationAnchor, ScoreBreakdown};

    fn sample_fragment(record_id: &str, snippet: &str) -> WorldFragmentProjection {
        WorldFragmentProjection {
            record_id: record_id.to_string(),
            snippet: snippet.to_string(),
            citation: Citation {
                record_id: record_id.to_string(),
                source_uri: format!("memo://test/{record_id}"),
                source_kind: SourceKind::Note,
                source_label: Some("test".to_string()),
                recorded_at: "2026-04-30T00:00:00Z".to_string(),
                validity: ValidityWindow {
                    valid_from: Some("2026-04-30T00:00:00Z".to_string()),
                    valid_to: None,
                },
                anchor: CitationAnchor {
                    chunk_index: 0,
                    chunk_count: 1,
                    anchor: ChunkAnchor::LineRange {
                        start_line: 1,
                        end_line: 3,
                    },
                },
            },
            provenance: Provenance {
                origin: "test".to_string(),
                imported_via: None,
                derived_from: Vec::new(),
            },
            truth_context: crate::cognition::working_memory::TruthContext {
                truth_layer: TruthLayer::T2,
                t3_state: None,
                open_review_ids: Vec::new(),
                open_candidate_ids: Vec::new(),
            },
            dsl: None,
            trace: crate::search::ResultTrace {
                matched_query: "test".to_string(),
                query_strategies: Vec::new(),
                channel_contribution: ChannelContribution::LexicalOnly,
                applied_filters: AppliedFilters::default(),
                attention: None,
            },
            score: ScoreBreakdown {
                lexical_raw: 0.0,
                lexical_base: 0.0,
                keyword_bonus: 0.0,
                importance_bonus: 0.0,
                recency_bonus: 0.0,
                attention_bonus: 0.0,
                final_score: 0.0,
            },
        }
    }

    fn sample_fragment_with_dsl(
        record_id: &str,
        snippet: &str,
        claim: &str,
    ) -> WorldFragmentProjection {
        let mut fragment = sample_fragment(record_id, snippet);
        fragment.dsl = Some(FlatFactDslRecordV1 {
            domain: "project".to_string(),
            topic: "world_model".to_string(),
            aspect: "state".to_string(),
            kind: "decision".to_string(),
            claim: claim.to_string(),
            truth_layer: "t2".to_string(),
            source_ref: format!("memo://test/{record_id}"),
            why: None,
            time: None,
            cond: None,
            impact: None,
            conf: None,
            rel: None,
        });
        fragment
    }

    #[test]
    fn simulation_prompt_includes_world_fragments_and_action() {
        let fragments = vec![
            sample_fragment_with_dsl(
                "r1",
                "baseline is lexical search",
                "lexical search is stable",
            ),
            sample_fragment("r2", "embedding index is empty"),
        ];
        let action = ActionCandidate::new(ActionKind::Epistemic, "switch to embedding search")
            .with_intent("improve recall quality for semantic queries");

        let prompt = build_simulation_prompt(&fragments, &action);

        assert!(prompt.contains("record_id: r1"));
        assert!(prompt.contains("snippet: baseline is lexical search"));
        assert!(prompt.contains("dsl_claim: lexical search is stable"));
        assert!(prompt.contains("record_id: r2"));
        assert!(prompt.contains("snippet: embedding index is empty"));
        assert!(prompt.contains("kind: epistemic"));
        assert!(prompt.contains("summary: switch to embedding search"));
        assert!(prompt.contains("intent: improve recall quality for semantic queries"));
        assert!(prompt.contains("uncertainty delta"));
    }

    #[test]
    fn simulation_prompt_handles_empty_expected_effects() {
        let fragments = vec![sample_fragment("r1", "some fact")];
        let action = ActionCandidate::new(ActionKind::Instrumental, "deploy new index");

        let prompt = build_simulation_prompt(&fragments, &action);

        assert!(prompt.contains("(none specified)"));
    }

    #[test]
    fn simulation_prompt_lists_expected_effects() {
        let fragments = vec![sample_fragment("r1", "some fact")];
        let mut action =
            ActionCandidate::new(ActionKind::Regulative, "adjust retrieval parameters");
        action.expected_effects.push("faster retrieval".to_string());
        action.expected_effects.push("slower indexing".to_string());

        let prompt = build_simulation_prompt(&fragments, &action);

        assert!(prompt.contains("1. faster retrieval"));
        assert!(prompt.contains("2. slower indexing"));
    }

    #[test]
    fn simulation_prompt_uses_not_specified_when_intent_is_missing() {
        let fragments = vec![sample_fragment("r1", "fragment")];
        let action = ActionCandidate::new(ActionKind::Epistemic, "observe");

        let prompt = build_simulation_prompt(&fragments, &action);

        assert!(prompt.contains("intent: (not specified)"));
    }

    #[derive(Debug, Clone)]
    struct StubSimulationBackend {
        response: SimulationStructuredOutput,
        prompt: Arc<Mutex<Option<String>>>,
    }

    impl RigWorldSimulationBackend for StubSimulationBackend {
        fn complete<'a>(
            &'a self,
            prompt: String,
        ) -> Pin<
            Box<
                dyn Future<Output = Result<SimulationStructuredOutput, SimulationError>>
                    + Send
                    + 'a,
            >,
        > {
            let slot = Arc::clone(&self.prompt);
            let response = self.response.clone();
            Box::pin(async move {
                *slot.lock().expect("prompt slot should lock") = Some(prompt);
                Ok(response)
            })
        }
    }

    fn stub_output() -> SimulationStructuredOutput {
        SimulationStructuredOutput {
            affected_fragments: vec![PredictedFragmentChange {
                record_id: "r1".to_string(),
                change_description: "lexical baseline may be replaced".to_string(),
                change_direction: ChangeDirection::Weakened,
            }],
            new_risks: vec![PredictedRisk {
                description: "recall regression during transition".to_string(),
                severity: PredictedSeverity::Medium,
            }],
            uncertainty_delta: 0.2,
            overall_assessment: "moderate risk of transition instability".to_string(),
            confidence: 0.65,
        }
    }

    #[tokio::test]
    async fn world_simulator_returns_structured_result() {
        let prompt = Arc::new(Mutex::new(None));
        let output = stub_output();
        let expected_action_summary = "switch to hybrid search".to_string();
        let simulator = WorldSimulator::new(StubSimulationBackend {
            response: output.clone(),
            prompt: Arc::clone(&prompt),
        });

        let fragments = vec![sample_fragment("r1", "lexical baseline is active")];
        let action = ActionCandidate::new(ActionKind::Epistemic, &expected_action_summary);

        let result = simulator
            .simulate_async(&fragments, &action)
            .await
            .expect("simulation should succeed");

        assert_eq!(result.action_summary, expected_action_summary);
        assert_eq!(result.confidence, 0.65);
        assert_eq!(result.predicted.affected_fragments.len(), 1);
        assert_eq!(result.predicted.affected_fragments[0].record_id, "r1");
        assert_eq!(
            result.predicted.affected_fragments[0].change_direction,
            ChangeDirection::Weakened
        );
        assert_eq!(result.predicted.new_risks.len(), 1);
        assert_eq!(
            result.predicted.new_risks[0].severity,
            PredictedSeverity::Medium
        );
        assert!((result.predicted.uncertainty_delta - 0.2).abs() < f32::EPSILON);

        let captured_prompt = prompt
            .lock()
            .expect("prompt slot should lock")
            .clone()
            .expect("prompt should have been captured");
        assert!(captured_prompt.contains("record_id: r1"));
        assert!(captured_prompt.contains(&expected_action_summary));
    }

    #[test]
    fn simulation_structured_output_converts_to_simulation_result() {
        let output = stub_output();
        let action_summary = "test action".to_string();
        let result = output.into_simulation_result(action_summary.clone());

        assert_eq!(result.action_summary, action_summary);
        assert_eq!(result.confidence, 0.65);
        assert_eq!(
            result.predicted.overall_assessment,
            "moderate risk of transition instability"
        );
        assert_eq!(result.predicted.affected_fragments.len(), 1);
        assert_eq!(result.predicted.new_risks.len(), 1);
    }

    #[test]
    fn predicted_world_slice_serialization_round_trip() {
        let slice = PredictedWorldSlice {
            affected_fragments: vec![PredictedFragmentChange {
                record_id: "r1".to_string(),
                change_description: "weakened".to_string(),
                change_direction: ChangeDirection::Weakened,
            }],
            new_risks: vec![PredictedRisk {
                description: "risk".to_string(),
                severity: PredictedSeverity::High,
            }],
            uncertainty_delta: -0.3,
            overall_assessment: "less uncertain".to_string(),
        };

        let json = serde_json::to_string(&slice).expect("should serialize");
        let restored: PredictedWorldSlice =
            serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(restored, slice);
    }
}
