use serde::Serialize;

use crate::{
    cognition::working_memory::{EvidenceFragment, TruthContext},
    memory::{
        dsl::FlatFactDslRecordV1,
        record::Provenance,
        repository::{
            PersistedWorldModelAppliedFilters, PersistedWorldModelChannelContribution,
            PersistedWorldModelCitation, PersistedWorldModelCitationAnchor,
            PersistedWorldModelQueryStrategy, PersistedWorldModelScore, PersistedWorldModelTrace,
            PersistedWorldModelTruthContext,
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
        emotion_bonus: score.emotion_bonus,
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
        emotion_bonus: score.emotion_bonus,
        final_score: score.final_score,
    }
}
