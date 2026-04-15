use rusqlite::{Connection, params};
use thiserror::Error;

use crate::memory::record::{
    MemoryRecord, Provenance, RecordType, Scope, SourceKind, SourceRef, RecordTimestamp, TruthLayer,
};

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
    InvalidEnum {
        field: &'static str,
        value: String,
    },
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
                created_at,
                updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
            params![
                record.id,
                record.source.uri,
                record.source.kind.as_str(),
                record.source.label,
                record.timestamp.recorded_at,
                record.scope.as_str(),
                record.record_type.as_str(),
                record.truth_layer.as_str(),
                provenance_json,
                record.content_text,
                record.timestamp.created_at,
                record.timestamp.updated_at,
            ],
        )?;

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

    Ok(MemoryRecord {
        id: row.get(0)?,
        source: SourceRef {
            uri: row.get(1)?,
            kind: parse_source_kind(&source_kind)?,
            label: row.get(3)?,
        },
        timestamp: RecordTimestamp {
            recorded_at: row.get(4)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        },
        scope: parse_scope(&scope)?,
        record_type: parse_record_type(&record_type)?,
        truth_layer: parse_truth_layer(&truth_layer)?,
        provenance: serde_json::from_str::<Provenance>(&provenance_json)?,
        content_text: row.get(9)?,
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
