use rusqlite::{Connection, params};
use serde::Serialize;
use thiserror::Error;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::{
    memory::{
        dsl::FlatFactDslRecordV1,
        record::{
            ChunkAnchor, ChunkMetadata, MemoryRecord, Provenance, RecordTimestamp, RecordType,
            Scope, SourceKind, SourceRef, TruthLayer, ValidityWindow,
        },
    },
    search::SearchRequest,
};

pub const MAX_RECALL_LIMIT: usize = 25;

const JIEBA_SQL: &str = r#"
    SELECT
        mr.id,
        mr.source_uri,
        mr.source_kind,
        mr.source_label,
        mr.recorded_at,
        mr.scope,
        mr.record_type,
        mr.truth_layer,
        mr.provenance_json,
        mr.content_text,
        mr.chunk_index,
        mr.chunk_count,
        mr.chunk_anchor_json,
        mr.content_hash,
        mr.valid_from,
        mr.valid_to,
        mr.created_at,
        mr.updated_at,
        bm25(memory_records_fts) AS lexical_raw,
        snippet(memory_records_fts, 1, '[', ']', '...', 12) AS snippet
    FROM memory_records_fts
    JOIN memory_records AS mr ON mr.rowid = memory_records_fts.rowid
    WHERE memory_records_fts MATCH jieba_query(?1)
      AND (?3 IS NULL OR mr.scope = ?3)
      AND (?4 IS NULL OR mr.record_type = ?4)
      AND (?5 IS NULL OR mr.truth_layer = ?5)
      AND (?6 IS NULL OR mr.valid_from IS NULL OR mr.valid_from <= ?6)
      AND (?6 IS NULL OR mr.valid_to IS NULL OR mr.valid_to >= ?6)
      AND (?7 IS NULL OR mr.recorded_at >= ?7)
      AND (?8 IS NULL OR mr.recorded_at <= ?8)
    ORDER BY bm25(memory_records_fts), mr.recorded_at DESC, mr.id ASC
    LIMIT ?2
"#;

const SIMPLE_SQL: &str = r#"
    SELECT
        mr.id,
        mr.source_uri,
        mr.source_kind,
        mr.source_label,
        mr.recorded_at,
        mr.scope,
        mr.record_type,
        mr.truth_layer,
        mr.provenance_json,
        mr.content_text,
        mr.chunk_index,
        mr.chunk_count,
        mr.chunk_anchor_json,
        mr.content_hash,
        mr.valid_from,
        mr.valid_to,
        mr.created_at,
        mr.updated_at,
        bm25(memory_records_fts) AS lexical_raw,
        snippet(memory_records_fts, 1, '[', ']', '...', 12) AS snippet
    FROM memory_records_fts
    JOIN memory_records AS mr ON mr.rowid = memory_records_fts.rowid
    WHERE memory_records_fts MATCH simple_query(?1)
      AND (?3 IS NULL OR mr.scope = ?3)
      AND (?4 IS NULL OR mr.record_type = ?4)
      AND (?5 IS NULL OR mr.truth_layer = ?5)
      AND (?6 IS NULL OR mr.valid_from IS NULL OR mr.valid_from <= ?6)
      AND (?6 IS NULL OR mr.valid_to IS NULL OR mr.valid_to >= ?6)
      AND (?7 IS NULL OR mr.recorded_at >= ?7)
      AND (?8 IS NULL OR mr.recorded_at <= ?8)
    ORDER BY bm25(memory_records_fts), mr.recorded_at DESC, mr.id ASC
    LIMIT ?2
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum QueryStrategy {
    Jieba,
    Simple,
    Structured,
    Embedding,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LexicalCandidate {
    pub record: MemoryRecord,
    pub lexical_raw: f32,
    pub snippet: String,
    pub dsl: Option<FlatFactDslRecordV1>,
    pub query_strategies: Vec<QueryStrategy>,
}

#[derive(Debug, Error)]
pub enum LexicalSearchError {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("invalid {field} stored in database: {value}")]
    InvalidEnum { field: &'static str, value: String },
    #[error("incomplete chunk metadata stored for record {record_id}")]
    IncompleteChunkMetadata { record_id: String },
}

pub struct LexicalSearch<'db> {
    conn: &'db Connection,
}

#[derive(Debug, Clone, Copy)]
struct RecallFilters<'a> {
    scope: Option<&'a str>,
    record_type: Option<&'a str>,
    truth_layer: Option<&'a str>,
    valid_at: Option<&'a str>,
    recorded_from: Option<&'a str>,
    recorded_to: Option<&'a str>,
}

impl<'db> LexicalSearch<'db> {
    pub fn new(conn: &'db Connection) -> Self {
        Self { conn }
    }

    pub fn recall(
        &self,
        request: &SearchRequest,
    ) -> Result<Vec<LexicalCandidate>, LexicalSearchError> {
        let query = request.query.trim();
        if query.is_empty() {
            return Ok(Vec::new());
        }

        let apply_temporal_filters = has_temporal_filters(request);
        let limit = if apply_temporal_filters {
            MAX_RECALL_LIMIT
        } else {
            request.bounded_limit()
        };
        let limit = i64::try_from(limit).expect("bounded recall limit should fit in i64");
        let mut candidates = Vec::new();
        let filters = RecallFilters {
            scope: request.filters.scope_value(),
            record_type: request.filters.record_type_value(),
            truth_layer: request.filters.truth_layer_value(),
            valid_at: if apply_temporal_filters {
                None
            } else {
                request.filters.valid_at.as_deref()
            },
            recorded_from: if apply_temporal_filters {
                None
            } else {
                request.filters.recorded_from.as_deref()
            },
            recorded_to: if apply_temporal_filters {
                None
            } else {
                request.filters.recorded_to.as_deref()
            },
        };

        self.collect_candidates(
            query,
            limit,
            QueryStrategy::Jieba,
            JIEBA_SQL,
            filters,
            &mut candidates,
        )?;
        self.collect_candidates(
            query,
            limit,
            QueryStrategy::Simple,
            SIMPLE_SQL,
            filters,
            &mut candidates,
        )?;

        if apply_temporal_filters {
            candidates.retain(|candidate| matches_temporal_filters(&candidate.record, request));
        }

        candidates.sort_by(|left, right| {
            left.lexical_raw
                .total_cmp(&right.lexical_raw)
                .then_with(|| left.record.id.cmp(&right.record.id))
        });
        candidates.truncate(request.bounded_limit());

        Ok(candidates)
    }

    fn collect_candidates(
        &self,
        query: &str,
        limit: i64,
        strategy: QueryStrategy,
        sql: &str,
        filters: RecallFilters<'_>,
        candidates: &mut Vec<LexicalCandidate>,
    ) -> Result<(), LexicalSearchError> {
        let mut statement = self.conn.prepare(sql)?;
        let mut rows = statement.query(params![
            query,
            limit,
            filters.scope,
            filters.record_type,
            filters.truth_layer,
            filters.valid_at,
            filters.recorded_from,
            filters.recorded_to
        ])?;

        while let Some(row) = rows.next()? {
            let record = map_record_row(row)?;
            let lexical_raw = row.get::<_, f32>(18)?;
            let snippet = row.get::<_, String>(19)?;

            if let Some(existing) = candidates
                .iter_mut()
                .find(|candidate| candidate.record.id == record.id)
            {
                if lexical_raw < existing.lexical_raw {
                    existing.lexical_raw = lexical_raw;
                }
                if existing.snippet.is_empty() && !snippet.is_empty() {
                    existing.snippet = snippet;
                }
                if !existing.query_strategies.contains(&strategy) {
                    existing.query_strategies.push(strategy);
                }
                continue;
            }

            candidates.push(LexicalCandidate {
                record,
                lexical_raw,
                snippet,
                dsl: None,
                query_strategies: vec![strategy],
            });
        }

        Ok(())
    }
}

fn has_temporal_filters(request: &SearchRequest) -> bool {
    request.filters.valid_at.is_some()
        || request.filters.recorded_from.is_some()
        || request.filters.recorded_to.is_some()
}

fn matches_temporal_filters(record: &MemoryRecord, request: &SearchRequest) -> bool {
    if let Some(valid_at) = request.filters.valid_at.as_deref()
        && !matches_optional_timestamp(
            record.validity.valid_from.as_deref(),
            valid_at,
            TimestampComparison::LessOrEqual,
        )
    {
        return false;
    }
    if let Some(valid_at) = request.filters.valid_at.as_deref()
        && !matches_optional_timestamp(
            record.validity.valid_to.as_deref(),
            valid_at,
            TimestampComparison::GreaterOrEqual,
        )
    {
        return false;
    }
    if let Some(recorded_from) = request.filters.recorded_from.as_deref()
        && !matches_required_timestamp(
            &record.timestamp.recorded_at,
            recorded_from,
            TimestampComparison::GreaterOrEqual,
        )
    {
        return false;
    }
    if let Some(recorded_to) = request.filters.recorded_to.as_deref()
        && !matches_required_timestamp(
            &record.timestamp.recorded_at,
            recorded_to,
            TimestampComparison::LessOrEqual,
        )
    {
        return false;
    }

    true
}

#[derive(Clone, Copy)]
enum TimestampComparison {
    LessOrEqual,
    GreaterOrEqual,
}

fn matches_optional_timestamp(
    candidate: Option<&str>,
    filter: &str,
    comparison: TimestampComparison,
) -> bool {
    let Some(candidate) = candidate else {
        return true;
    };
    matches_required_timestamp(candidate, filter, comparison)
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

fn parse_rfc3339(value: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(value, &Rfc3339).ok()
}

fn map_record_row(row: &rusqlite::Row<'_>) -> Result<MemoryRecord, LexicalSearchError> {
    let source_kind = row.get::<_, String>(2)?;
    let scope = row.get::<_, String>(5)?;
    let record_type = row.get::<_, String>(6)?;
    let truth_layer = row.get::<_, String>(7)?;
    let provenance_json = row.get::<_, String>(8)?;
    let record_id = row.get::<_, String>(0)?;
    let chunk = map_chunk_metadata(row, &record_id)?;

    Ok(MemoryRecord {
        id: record_id,
        source: SourceRef {
            uri: row.get(1)?,
            kind: parse_source_kind(&source_kind)?,
            label: row.get(3)?,
        },
        timestamp: RecordTimestamp {
            recorded_at: row.get(4)?,
            created_at: row.get(16)?,
            updated_at: row.get(17)?,
        },
        scope: parse_scope(&scope)?,
        record_type: parse_record_type(&record_type)?,
        truth_layer: parse_truth_layer(&truth_layer)?,
        provenance: serde_json::from_str::<Provenance>(&provenance_json)?,
        content_text: row.get(9)?,
        chunk,
        validity: ValidityWindow {
            valid_from: row.get(14)?,
            valid_to: row.get(15)?,
        },
    })
}

fn map_chunk_metadata(
    row: &rusqlite::Row<'_>,
    record_id: &str,
) -> Result<Option<ChunkMetadata>, LexicalSearchError> {
    let chunk_index = row.get::<_, Option<u32>>(10)?;
    let chunk_count = row.get::<_, Option<u32>>(11)?;
    let anchor_json = row.get::<_, Option<String>>(12)?;
    let content_hash = row.get::<_, Option<String>>(13)?;

    match (chunk_index, chunk_count, anchor_json, content_hash) {
        (None, None, None, None) => Ok(None),
        (Some(chunk_index), Some(chunk_count), Some(anchor_json), Some(content_hash)) => {
            let anchor = serde_json::from_str::<ChunkAnchor>(&anchor_json)?;
            Ok(Some(ChunkMetadata {
                chunk_index,
                chunk_count,
                anchor,
                content_hash,
            }))
        }
        _ => Err(LexicalSearchError::IncompleteChunkMetadata {
            record_id: record_id.to_string(),
        }),
    }
}

fn parse_source_kind(value: &str) -> Result<SourceKind, LexicalSearchError> {
    SourceKind::parse(value).ok_or_else(|| LexicalSearchError::InvalidEnum {
        field: "source_kind",
        value: value.to_string(),
    })
}

fn parse_scope(value: &str) -> Result<Scope, LexicalSearchError> {
    Scope::parse(value).ok_or_else(|| LexicalSearchError::InvalidEnum {
        field: "scope",
        value: value.to_string(),
    })
}

fn parse_record_type(value: &str) -> Result<RecordType, LexicalSearchError> {
    RecordType::parse(value).ok_or_else(|| LexicalSearchError::InvalidEnum {
        field: "record_type",
        value: value.to_string(),
    })
}

fn parse_truth_layer(value: &str) -> Result<TruthLayer, LexicalSearchError> {
    TruthLayer::parse(value).ok_or_else(|| LexicalSearchError::InvalidEnum {
        field: "truth_layer",
        value: value.to_string(),
    })
}
