use serde::Serialize;

use crate::search::{
    AppliedFilters, SearchRequest, SearchResponse, SearchResult, citation::Citation,
    citation::CitationError, score::ScoredCandidate,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelContribution {
    LexicalOnly,
    EmbeddingOnly,
    Hybrid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResultTrace {
    pub matched_query: String,
    pub query_strategies: Vec<crate::search::QueryStrategy>,
    pub channel_contribution: ChannelContribution,
    pub applied_filters: AppliedFilters,
}

pub fn rerank_results(
    request: &SearchRequest,
    scored: Vec<ScoredCandidate>,
) -> Result<SearchResponse, CitationError> {
    let applied_filters = request.filters.clone();
    let mut results = scored
        .into_iter()
        .map(|candidate| {
            Ok(SearchResult {
                citation: Citation::from_record(&candidate.record)?,
                record: candidate.record,
                snippet: candidate.snippet,
                dsl: None,
                score: candidate.score,
                trace: ResultTrace {
                    matched_query: request.query.clone(),
                    channel_contribution: channel_contribution(&candidate.query_strategies),
                    query_strategies: candidate.query_strategies,
                    applied_filters: applied_filters.clone(),
                },
            })
        })
        .collect::<Result<Vec<_>, CitationError>>()?;

    results.sort_by(|left, right| {
        right
            .score
            .final_score
            .total_cmp(&left.score.final_score)
            .then_with(|| left.score.lexical_raw.total_cmp(&right.score.lexical_raw))
            .then_with(|| left.record.id.cmp(&right.record.id))
    });

    Ok(SearchResponse {
        applied_filters,
        results,
    })
}

fn channel_contribution(strategies: &[crate::search::QueryStrategy]) -> ChannelContribution {
    let has_embedding = strategies.contains(&crate::search::QueryStrategy::Embedding);
    let has_lexical = strategies
        .iter()
        .any(|strategy| *strategy != crate::search::QueryStrategy::Embedding);

    match (has_lexical, has_embedding) {
        (true, true) => ChannelContribution::Hybrid,
        (false, true) => ChannelContribution::EmbeddingOnly,
        _ => ChannelContribution::LexicalOnly,
    }
}
