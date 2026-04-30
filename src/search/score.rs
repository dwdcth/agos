use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::{
    cognition::attention::{
        ATTENTION_BONUS_CAP, AttentionContribution, AttentionTrace, lane_weight,
    },
    memory::record::RecordType,
    search::{LexicalCandidate, QueryStrategy, SearchRequest},
};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScoreBreakdown {
    pub lexical_raw: f32,
    pub lexical_base: f32,
    pub keyword_bonus: f32,
    pub importance_bonus: f32,
    pub recency_bonus: f32,
    pub attention_bonus: f32,
    pub final_score: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScoredCandidate {
    pub record: crate::memory::record::MemoryRecord,
    pub snippet: String,
    pub query_strategies: Vec<QueryStrategy>,
    pub score: ScoreBreakdown,
    pub attention_trace: Option<AttentionTrace>,
}

pub fn score_candidates(
    request: &SearchRequest,
    candidates: Vec<LexicalCandidate>,
) -> Vec<ScoredCandidate> {
    if candidates.is_empty() {
        return Vec::new();
    }

    let (recency_positions, recency_slots) = recency_positions(&candidates);
    let query_terms = query_terms(&request.query);
    let attention_delta = request
        .attention_state
        .as_ref()
        .filter(|state| !state.is_empty())
        .map(|state| &state.delta);
    candidates
        .into_iter()
        .map(|candidate| {
            let lexical_base = 1.0 / (1.0 + candidate.lexical_raw.abs());
            let keyword_bonus = keyword_bonus(&query_terms, &candidate);
            let importance_bonus = importance_bonus(candidate.record.record_type);
            let recency_bonus = recency_bonus(
                recency_positions
                    .get(&candidate.record.id)
                    .copied()
                    .unwrap_or(usize::MAX),
                recency_slots,
            );
            let (attention_bonus, attention_trace) =
                compute_attention_bonus(attention_delta, &candidate);
            let final_score =
                (lexical_base + keyword_bonus + importance_bonus + recency_bonus + attention_bonus)
                    .min(
                        lexical_base
                            + keyword_bonus
                            + importance_bonus
                            + recency_bonus
                            + ATTENTION_BONUS_CAP,
                    );

            ScoredCandidate {
                record: candidate.record,
                snippet: candidate.snippet,
                query_strategies: sorted_strategies(candidate.query_strategies),
                score: ScoreBreakdown {
                    lexical_raw: candidate.lexical_raw,
                    lexical_base,
                    keyword_bonus,
                    importance_bonus,
                    recency_bonus,
                    attention_bonus,
                    final_score,
                },
                attention_trace,
            }
        })
        .collect::<Vec<_>>()
}

fn compute_attention_bonus(
    attention_delta: Option<&crate::cognition::attention::AttentionDelta>,
    candidate: &LexicalCandidate,
) -> (f32, Option<AttentionTrace>) {
    let Some(delta) = attention_delta else {
        return (0.0, None);
    };

    if delta.contributions.is_empty() {
        return (0.0, None);
    }

    let label = candidate
        .record
        .source
        .label
        .as_deref()
        .unwrap_or("")
        .to_lowercase();
    let content = candidate.record.content_text.to_lowercase();
    let dsl_fields = candidate
        .dsl
        .as_ref()
        .map(|dsl| {
            let mut fields = vec![
                dsl.domain.to_lowercase(),
                dsl.topic.to_lowercase(),
                dsl.aspect.to_lowercase(),
                dsl.kind.to_lowercase(),
                dsl.claim.to_lowercase(),
                dsl.source_ref.to_lowercase(),
            ];
            for extra in [
                dsl.why.as_deref(),
                dsl.time.as_deref(),
                dsl.cond.as_deref(),
                dsl.impact.as_deref(),
            ]
            .into_iter()
            .flatten()
            {
                fields.push(extra.to_lowercase());
            }
            fields
        })
        .unwrap_or_default();

    let mut total_bonus: f32 = 0.0;
    let mut matched_contributions = Vec::new();

    for contribution in &delta.contributions {
        let cue_lower = contribution.cue.to_lowercase();
        let weight = lane_weight(contribution.lane);

        // Split cue into individual terms for matching
        let cue_terms = cue_lower
            .split(|ch: char| ch.is_ascii_punctuation() || ch.is_whitespace())
            .filter(|term| !term.is_empty())
            .collect::<Vec<_>>();

        if cue_terms.is_empty() {
            continue;
        }

        let mut matched_fields = Vec::new();
        let mut any_matched = false;

        for term in cue_terms {
            if !label.is_empty() && label.contains(term) {
                if !matched_fields.contains(&"label".to_string()) {
                    matched_fields.push("label".to_string());
                }
                any_matched = true;
            }
            if content.contains(term) {
                if !matched_fields.contains(&"content".to_string()) {
                    matched_fields.push("content".to_string());
                }
                any_matched = true;
            }
            if dsl_fields.iter().any(|field| field.contains(term)) {
                if !matched_fields.contains(&"dsl".to_string()) {
                    matched_fields.push("dsl".to_string());
                }
                any_matched = true;
            }
        }

        if any_matched {
            let bonus = weight.min(ATTENTION_BONUS_CAP - total_bonus).max(0.0);
            if bonus > 0.0 {
                total_bonus += bonus;
                matched_contributions.push(AttentionContribution {
                    lane: contribution.lane,
                    source: contribution.source.clone(),
                    cue: contribution.cue.clone(),
                    matched_fields,
                    bonus,
                });
            }
        }
    }

    total_bonus = total_bonus.min(ATTENTION_BONUS_CAP);

    if matched_contributions.is_empty() {
        (0.0, None)
    } else {
        (
            total_bonus,
            Some(AttentionTrace {
                total_bonus,
                contributions: matched_contributions,
            }),
        )
    }
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

    let mut seen = BTreeSet::new();
    terms.retain(|term| seen.insert(term.clone()));

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
    let structured_fields = candidate
        .dsl
        .as_ref()
        .map(|dsl| {
            let mut fields = vec![
                dsl.domain.to_lowercase(),
                dsl.topic.to_lowercase(),
                dsl.aspect.to_lowercase(),
                dsl.kind.to_lowercase(),
                dsl.claim.to_lowercase(),
                dsl.source_ref.to_lowercase(),
            ];
            for extra in [
                dsl.why.as_deref(),
                dsl.time.as_deref(),
                dsl.cond.as_deref(),
                dsl.impact.as_deref(),
            ]
            .into_iter()
            .flatten()
            {
                fields.push(extra.to_lowercase());
            }
            fields
        })
        .unwrap_or_default();
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
        if structured_fields.iter().any(|field| field.contains(term)) {
            bonus += if matched { 0.03 } else { 0.05 };
        }
    }

    bonus.min(0.24)
}

fn importance_bonus(record_type: RecordType) -> f32 {
    match record_type {
        RecordType::Decision => 0.08,
        RecordType::Fact => 0.05,
        RecordType::Observation => 0.02,
    }
}

fn recency_positions(candidates: &[LexicalCandidate]) -> (BTreeMap<String, usize>, usize) {
    let mut parsed_slots = candidates
        .iter()
        .filter_map(|candidate| parse_recorded_at(&candidate.record.timestamp.recorded_at))
        .collect::<Vec<_>>();
    parsed_slots.sort();
    parsed_slots.dedup();
    parsed_slots.reverse();

    let mut fallback_slots = candidates
        .iter()
        .filter(|candidate| parse_recorded_at(&candidate.record.timestamp.recorded_at).is_none())
        .map(|candidate| candidate.record.timestamp.recorded_at.clone())
        .collect::<Vec<_>>();
    fallback_slots.sort();
    fallback_slots.dedup();
    fallback_slots.reverse();

    let mut positions = BTreeMap::new();
    for candidate in candidates {
        let position =
            if let Some(parsed) = parse_recorded_at(&candidate.record.timestamp.recorded_at) {
                parsed_slots
                    .iter()
                    .position(|slot| *slot == parsed)
                    .unwrap_or(usize::MAX)
            } else {
                let fallback_position = fallback_slots
                    .iter()
                    .position(|slot| slot == &candidate.record.timestamp.recorded_at)
                    .unwrap_or(usize::MAX);
                parsed_slots.len().saturating_add(fallback_position)
            };
        positions.insert(candidate.record.id.clone(), position);
    }

    (positions, parsed_slots.len() + fallback_slots.len())
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
        QueryStrategy::Structured => 2,
        QueryStrategy::Embedding => 3,
    });
    strategies
}

fn parse_recorded_at(value: &str) -> Option<i128> {
    OffsetDateTime::parse(value, &Rfc3339)
        .ok()
        .map(|value| value.unix_timestamp_nanos())
}
