use serde::Serialize;
use thiserror::Error;

use crate::memory::record::{ChunkAnchor, MemoryRecord, SourceKind, ValidityWindow};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CitationAnchor {
    pub chunk_index: u32,
    pub chunk_count: u32,
    pub anchor: ChunkAnchor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Citation {
    pub record_id: String,
    pub source_uri: String,
    pub source_kind: SourceKind,
    pub source_label: Option<String>,
    pub recorded_at: String,
    pub validity: ValidityWindow,
    pub anchor: CitationAnchor,
}

#[derive(Debug, Error)]
pub enum CitationError {
    #[error("record {record_id} is missing persisted chunk metadata required for citation output")]
    MissingChunkMetadata { record_id: String },
}

impl Citation {
    pub fn from_record(record: &MemoryRecord) -> Result<Self, CitationError> {
        let chunk = record
            .chunk
            .as_ref()
            .ok_or_else(|| CitationError::MissingChunkMetadata {
                record_id: record.id.clone(),
            })?;

        Ok(Self {
            record_id: record.id.clone(),
            source_uri: record.source.uri.clone(),
            source_kind: record.source.kind,
            source_label: record.source.label.clone(),
            recorded_at: record.timestamp.recorded_at.clone(),
            validity: record.validity.clone(),
            anchor: CitationAnchor {
                chunk_index: chunk.chunk_index,
                chunk_count: chunk.chunk_count,
                anchor: chunk.anchor.clone(),
            },
        })
    }
}
