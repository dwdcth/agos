pub mod citation;
pub mod filter;
pub mod lexical;
pub mod rerank;
pub mod score;

use rusqlite::Connection;
use serde::Serialize;
use thiserror::Error;

use crate::{
    core::config::{EmbeddingBackend, RetrievalMode, RetrievalModeVariant, VectorBackend},
    memory::{record::MemoryRecord, repository::{MemoryRepository, RepositoryError}},
};

pub use citation::{Citation, CitationAnchor, CitationError};
pub use filter::{AppliedFilters, SearchFilters};
pub use lexical::{LexicalCandidate, LexicalSearchError, QueryStrategy};
pub use rerank::ResultTrace;
pub use score::ScoreBreakdown;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchRequest {
    pub query: String,
    pub limit: usize,
    pub filters: SearchFilters,
}

impl SearchRequest {
    pub const DEFAULT_LIMIT: usize = 10;

    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            limit: Self::DEFAULT_LIMIT,
            filters: SearchFilters::default(),
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
    pub score: ScoreBreakdown,
    pub trace: ResultTrace,
}

#[derive(Debug, Error)]
pub enum SearchError {
    #[error(transparent)]
    Lexical(#[from] LexicalSearchError),
    #[error(transparent)]
    Citation(#[from] CitationError),
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

    pub fn search(&self, request: &SearchRequest) -> Result<SearchResponse, SearchError> {
        let candidates = match self.mode {
            RetrievalMode::LexicalOnly => self.lexical.recall(request)?,
            RetrievalMode::EmbeddingOnly => self.embedding_recall(request)?,
            RetrievalMode::Hybrid => {
                let lexical = self.lexical.recall(request)?;
                let embedding = self.embedding_recall(request)?;
                merge_candidates(lexical, embedding)
            }
        };
        let scored = score::score_candidates(request, candidates);
        Ok(rerank::rerank_results(request, scored)?)
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
        let query_embedding = builtin_embedding(request.query.trim(), parse_builtin_dimensions(model));
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
            if !existing
                .query_strategies
                .contains(&QueryStrategy::Embedding)
            {
                existing.query_strategies.push(QueryStrategy::Embedding);
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
