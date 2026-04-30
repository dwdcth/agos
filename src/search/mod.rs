pub mod citation;
pub mod filter;
pub mod lexical;
pub mod rerank;
pub mod score;

use rusqlite::Connection;
use serde::Serialize;
use std::collections::BTreeSet;
use thiserror::Error;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::{
    core::config::{Config, EmbeddingBackend, RetrievalMode, RetrievalModeVariant, VectorBackend},
    memory::{
        dsl::FlatFactDslRecordV1,
        record::MemoryRecord,
        repository::{MemoryRepository, RepositoryError},
    },
};

pub use citation::{Citation, CitationAnchor, CitationError};
pub use filter::{AppliedFilters, SearchFilters};
pub use lexical::{LexicalCandidate, LexicalSearchError, QueryStrategy};
pub use rerank::{ChannelContribution, ResultTrace};
pub use score::ScoreBreakdown;

use crate::cognition::attention::AttentionState;

#[derive(Debug, Clone, PartialEq)]
pub struct SearchRequest {
    pub query: String,
    pub limit: usize,
    pub filters: SearchFilters,
    pub attention_state: Option<AttentionState>,
}

impl SearchRequest {
    pub const DEFAULT_LIMIT: usize = 10;

    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            limit: Self::DEFAULT_LIMIT,
            filters: SearchFilters::default(),
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

    pub fn with_attention_state(mut self, attention_state: AttentionState) -> Self {
        self.attention_state = Some(attention_state);
        self
    }

    pub fn bounded_limit(&self) -> usize {
        self.limit.clamp(1, lexical::MAX_RECALL_LIMIT)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SearchResponse {
    pub applied_filters: AppliedFilters,
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SearchResult {
    pub record: MemoryRecord,
    pub snippet: String,
    pub citation: Citation,
    pub dsl: Option<FlatFactDslRecordV1>,
    pub score: ScoreBreakdown,
    pub trace: ResultTrace,
}

#[derive(Debug, Error)]
pub enum SearchError {
    #[error(transparent)]
    Lexical(#[from] LexicalSearchError),
    #[error(transparent)]
    Citation(#[from] CitationError),
    #[error("{0}")]
    InvalidFilters(String),
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

pub struct SearchService<'db> {
    lexical: lexical::LexicalSearch<'db>,
    repository: MemoryRepository<'db>,
    mode: RetrievalMode,
    embedding_backend: EmbeddingBackend,
    embedding_model: Option<String>,
    vector_backend: VectorBackend,
}

impl<'db> SearchService<'db> {
    pub fn new(conn: &'db Connection) -> Self {
        Self {
            lexical: lexical::LexicalSearch::new(conn),
            repository: MemoryRepository::new(conn),
            mode: RetrievalMode::LexicalOnly,
            embedding_backend: EmbeddingBackend::Disabled,
            embedding_model: None,
            vector_backend: VectorBackend::None,
        }
    }

    pub fn with_variant(conn: &'db Connection, variant: &RetrievalModeVariant) -> Self {
        Self {
            lexical: lexical::LexicalSearch::new(conn),
            repository: MemoryRepository::new(conn),
            mode: variant.mode,
            embedding_backend: variant.embedding_backend,
            embedding_model: variant.embedding.as_ref().map(|cfg| cfg.model.clone()),
            vector_backend: variant
                .vector
                .as_ref()
                .map(|cfg| cfg.backend)
                .unwrap_or(VectorBackend::None),
        }
    }

    pub fn with_runtime_config(
        conn: &'db Connection,
        config: &Config,
        mode_override: Option<RetrievalMode>,
    ) -> Self {
        Self {
            lexical: lexical::LexicalSearch::new(conn),
            repository: MemoryRepository::new(conn),
            mode: mode_override.unwrap_or(config.retrieval.mode),
            embedding_backend: config.embedding.backend,
            embedding_model: config.embedding.model.clone(),
            vector_backend: config.vector.backend,
        }
    }

    pub fn search(&self, request: &SearchRequest) -> Result<SearchResponse, SearchError> {
        request
            .filters
            .validate_taxonomy()
            .map_err(SearchError::InvalidFilters)?;
        let recall_request = if has_taxonomy_filters(&request.filters)
            && request.bounded_limit() < lexical::MAX_RECALL_LIMIT
        {
            request.clone().with_limit(lexical::MAX_RECALL_LIMIT)
        } else {
            request.clone()
        };
        let mut candidates = match self.mode {
            RetrievalMode::LexicalOnly => merge_candidates(
                self.lexical.recall(&recall_request)?,
                self.structured_recall(&recall_request)?,
            ),
            RetrievalMode::EmbeddingOnly => self.embedding_recall(&recall_request)?,
            RetrievalMode::Hybrid => {
                let lexical = merge_candidates(
                    self.lexical.recall(&recall_request)?,
                    self.structured_recall(&recall_request)?,
                );
                let embedding = self.embedding_recall(&recall_request)?;
                merge_candidates(lexical, embedding)
            }
        };
        self.attach_candidate_dsls_and_apply_taxonomy_filters(&mut candidates, &request.filters)?;
        let scored = score::score_candidates(request, candidates);
        let mut response = rerank::rerank_results(request, scored)?;
        self.attach_dsl_sidecars(&mut response.results)?;
        apply_taxonomy_filters(&mut response.results, &request.filters);
        response.results.truncate(request.bounded_limit());
        Ok(response)
    }

    fn embedding_recall(
        &self,
        request: &SearchRequest,
    ) -> Result<Vec<lexical::LexicalCandidate>, SearchError> {
        if !matches!(self.embedding_backend, EmbeddingBackend::Builtin)
            || !matches!(self.vector_backend, VectorBackend::SqliteVec)
        {
            return Ok(Vec::new());
        }

        let Some(model) = self.embedding_model.as_deref() else {
            return Ok(Vec::new());
        };
        let embeddings = self.repository.list_record_embeddings()?;
        let query_embedding =
            builtin_embedding(request.query.trim(), parse_builtin_dimensions(model));
        if query_embedding.is_empty() {
            return Ok(Vec::new());
        }

        let mut candidates = Vec::new();
        for embedding in embeddings {
            if embedding.backend != EmbeddingBackend::Builtin || embedding.model != model {
                continue;
            }
            let similarity = cosine_similarity(&query_embedding, &embedding.embedding);
            if similarity <= 0.0 {
                continue;
            }
            let Some(record) = self.repository.get_record(&embedding.record_id)? else {
                continue;
            };
            candidates.push(lexical::LexicalCandidate {
                lexical_raw: 1.0 - similarity,
                snippet: snippet(&record.content_text),
                dsl: None,
                query_strategies: vec![QueryStrategy::Embedding],
                record,
            });
        }

        candidates.sort_by(|left, right| {
            left.lexical_raw
                .total_cmp(&right.lexical_raw)
                .then_with(|| left.record.id.cmp(&right.record.id))
        });
        candidates.truncate(request.bounded_limit());
        Ok(candidates)
    }

    fn attach_dsl_sidecars(&self, results: &mut [SearchResult]) -> Result<(), SearchError> {
        if results.is_empty() {
            return Ok(());
        }

        let layered = self
            .repository
            .list_layered_records_for_ids(
                &results
                    .iter()
                    .map(|result| result.record.id.clone())
                    .collect::<Vec<_>>(),
            )?
            .into_iter()
            .filter_map(|record| record.dsl.map(|dsl| (record.record.id, dsl.payload)))
            .collect::<std::collections::BTreeMap<_, _>>();

        for result in results {
            result.dsl = layered.get(&result.record.id).cloned();
        }

        Ok(())
    }

    fn attach_candidate_dsls_and_apply_taxonomy_filters(
        &self,
        candidates: &mut Vec<lexical::LexicalCandidate>,
        filters: &SearchFilters,
    ) -> Result<(), SearchError> {
        if candidates.is_empty() {
            return Ok(());
        }

        let layered = self
            .repository
            .list_layered_records_for_ids(
                &candidates
                    .iter()
                    .map(|candidate| candidate.record.id.clone())
                    .collect::<Vec<_>>(),
            )?
            .into_iter()
            .filter_map(|record| record.dsl.map(|dsl| (record.record.id, dsl.payload)))
            .collect::<std::collections::BTreeMap<_, _>>();

        for candidate in candidates.iter_mut() {
            if candidate.dsl.is_none() {
                candidate.dsl = layered.get(&candidate.record.id).cloned();
            }
        }

        if has_taxonomy_filters(filters) {
            candidates.retain(|candidate| {
                candidate
                    .dsl
                    .as_ref()
                    .is_some_and(|dsl| dsl_matches_filters(dsl, filters))
            });
        }

        Ok(())
    }

    fn structured_recall(
        &self,
        request: &SearchRequest,
    ) -> Result<Vec<lexical::LexicalCandidate>, SearchError> {
        let terms = structured_query_terms(&request.query);
        if terms.is_empty() {
            return Ok(Vec::new());
        }

        let mut candidates = self
            .repository
            .list_layered_records()?
            .into_iter()
            .filter(|layered| matches_search_filters(&layered.record, &request.filters))
            .filter_map(|layered| {
                let dsl = layered.dsl?;
                let score = structured_match_score(&terms, &dsl.payload)?;
                Some(lexical::LexicalCandidate {
                    lexical_raw: 1.0 / (1.0 + score),
                    snippet: structured_snippet(&dsl.payload),
                    dsl: Some(dsl.payload),
                    query_strategies: vec![QueryStrategy::Structured],
                    record: layered.record,
                })
            })
            .collect::<Vec<_>>();

        candidates.sort_by(|left, right| {
            left.lexical_raw
                .total_cmp(&right.lexical_raw)
                .then_with(|| left.record.id.cmp(&right.record.id))
        });
        candidates.truncate(request.bounded_limit());
        Ok(candidates)
    }
}

fn merge_candidates(
    lexical: Vec<lexical::LexicalCandidate>,
    embedding: Vec<lexical::LexicalCandidate>,
) -> Vec<lexical::LexicalCandidate> {
    let mut merged = lexical;
    for candidate in embedding {
        if let Some(existing) = merged
            .iter_mut()
            .find(|existing| existing.record.id == candidate.record.id)
        {
            existing.lexical_raw = existing.lexical_raw.min(candidate.lexical_raw);
            if existing.dsl.is_none() {
                existing.dsl = candidate.dsl.clone();
            }
            if candidate
                .query_strategies
                .contains(&QueryStrategy::Structured)
            {
                existing.snippet = candidate.snippet.clone();
            }
            for strategy in candidate.query_strategies {
                if !existing.query_strategies.contains(&strategy) {
                    existing.query_strategies.push(strategy);
                }
            }
        } else {
            merged.push(candidate);
        }
    }
    merged
}

fn parse_builtin_dimensions(model: &str) -> usize {
    model
        .rsplit('-')
        .next()
        .and_then(|suffix| suffix.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(16)
}

fn builtin_embedding(text: &str, dimensions: usize) -> Vec<f32> {
    if text.is_empty() || dimensions == 0 {
        return Vec::new();
    }
    let mut values = vec![0.0f32; dimensions];
    for (index, byte) in text.bytes().enumerate() {
        let slot = (usize::from(byte) + index) % dimensions;
        values[slot] += f32::from(byte) / 255.0;
    }
    let magnitude = values.iter().map(|value| value * value).sum::<f32>().sqrt();
    if magnitude > 0.0 {
        for value in &mut values {
            *value /= magnitude;
        }
    }
    values
}

fn structured_query_terms(query: &str) -> Vec<String> {
    let lowered = query.trim().to_lowercase();
    if lowered.is_empty() {
        return Vec::new();
    }

    let mut terms = lowered
        .split(|ch: char| ch.is_ascii_punctuation() || ch.is_whitespace())
        .filter(|term| !term.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if terms.is_empty() {
        terms.push(lowered);
    }
    let mut seen = BTreeSet::new();
    terms.retain(|term| seen.insert(term.clone()));
    terms
}

fn matches_search_filters(record: &MemoryRecord, filters: &SearchFilters) -> bool {
    if let Some(scope) = filters.scope
        && record.scope != scope
    {
        return false;
    }
    if let Some(record_type) = filters.record_type
        && record.record_type != record_type
    {
        return false;
    }
    if let Some(truth_layer) = filters.truth_layer
        && record.truth_layer != truth_layer
    {
        return false;
    }
    if let Some(valid_at) = filters.valid_at.as_deref() {
        if let Some(valid_from) = record.validity.valid_from.as_deref()
            && !matches_required_timestamp(valid_from, valid_at, TimestampComparison::LessOrEqual)
        {
            return false;
        }
        if let Some(valid_to) = record.validity.valid_to.as_deref()
            && !matches_required_timestamp(valid_to, valid_at, TimestampComparison::GreaterOrEqual)
        {
            return false;
        }
    }
    if let Some(recorded_from) = filters.recorded_from.as_deref()
        && !matches_required_timestamp(
            record.timestamp.recorded_at.as_str(),
            recorded_from,
            TimestampComparison::GreaterOrEqual,
        )
    {
        return false;
    }
    if let Some(recorded_to) = filters.recorded_to.as_deref()
        && !matches_required_timestamp(
            record.timestamp.recorded_at.as_str(),
            recorded_to,
            TimestampComparison::LessOrEqual,
        )
    {
        return false;
    }

    true
}

fn apply_taxonomy_filters(results: &mut Vec<SearchResult>, filters: &SearchFilters) {
    if !has_taxonomy_filters(filters) {
        return;
    }

    results.retain(|result| {
        result
            .dsl
            .as_ref()
            .is_some_and(|dsl| dsl_matches_filters(dsl, filters))
    });
}

fn has_taxonomy_filters(filters: &SearchFilters) -> bool {
    filters.domain.is_some()
        || filters.topic.is_some()
        || filters.aspect.is_some()
        || filters.kind.is_some()
}

fn dsl_matches_filters(dsl: &FlatFactDslRecordV1, filters: &SearchFilters) -> bool {
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

    true
}

fn structured_match_score(terms: &[String], dsl: &FlatFactDslRecordV1) -> Option<f32> {
    let mut score = 0.0f32;
    let mut matched = false;

    for term in terms {
        let taxonomy_hit = [&dsl.domain, &dsl.topic, &dsl.aspect, &dsl.kind]
            .iter()
            .any(|field| field.as_str() == term);
        if taxonomy_hit {
            score += 1.0;
            matched = true;
        }

        if dsl.claim.to_lowercase().contains(term) {
            score += 0.8;
            matched = true;
        }

        for field in [
            dsl.why.as_deref(),
            dsl.time.as_deref(),
            dsl.cond.as_deref(),
            dsl.impact.as_deref(),
            Some(dsl.source_ref.as_str()),
        ]
        .into_iter()
        .flatten()
        {
            if field.to_lowercase().contains(term) {
                score += 0.35;
                matched = true;
                break;
            }
        }
    }

    matched.then_some(score)
}

fn structured_snippet(dsl: &FlatFactDslRecordV1) -> String {
    let mut parts = vec![dsl.claim.clone()];
    if let Some(why) = dsl.why.as_deref() {
        parts.push(format!("WHY: {why}"));
    }
    if let Some(impact) = dsl.impact.as_deref() {
        parts.push(format!("IMPACT: {impact}"));
    }
    parts.join(" | ")
}

fn cosine_similarity(left: &[f32], right: &[f32]) -> f32 {
    if left.len() != right.len() || left.is_empty() {
        return 0.0;
    }

    let dot = left
        .iter()
        .zip(right.iter())
        .map(|(l, r)| l * r)
        .sum::<f32>();
    let left_mag = left.iter().map(|value| value * value).sum::<f32>().sqrt();
    let right_mag = right.iter().map(|value| value * value).sum::<f32>().sqrt();
    if left_mag == 0.0 || right_mag == 0.0 {
        0.0
    } else {
        dot / (left_mag * right_mag)
    }
}

fn snippet(content: &str) -> String {
    content.chars().take(96).collect::<String>()
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
