pub mod lexical;
pub mod score;

use rusqlite::Connection;
use thiserror::Error;

use crate::memory::record::MemoryRecord;

pub use lexical::{LexicalCandidate, LexicalSearchError, QueryStrategy};
pub use score::ScoreBreakdown;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchRequest {
    pub query: String,
    pub limit: usize,
}

impl SearchRequest {
    pub const DEFAULT_LIMIT: usize = 10;

    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            limit: Self::DEFAULT_LIMIT,
        }
    }

    pub fn bounded_limit(&self) -> usize {
        self.limit.clamp(1, lexical::MAX_RECALL_LIMIT)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    pub record: MemoryRecord,
    pub snippet: String,
    pub query_strategies: Vec<QueryStrategy>,
    pub score: ScoreBreakdown,
}

#[derive(Debug, Error)]
pub enum SearchError {
    #[error(transparent)]
    Lexical(#[from] LexicalSearchError),
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

    pub fn search(&self, request: &SearchRequest) -> Result<Vec<SearchResult>, SearchError> {
        let candidates = self.lexical.recall(request)?;
        Ok(score::score_candidates(request, candidates))
    }
}
