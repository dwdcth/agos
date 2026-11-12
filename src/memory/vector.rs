//! sqlite-vec vec0 acceleration index over `record_embeddings`.
//!
//! Design: the `record_embeddings` JSON table remains the source of truth for
//! every stored vector. When `vector.backend = "sqlite_vec"` is configured,
//! vectors are additionally maintained in a `vec0` virtual table (named by
//! `vector.table`, default `object_embeddings_vec`) and semantic recall goes
//! through a native KNN query instead of the JSON scan.
//!
//! Fallback: if the sqlite-vec extension is unavailable (platform without the
//! statically linked extension) or the vec0 table cannot be used (dimension
//! mismatch from a model switch, corrupted table), callers fall back to the
//! brute-force cosine scan over `record_embeddings.embedding_json` — recall
//! stays correct, only latency grows. See `doc/理论实现对应.md` §3.2.
//!
//! vec0 limitations honored here: virtual tables do not support UPSERT, so an
//! upsert is `DELETE` + `INSERT`; rows for deleted records are not cascaded —
//! stale ids are skipped at read time and cleaned by the next backfill.

use rusqlite::{Connection, params};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VectorIndexError {
    #[error("sqlite-vec extension is not available on this connection")]
    ExtensionUnavailable,
    #[error("vector table {table} query failed: {message}")]
    Query { table: String, message: String },
    #[error("vector table {table} is not usable: {message}")]
    Unusable { table: String, message: String },
}

/// Register the statically linked sqlite-vec extension for every connection
/// opened from now on (process-wide, idempotent).
pub fn register_sqlite_vec_extension() {
    static REGISTER: std::sync::Once = std::sync::Once::new();
    REGISTER.call_once(|| unsafe {
        // Canonical sqlite-vec static registration (mirrors the crate's own
        // tests): the zero-arg init fn is passed through a raw-pointer shim
        // to sqlite3_auto_extension, whose parameter type here is the
        // 3-arg sqlite3 extension entrypoint signature.
        #[allow(clippy::missing_transmute_annotations)]
        rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute(
            sqlite_vec::sqlite3_vec_init as *const (),
        )));
    });
}

/// Serialize an f32 slice as the little-endian blob sqlite-vec expects.
pub fn serialize_f32(vector: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vector.len() * 4);
    for value in vector {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

/// A vec0 virtual table mirroring `record_embeddings` for KNN recall.
pub struct VectorIndex<'db> {
    conn: &'db Connection,
    table: String,
}

impl<'db> VectorIndex<'db> {
    pub fn new(conn: &'db Connection, table: &str) -> Self {
        Self {
            conn,
            table: table.to_string(),
        }
    }

    /// True when the sqlite-vec extension is loaded on this connection.
    pub fn extension_available(&self) -> bool {
        self.conn
            .query_row("SELECT vec_version()", [], |row| row.get::<_, String>(0))
            .is_ok()
    }

    /// Create the vec0 table (if absent) sized for `dimensions` vectors with
    /// cosine distance. Returns `Err(ExtensionUnavailable)` when the extension
    /// is missing so callers can degrade to the JSON scan.
    pub fn ensure_table(&self, dimensions: usize) -> Result<(), VectorIndexError> {
        if !self.extension_available() {
            return Err(VectorIndexError::ExtensionUnavailable);
        }
        if dimensions == 0 {
            return Err(VectorIndexError::Unusable {
                table: self.table.clone(),
                message: "cannot create a vec0 table with zero dimensions".to_string(),
            });
        }
        self.conn
            .execute_batch(&format!(
                "CREATE VIRTUAL TABLE IF NOT EXISTS \"{table}\" USING vec0(\
                    record_id TEXT PRIMARY KEY, \
                    embedding float[{dimensions}] distance_metric=cosine\
                );",
                table = self.table
            ))
            .map_err(|e| VectorIndexError::Query {
                table: self.table.clone(),
                message: e.to_string(),
            })?;
        Ok(())
    }

    /// Whether the vec0 table exists (extension or not).
    pub fn table_exists(&self) -> bool {
        self.conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
                params![self.table],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count != 0)
            .unwrap_or(false)
    }

    /// Insert or replace a vector. The table is created lazily on first write.
    /// Dimension mismatches surface as `Query` errors for the caller to warn
    /// and fall back to the JSON row.
    pub fn upsert(&self, record_id: &str, vector: &[f32]) -> Result<(), VectorIndexError> {
        if !self.extension_available() {
            return Err(VectorIndexError::ExtensionUnavailable);
        }
        if vector.is_empty() {
            return Err(VectorIndexError::Unusable {
                table: self.table.clone(),
                message: format!("record {record_id} has an empty embedding"),
            });
        }
        self.ensure_table(vector.len())?;
        let blob = serialize_f32(vector);
        self.conn
            .execute(
                &format!("DELETE FROM \"{}\" WHERE record_id = ?1", self.table),
                params![record_id],
            )
            .and_then(|_| {
                self.conn.execute(
                    &format!(
                        "INSERT INTO \"{}\" (record_id, embedding) VALUES (?1, ?2)",
                        self.table
                    ),
                    params![record_id, blob],
                )
            })
            .map_err(|e| VectorIndexError::Query {
                table: self.table.clone(),
                message: e.to_string(),
            })?;
        Ok(())
    }

    /// KNN query: the `k` nearest rows with their cosine similarity
    /// (`1 - cosine_distance`). Empty when the extension/table is unusable.
    pub fn knn(&self, query: &[f32], k: usize) -> Result<Vec<(String, f32)>, VectorIndexError> {
        if !self.extension_available() || !self.table_exists() || query.is_empty() || k == 0 {
            return Ok(Vec::new());
        }

        let blob = serialize_f32(query);
        let mut statement = self
            .conn
            .prepare(&format!(
                "SELECT record_id, distance FROM \"{}\" WHERE embedding MATCH ?1 AND k = ?2",
                self.table
            ))
            .map_err(|e| VectorIndexError::Query {
                table: self.table.clone(),
                message: e.to_string(),
            })?;
        let rows = statement
            .query_map(params![blob, k as i64], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, f64>(1).map(|distance| 1.0 - distance as f32)?,
                ))
            })
            .map_err(|e| VectorIndexError::Query {
                table: self.table.clone(),
                message: e.to_string(),
            })?;

        let mut neighbors = Vec::new();
        for row in rows {
            let row = row.map_err(|e| VectorIndexError::Query {
                table: self.table.clone(),
                message: e.to_string(),
            })?;
            neighbors.push(row);
        }
        Ok(neighbors)
    }

    /// All record ids currently present in the vec0 table.
    pub fn list_ids(&self) -> Result<Vec<String>, VectorIndexError> {
        if !self.table_exists() {
            return Ok(Vec::new());
        }
        let mut statement = self
            .conn
            .prepare(&format!("SELECT record_id FROM \"{}\"", self.table))
            .map_err(|e| VectorIndexError::Query {
                table: self.table.clone(),
                message: e.to_string(),
            })?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| VectorIndexError::Query {
                table: self.table.clone(),
                message: e.to_string(),
            })?;
        let mut ids = Vec::new();
        for row in rows {
            ids.push(row.map_err(|e| VectorIndexError::Query {
                table: self.table.clone(),
                message: e.to_string(),
            })?);
        }
        Ok(ids)
    }

    /// Best-effort delete (stale rows after record deletion).
    pub fn delete(&self, record_id: &str) -> Result<(), VectorIndexError> {
        if !self.table_exists() {
            return Ok(());
        }
        self.conn
            .execute(
                &format!("DELETE FROM \"{}\" WHERE record_id = ?1", self.table),
                params![record_id],
            )
            .map_err(|e| VectorIndexError::Query {
                table: self.table.clone(),
                message: e.to_string(),
            })?;
        Ok(())
    }
}
