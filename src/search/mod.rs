pub mod citation;
pub mod filter;
pub mod lexical;
pub mod rerank;
pub mod score;

use rusqlite::Connection;
use serde::Serialize;
use thiserror::Error;

use crate::memory::record::MemoryRecord;

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
}

pub struct SearchService<'db> {
    lexical: lexical::LexicalSearch<'db>,
}

impl<'db> SearchService<'db> {
    pub fn new(conn: &'db Connection) -> Self {
        Self {
            lexical: lexical::LexicalSearch::new(conn),
        }
    }

    pub fn search(&self, request: &SearchRequest) -> Result<SearchResponse, SearchError> {
        let candidates = self.lexical.recall(request)?;
        let scored = score::score_candidates(request, candidates);
        Ok(rerank::rerank_results(request, scored)?)
    }
}
