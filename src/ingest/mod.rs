pub mod chunk;
pub mod detect;
pub mod normalize;

use rusqlite::Connection;
use serde::Serialize;
use thiserror::Error;

use crate::{
    ingest::{
        chunk::{ChunkConfig, chunk_source, to_chunk_metadata},
        detect::{Format, detect_format},
        normalize::{NormalizeError, NormalizedSource, normalize_source},
    },
    memory::{
        record::{
            MemoryRecord, Provenance, RecordTimestamp, RecordType, Scope, SourceKind, SourceRef,
            TruthLayer, ValidityWindow,
        },
        repository::{MemoryRepository, RepositoryError},
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestRequest {
    pub source_uri: String,
    pub source_label: Option<String>,
    pub source_kind: Option<SourceKind>,
    pub content: String,
    pub scope: Scope,
    pub record_type: RecordType,
    pub truth_layer: TruthLayer,
    pub recorded_at: String,
    pub valid_from: Option<String>,
    pub valid_to: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IngestReport {
    pub detected_format: Format,
    pub normalized_source: NormalizedSource,
    pub chunk_count: usize,
    pub record_ids: Vec<String>,
}

#[derive(Debug, Error)]
pub enum IngestError {
    #[error(transparent)]
    Normalize(#[from] NormalizeError),
    #[error(transparent)]
    Persist(#[from] RepositoryError),
}

pub struct IngestService<'db> {
    repository: MemoryRepository<'db>,
    chunk_config: ChunkConfig,
}

impl<'db> IngestService<'db> {
    pub fn new(conn: &'db Connection) -> Self {
        Self::with_chunk_config(conn, ChunkConfig::default())
    }

    pub fn with_chunk_config(conn: &'db Connection, chunk_config: ChunkConfig) -> Self {
        Self {
            repository: MemoryRepository::new(conn),
            chunk_config,
        }
    }

    pub fn ingest(&self, request: IngestRequest) -> Result<IngestReport, IngestError> {
        let detected_format = detect_format(&request.content);
        let normalized_source = normalize_source(&request, detected_format)?;
        let chunks = chunk_source(&normalized_source, self.chunk_config);
        let mut record_ids = Vec::with_capacity(chunks.len());

        for draft in &chunks {
            let anchor_fragment = anchor_fragment(&draft.anchor);
            let record = MemoryRecord {
                id: build_record_id(
                    &normalized_source.canonical_uri,
                    &normalized_source.recorded_at,
                    draft.chunk_index,
                    &draft.content_hash,
                ),
                source: SourceRef {
                    uri: normalized_source.canonical_uri.clone(),
                    kind: normalized_source.source_kind,
                    label: normalized_source.source_label.clone(),
                },
                timestamp: RecordTimestamp {
                    recorded_at: normalized_source.recorded_at.clone(),
                    created_at: normalized_source.recorded_at.clone(),
                    updated_at: normalized_source.recorded_at.clone(),
                },
                scope: normalized_source.scope,
                record_type: normalized_source.record_type,
                truth_layer: normalized_source.truth_layer,
                provenance: Provenance {
                    origin: "ingest".to_string(),
                    imported_via: Some("ingest_service".to_string()),
                    derived_from: vec![format!(
                        "{}#{anchor_fragment}",
                        normalized_source.canonical_uri
                    )],
                },
                content_text: draft.text.clone(),
                chunk: Some(to_chunk_metadata(draft)),
                validity: ValidityWindow {
                    valid_from: normalized_source.valid_from.clone(),
                    valid_to: normalized_source.valid_to.clone(),
                },
            };
            record_ids.push(record.id.clone());
            self.repository.insert_record(&record)?;
        }

        Ok(IngestReport {
            detected_format,
            normalized_source,
            chunk_count: record_ids.len(),
            record_ids,
        })
    }
}

fn build_record_id(
    canonical_uri: &str,
    recorded_at: &str,
    chunk_index: u32,
    content_hash: &str,
) -> String {
    let seed = format!("{canonical_uri}|{recorded_at}|{chunk_index}|{content_hash}");
    let mut hash = 0xcbf29ce484222325u64;

    for byte in seed.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }

    format!("mem-{hash:016x}-{chunk_index:04}")
}

fn anchor_fragment(anchor: &crate::memory::record::ChunkAnchor) -> String {
    match anchor {
        crate::memory::record::ChunkAnchor::CharRange {
            start_char,
            end_char,
        } => format!("char-{start_char}-{end_char}"),
        crate::memory::record::ChunkAnchor::LineRange {
            start_line,
            end_line,
        } => format!("line-{start_line}-{end_line}"),
        crate::memory::record::ChunkAnchor::TurnRange {
            start_turn,
            end_turn,
        } => format!("turn-{start_turn}-{end_turn}"),
    }
}
