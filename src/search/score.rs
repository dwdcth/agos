use crate::{
    memory::record::RecordType,
    search::{LexicalCandidate, QueryStrategy, SearchRequest, SearchResult},
};

#[derive(Debug, Clone, PartialEq)]
pub struct ScoreBreakdown {
    pub lexical_raw: f32,
    pub lexical_base: f32,
    pub keyword_bonus: f32,
    pub importance_bonus: f32,
    pub recency_bonus: f32,
    pub emotion_bonus: f32,
    pub final_score: f32,
}

pub fn score_candidates(
    request: &SearchRequest,
    candidates: Vec<LexicalCandidate>,
) -> Vec<SearchResult> {
    if candidates.is_empty() {
        return Vec::new();
    }

    let recency_order = recency_order(&candidates);
    let query_terms = query_terms(&request.query);
    let mut results = candidates
        .into_iter()
        .map(|candidate| {
            let lexical_base = 1.0 / (1.0 + candidate.lexical_raw.abs());
            let keyword_bonus = keyword_bonus(&query_terms, &candidate);
            let importance_bonus = importance_bonus(candidate.record.record_type);
            let recency_bonus = recency_bonus(
                recency_order
                    .iter()
                    .position(|recorded_at| recorded_at == &candidate.record.timestamp.recorded_at)
                    .unwrap_or(usize::MAX),
                recency_order.len(),
            );
            let emotion_bonus = 0.0;
            let final_score =
                lexical_base + keyword_bonus + importance_bonus + recency_bonus + emotion_bonus;

            SearchResult {
                record: candidate.record,
                snippet: candidate.snippet,
                query_strategies: sorted_strategies(candidate.query_strategies),
                score: ScoreBreakdown {
                    lexical_raw: candidate.lexical_raw,
                    lexical_base,
                    keyword_bonus,
                    importance_bonus,
                    recency_bonus,
                    emotion_bonus,
                    final_score,
                },
            }
        })
        .collect::<Vec<_>>();

    results.sort_by(|left, right| {
        right
            .score
            .final_score
            .total_cmp(&left.score.final_score)
            .then_with(|| left.score.lexical_raw.total_cmp(&right.score.lexical_raw))
            .then_with(|| left.record.id.cmp(&right.record.id))
    });

    results
}

fn query_terms(query: &str) -> Vec<String> {
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

    terms
}

fn keyword_bonus(terms: &[String], candidate: &LexicalCandidate) -> f32 {
    let label = candidate
        .record
        .source
        .label
        .as_deref()
        .unwrap_or("")
        .to_lowercase();
    let content = candidate.record.content_text.to_lowercase();
    let mut bonus: f32 = 0.0;

    for term in terms {
        let mut matched = false;
        if !label.is_empty() && label.contains(term) {
            bonus += 0.04;
            matched = true;
        }
        if content.contains(term) {
            bonus += if matched { 0.01 } else { 0.02 };
        }
    }

    bonus.min(0.18)
}

fn importance_bonus(record_type: RecordType) -> f32 {
    match record_type {
        RecordType::Decision => 0.08,
        RecordType::Fact => 0.05,
        RecordType::Observation => 0.02,
    }
}

fn recency_order(candidates: &[LexicalCandidate]) -> Vec<String> {
    let mut recorded = candidates
        .iter()
        .map(|candidate| candidate.record.timestamp.recorded_at.clone())
        .collect::<Vec<_>>();
    recorded.sort();
    recorded.dedup();
    recorded.reverse();
    recorded
}

fn recency_bonus(position: usize, total: usize) -> f32 {
    if total <= 1 || position == usize::MAX {
        return 0.0;
    }

    let steps = total - 1;
    let remaining = steps.saturating_sub(position);
    (remaining as f32 / steps as f32) * 0.03
}

fn sorted_strategies(mut strategies: Vec<QueryStrategy>) -> Vec<QueryStrategy> {
    strategies.sort_by_key(|strategy| match strategy {
        QueryStrategy::Jieba => 0,
        QueryStrategy::Simple => 1,
    });
    strategies
}
