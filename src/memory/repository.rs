use rusqlite::{Connection, OptionalExtension, params};
use thiserror::Error;

use crate::memory::record::{
    ChunkAnchor, ChunkMetadata, MemoryRecord, Provenance, RecordTimestamp, RecordType, Scope,
    SourceKind, SourceRef, TruthLayer, ValidityWindow,
};
use crate::memory::truth::{T3Confidence, T3RevocationState, T3State, TruthRecord};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeCount {
    pub scope: Scope,
    pub count: u64,
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("invalid {field} stored in database: {value}")]
    InvalidEnum { field: &'static str, value: String },
    #[error("incomplete chunk metadata stored for record {record_id}")]
    IncompleteChunkMetadata { record_id: String },
    #[error("missing t3 governance state for record {record_id}")]
    MissingT3State { record_id: String },
}

pub struct MemoryRepository<'db> {
    conn: &'db Connection,
}

impl<'db> MemoryRepository<'db> {
    pub fn new(conn: &'db Connection) -> Self {
        Self { conn }
    }

    pub fn insert_record(&self, record: &MemoryRecord) -> Result<(), RepositoryError> {
        let provenance_json = serde_json::to_string(&record.provenance)?;
        let chunk_index = record.chunk.as_ref().map(|chunk| chunk.chunk_index);
        let chunk_count = record.chunk.as_ref().map(|chunk| chunk.chunk_count);
        let chunk_anchor_json = record
            .chunk
            .as_ref()
            .map(|chunk| serde_json::to_string(&chunk.anchor))
            .transpose()?;
        let content_hash = record
            .chunk
            .as_ref()
            .map(|chunk| chunk.content_hash.as_str());

        self.conn.execute(
            r#"
            INSERT INTO memory_records (
                id,
                source_uri,
                source_kind,
                source_label,
                recorded_at,
                scope,
                record_type,
                truth_layer,
                provenance_json,
                content_text,
                chunk_index,
                chunk_count,
                chunk_anchor_json,
                content_hash,
                valid_from,
                valid_to,
                created_at,
                updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
            "#,
            params![
                &record.id,
                &record.source.uri,
                record.source.kind.as_str(),
                &record.source.label,
                &record.timestamp.recorded_at,
                record.scope.as_str(),
                record.record_type.as_str(),
                record.truth_layer.as_str(),
                provenance_json,
                &record.content_text,
                chunk_index,
                chunk_count,
                chunk_anchor_json,
                content_hash,
                &record.validity.valid_from,
                &record.validity.valid_to,
                &record.timestamp.created_at,
                &record.timestamp.updated_at,
            ],
        )?;

        if matches!(record.truth_layer, TruthLayer::T3) {
            self.conn.execute(
                r#"
                INSERT INTO truth_t3_state (
                    record_id,
                    confidence,
                    revocation_state,
                    revoked_at,
                    revocation_reason,
                    shared_conflict_note,
                    last_reviewed_at
                )
                VALUES (?1, ?2, ?3, NULL, NULL, NULL, NULL)
                ON CONFLICT(record_id) DO NOTHING
                "#,
                params![
                    &record.id,
                    T3Confidence::Medium.as_str(),
                    T3RevocationState::Active.as_str(),
                ],
            )?;
        }

        Ok(())
    }

    pub fn get_record(&self, id: &str) -> Result<Option<MemoryRecord>, RepositoryError> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT
                id,
                source_uri,
                source_kind,
                source_label,
                recorded_at,
                scope,
                record_type,
                truth_layer,
                provenance_json,
                content_text,
                chunk_index,
                chunk_count,
                chunk_anchor_json,
                content_hash,
                valid_from,
                valid_to,
                created_at,
                updated_at
            FROM memory_records
            WHERE id = ?1
            "#,
        )?;

        let mut rows = statement.query([id])?;
        match rows.next()? {
            Some(row) => Ok(Some(map_record_row(row)?)),
            None => Ok(None),
        }
    }

    pub fn list_records(&self) -> Result<Vec<MemoryRecord>, RepositoryError> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT
                id,
                source_uri,
                source_kind,
                source_label,
                recorded_at,
                scope,
                record_type,
                truth_layer,
                provenance_json,
                content_text,
                chunk_index,
                chunk_count,
                chunk_anchor_json,
                content_hash,
                valid_from,
                valid_to,
                created_at,
                updated_at
            FROM memory_records
            ORDER BY recorded_at ASC, id ASC
            "#,
        )?;

        let mut rows = statement.query([])?;
        let mut records = Vec::new();
        while let Some(row) = rows.next()? {
            records.push(map_record_row(row)?);
        }

        Ok(records)
    }

    pub fn get_t3_state(&self, record_id: &str) -> Result<Option<T3State>, RepositoryError> {
        self.conn
            .query_row(
                r#"
                SELECT
                    record_id,
                    confidence,
                    revocation_state,
                    revoked_at,
                    revocation_reason,
                    shared_conflict_note,
                    last_reviewed_at,
                    created_at,
                    updated_at
                FROM truth_t3_state
                WHERE record_id = ?1
                "#,
                [record_id],
                map_t3_state_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn get_truth_record(&self, id: &str) -> Result<Option<TruthRecord>, RepositoryError> {
        let Some(base) = self.get_record(id)? else {
            return Ok(None);
        };

        let truth_record = match base.truth_layer {
            TruthLayer::T1 => TruthRecord::T1 { base },
            TruthLayer::T2 => TruthRecord::T2 { base },
            TruthLayer::T3 => TruthRecord::T3 {
                t3_state: Some(self.get_t3_state(id)?.ok_or_else(|| {
                    RepositoryError::MissingT3State {
                        record_id: id.to_string(),
                    }
                })?),
                base,
            },
        };

        Ok(Some(truth_record))
    }

    pub fn count_records(&self) -> Result<u64, RepositoryError> {
        self.conn
            .query_row("SELECT COUNT(*) FROM memory_records", [], |row| row.get(0))
            .map_err(Into::into)
    }

    pub fn scope_counts(&self) -> Result<Vec<ScopeCount>, RepositoryError> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT scope, COUNT(*)
            FROM memory_records
            GROUP BY scope
            ORDER BY scope
            "#,
        )?;
        let rows = statement.query_map([], |row| {
            let scope = row.get::<_, String>(0)?;
            let count = row.get::<_, u64>(1)?;
            Ok((scope, count))
        })?;

        rows.map(|row| {
            let (scope, count) = row?;
            Ok(ScopeCount {
                scope: parse_scope(&scope)?,
                count,
            })
        })
        .collect()
    }
}

fn map_record_row(row: &rusqlite::Row<'_>) -> Result<MemoryRecord, RepositoryError> {
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
) -> Result<Option<ChunkMetadata>, RepositoryError> {
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
        _ => Err(RepositoryError::IncompleteChunkMetadata {
            record_id: record_id.to_string(),
        }),
    }
}

fn map_t3_state_row(row: &rusqlite::Row<'_>) -> Result<T3State, rusqlite::Error> {
    Ok(T3State {
        record_id: row.get(0)?,
        confidence: T3Confidence::parse(&row.get::<_, String>(1)?).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                1,
                rusqlite::types::Type::Text,
                "invalid t3 confidence".into(),
            )
        })?,
        revocation_state: T3RevocationState::parse(&row.get::<_, String>(2)?).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                2,
                rusqlite::types::Type::Text,
                "invalid t3 revocation state".into(),
            )
        })?,
        revoked_at: row.get(3)?,
        revocation_reason: row.get(4)?,
        shared_conflict_note: row.get(5)?,
        last_reviewed_at: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn parse_source_kind(value: &str) -> Result<SourceKind, RepositoryError> {
    SourceKind::parse(value).ok_or_else(|| RepositoryError::InvalidEnum {
        field: "source_kind",
        value: value.to_string(),
    })
}

fn parse_scope(value: &str) -> Result<Scope, RepositoryError> {
    Scope::parse(value).ok_or_else(|| RepositoryError::InvalidEnum {
        field: "scope",
        value: value.to_string(),
    })
}

fn parse_record_type(value: &str) -> Result<RecordType, RepositoryError> {
    RecordType::parse(value).ok_or_else(|| RepositoryError::InvalidEnum {
        field: "record_type",
        value: value.to_string(),
    })
}

fn parse_truth_layer(value: &str) -> Result<TruthLayer, RepositoryError> {
    TruthLayer::parse(value).ok_or_else(|| RepositoryError::InvalidEnum {
        field: "truth_layer",
        value: value.to_string(),
    })
}
