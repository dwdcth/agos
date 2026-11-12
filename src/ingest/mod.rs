pub mod chunk;
pub mod detect;
pub mod normalize;

use rusqlite::Connection;
use serde::Serialize;
use thiserror::Error;
use tracing::warn;

use crate::{
    core::config::{
        Config, EmbeddingBackend, EmbeddingConfig, MemoryConfig, MemorySummaryBackend,
        RootLlmConfig, VectorBackend,
    },
    ingest::{
        chunk::{ChunkConfig, chunk_source, to_chunk_metadata},
        detect::{Format, detect_format},
        normalize::{NormalizeError, NormalizedSource, normalize_source},
    },
    memory::{
        calibrated::CalibratedTaxonomyClassifier,
        classifier::KeywordTaxonomyClassifier,
        embedding::{RemoteHttpEmbeddingGenerator, RigEmbeddingGenerator},
        pipeline::{DefaultMemoryPipeline, MemoryPipelineError},
        record::{
            MemoryRecord, Provenance, RecordTimestamp, RecordType, Scope, SourceKind, SourceRef,
            TruthLayer, ValidityWindow,
        },
        repository::{MemoryRepository, RecordEmbedding, RepositoryError},
        store::{FactDslStore, PersistedFactDslRecordV1},
        summary::{FactSummaryGenerator, RigSummaryGenerator, RuleBasedSummaryGenerator},
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
    #[error(transparent)]
    LayeredMemory(#[from] crate::memory::pipeline::MemoryPipelineError),
}

pub struct IngestService<'db> {
    repository: MemoryRepository<'db>,
    chunk_config: ChunkConfig,
    embedding_config: EmbeddingConfig,
    memory_config: MemoryConfig,
    llm_config: RootLlmConfig,
    summary_preamble: Option<String>,
    /// 可校准统计分类层（`[memory] calibrated_classifier = true` 时从库内
    /// DSL sidecar 训练；训练失败降级为纯关键词分类，ingest 不因它失败）。
    calibrated_classifier: Option<CalibratedTaxonomyClassifier>,
}

impl<'db> IngestService<'db> {
    pub fn new(conn: &'db Connection) -> Self {
        Self::with_components(
            conn,
            ChunkConfig::default(),
            EmbeddingConfig::default(),
            MemoryConfig::default(),
            RootLlmConfig::default(),
            None,
            None,
        )
    }

    pub fn with_chunk_config(conn: &'db Connection, chunk_config: ChunkConfig) -> Self {
        Self::with_components(
            conn,
            chunk_config,
            EmbeddingConfig::default(),
            MemoryConfig::default(),
            RootLlmConfig::default(),
            None,
            None,
        )
    }

    pub fn with_embedding_config(
        conn: &'db Connection,
        chunk_config: ChunkConfig,
        embedding_config: EmbeddingConfig,
    ) -> Self {
        Self::with_components(
            conn,
            chunk_config,
            embedding_config,
            MemoryConfig::default(),
            RootLlmConfig::default(),
            None,
            None,
        )
    }

    pub fn with_config(conn: &'db Connection, config: &Config) -> Self {
        Self::with_components(
            conn,
            ChunkConfig::default(),
            config.embedding.clone(),
            config.memory.clone(),
            config.llm.clone(),
            config.prompts.summary_preamble.clone(),
            vector_table_name(&config.vector),
        )
    }

    fn with_components(
        conn: &'db Connection,
        chunk_config: ChunkConfig,
        embedding_config: EmbeddingConfig,
        memory_config: MemoryConfig,
        llm_config: RootLlmConfig,
        summary_preamble: Option<String>,
        vector_table: Option<String>,
    ) -> Self {
        let repository = MemoryRepository::with_vector_table(conn, vector_table.as_deref());
        let calibrated_classifier = if memory_config.calibrated_classifier {
            match CalibratedTaxonomyClassifier::fit_from_repository(&repository) {
                Ok(classifier) => {
                    if !classifier.statistical_available() {
                        warn!(
                            "calibrated_classifier enabled but labeled DSL records below \
                             the minimum; keyword rules stay authoritative"
                        );
                    }
                    Some(classifier)
                }
                Err(error) => {
                    warn!(
                        error = %error,
                        "calibrated_classifier fit failed; falling back to keyword rules"
                    );
                    None
                }
            }
        } else {
            None
        };

        Self {
            repository,
            chunk_config,
            embedding_config,
            memory_config,
            llm_config,
            summary_preamble,
            calibrated_classifier,
        }
    }

    pub fn ingest(&self, request: IngestRequest) -> Result<IngestReport, IngestError> {
        let detected_format = detect_format(&request.content);
        let normalized_source = normalize_source(&request, detected_format)?;
        let chunks = chunk_source(&normalized_source, self.chunk_config);
        let mut record_ids = Vec::with_capacity(chunks.len());
        let mut records = Vec::with_capacity(chunks.len());

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
            self.persist_layered_memory(&record)?;
            records.push(record);
        }

        // Embedding phase runs after every record is durable: a provider
        // outage must degrade recall, never lose the ingested memory itself.
        for embedding in self.generate_record_embeddings(&records) {
            self.repository.insert_record_embedding(&embedding)?;
        }

        Ok(IngestReport {
            detected_format,
            normalized_source,
            chunk_count: record_ids.len(),
            record_ids,
        })
    }
}

impl IngestService<'_> {
    fn persist_layered_memory(&self, record: &MemoryRecord) -> Result<(), IngestError> {
        let backend = self
            .memory_config
            .effective_summary_backend(&self.llm_config);

        match backend {
            MemorySummaryBackend::RuleBased => self.persist_layered_memory_rule_based(record),
            MemorySummaryBackend::Rig => match self.persist_layered_memory_rig(record) {
                Ok(()) => Ok(()),
                Err(error) if self.memory_config.summary_backend == MemorySummaryBackend::Auto => {
                    warn!(
                        error = %error,
                        record_id = %record.id,
                        "rig summary generation failed during ingest; falling back to rule-based summary"
                    );
                    self.persist_layered_memory_rule_based(record)
                }
                Err(error) => Err(error),
            },
            MemorySummaryBackend::Auto => unreachable!("auto should resolve to a concrete backend"),
        }
    }

    fn persist_layered_memory_rule_based(&self, record: &MemoryRecord) -> Result<(), IngestError> {
        match self.calibrated_classifier.as_ref() {
            // 默认路径保持逐字节等价：纯关键词分类器 + 规则摘要
            None => {
                let persisted = DefaultMemoryPipeline::default_v1()
                    .build_persisted_sync_from_memory_record(record)?;
                self.put_persisted_fact_dsl(&persisted)
            }
            // 统计层开启：同一管线的分类步换成"关键词优先 + 贝叶斯兜底"
            Some(calibrated) => {
                let classification = calibrated
                    .classify_record_sync(record)
                    .map_err(MemoryPipelineError::Classification)?;
                let summary_input = classification
                    .clone()
                    .into_summary_input(
                        record.truth_layer,
                        &record.source.uri,
                        &record.content_text,
                    )
                    .map_err(MemoryPipelineError::Classification)?;
                let draft = RuleBasedSummaryGenerator
                    .summarize_sync(&summary_input)
                    .map_err(MemoryPipelineError::Summary)?;
                let built = summary_input
                    .into_record(draft)
                    .map_err(MemoryPipelineError::Summary)?;
                let mut persisted =
                    PersistedFactDslRecordV1::from_fact_dsl_record(&record.id, &built)
                        .map_err(MemoryPipelineError::Store)?;
                persisted.classification_confidence = Some(classification.confidence);
                persisted.needs_review = classification.needs_review;
                persisted.validate().map_err(MemoryPipelineError::Store)?;
                self.put_persisted_fact_dsl(&persisted)
            }
        }
    }

    fn persist_layered_memory_rig(&self, record: &MemoryRecord) -> Result<(), IngestError> {
        let classification = match self.calibrated_classifier.as_ref() {
            Some(calibrated) => calibrated
                .classify_record_sync(record)
                .map_err(MemoryPipelineError::Classification)?,
            None => KeywordTaxonomyClassifier::default()
                .classify_record_sync(record)
                .map_err(MemoryPipelineError::Classification)?,
        };
        let summary_input = classification
            .clone()
            .into_summary_input(record.truth_layer, &record.source.uri, &record.content_text)
            .map_err(MemoryPipelineError::Classification)?;
        let draft = self.block_on_summary(
            RigSummaryGenerator::from_llm_config_with_preamble(
                self.llm_config.clone(),
                self.summary_preamble.clone(),
            )
            .summarize(&summary_input),
        )?;
        let built = summary_input
            .into_record(draft)
            .map_err(MemoryPipelineError::Summary)?;
        let mut persisted = crate::memory::store::PersistedFactDslRecordV1::from_fact_dsl_record(
            &record.id, &built,
        )
        .map_err(MemoryPipelineError::Store)?;
        persisted.classification_confidence = Some(classification.confidence);
        persisted.needs_review = classification.needs_review;
        persisted.validate().map_err(MemoryPipelineError::Store)?;
        self.put_persisted_fact_dsl(&persisted)
    }

    fn block_on_summary<F>(
        &self,
        future: F,
    ) -> Result<crate::memory::dsl::FactDslDraft, IngestError>
    where
        F: std::future::Future<
                Output = Result<
                    crate::memory::dsl::FactDslDraft,
                    crate::memory::summary::FactSummaryError,
                >,
            > + Send,
    {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            return tokio::task::block_in_place(|| handle.block_on(future))
                .map_err(|error| IngestError::LayeredMemory(MemoryPipelineError::Summary(error)));
        }

        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| {
                IngestError::LayeredMemory(MemoryPipelineError::Summary(
                    crate::memory::summary::FactSummaryError::Generator(format!(
                        "failed to create runtime for rig summary generation: {error}"
                    )),
                ))
            })?
            .block_on(future)
            .map_err(|error| IngestError::LayeredMemory(MemoryPipelineError::Summary(error)))
    }

    fn put_persisted_fact_dsl(
        &self,
        persisted: &crate::memory::store::PersistedFactDslRecordV1,
    ) -> Result<(), IngestError> {
        match self.repository.put_fact_dsl(persisted) {
            Ok(()) => Ok(()),
            Err(crate::memory::store::FactDslStoreError::Sqlite(source)) => {
                Err(IngestError::Persist(RepositoryError::Sqlite(source)))
            }
            Err(other) => Err(IngestError::LayeredMemory(MemoryPipelineError::Store(
                other,
            ))),
        }
    }

    /// Generate embeddings for a batch of freshly ingested records.
    ///
    /// - `builtin` hashes locally (offline);
    /// - `openai` keeps the historical per-record rig call;
    /// - `remote_http` batches all chunk texts through the OpenAI-compatible
    ///   /embeddings endpoint (provider batch cap applied inside the
    ///   generator, retries and dimension validation included).
    ///
    /// Provider failures are non-fatal: the records stay ingested and the
    /// embedding channel simply has no vectors for them (warned).
    fn generate_record_embeddings(&self, records: &[MemoryRecord]) -> Vec<RecordEmbedding> {
        match self.embedding_config.backend {
            EmbeddingBackend::Disabled | EmbeddingBackend::Reserved => Vec::new(),
            EmbeddingBackend::Builtin => records
                .iter()
                .filter_map(|record| self.builtin_record_embedding(record))
                .collect(),
            EmbeddingBackend::OpenAi => records
                .iter()
                .filter_map(|record| self.remote_record_embedding(record))
                .collect(),
            EmbeddingBackend::RemoteHttp => self.remote_batch_record_embeddings(records),
        }
    }

    fn builtin_record_embedding(&self, record: &MemoryRecord) -> Option<RecordEmbedding> {
        let model = self.embedding_config.model.as_ref()?;
        let dimensions = parse_builtin_dimensions(model);
        let embedding = builtin_embedding(&record.content_text, dimensions);
        let timestamp = record.timestamp.recorded_at.clone();
        let source_text_hash = record
            .chunk
            .as_ref()
            .map(|chunk| chunk.content_hash.clone())
            .unwrap_or_else(|| "inline-text".to_string());

        Some(RecordEmbedding {
            record_id: record.id.clone(),
            backend: EmbeddingBackend::Builtin,
            model: model.clone(),
            dimensions: dimensions as u32,
            embedding,
            source_text_hash,
            created_at: timestamp.clone(),
            updated_at: timestamp,
        })
    }

    /// Per-record remote embedding via the rig OpenAI client (legacy path).
    fn remote_record_embedding(&self, record: &MemoryRecord) -> Option<RecordEmbedding> {
        let model = self.embedding_config.model.as_ref()?;
        let runtime_config = crate::core::config::RootEmbeddingRuntimeConfig {
            provider: "openai".to_string(),
            model: model.clone(),
            api_base: self.embedding_config.endpoint.clone(),
            api_key: self.embedding_config.api_key.clone(),
            ..Default::default()
        };

        let generator = RigEmbeddingGenerator::new(runtime_config);
        let embedding = self
            .block_on_embedding(generator.generate_embedding(&record.content_text))
            .map_err(|e| {
                warn!(error = %e, record_id = %record.id, "failed to generate openai embedding");
                e
            })
            .ok()?;

        Some(self.build_record_embedding_record(
            record,
            EmbeddingBackend::OpenAi,
            model.clone(),
            embedding,
        ))
    }

    /// Batched remote embeddings for the whole ingest (remote_http backend).
    fn remote_batch_record_embeddings(&self, records: &[MemoryRecord]) -> Vec<RecordEmbedding> {
        if records.is_empty() {
            return Vec::new();
        }
        let runtime_config = self.embedding_config.to_runtime_config();
        let generator = match RemoteHttpEmbeddingGenerator::from_config(&runtime_config) {
            Ok(generator) => generator,
            Err(error) => {
                warn!(
                    error = %error,
                    "remote_http embedding backend is misconfigured; skipping embedding phase"
                );
                return Vec::new();
            }
        };

        let texts = records
            .iter()
            .map(|record| record.content_text.clone())
            .collect::<Vec<_>>();
        let embeddings = self
            .block_on_embedding_vec(generator.generate_embeddings(texts))
            .map_err(|error| {
                warn!(
                    error = %error,
                    record_count = records.len(),
                    "remote embedding generation failed during ingest; records stay ingested without vectors"
                );
                error
            })
            .ok()
            .unwrap_or_default();

        records
            .iter()
            .zip(embeddings)
            .map(|(record, embedding)| {
                self.build_record_embedding_record(
                    record,
                    EmbeddingBackend::RemoteHttp,
                    runtime_config.model.clone(),
                    embedding,
                )
            })
            .collect()
    }

    fn build_record_embedding_record(
        &self,
        record: &MemoryRecord,
        backend: EmbeddingBackend,
        model: String,
        embedding: Vec<f32>,
    ) -> RecordEmbedding {
        let timestamp = record.timestamp.recorded_at.clone();
        let source_text_hash = record
            .chunk
            .as_ref()
            .map(|chunk| chunk.content_hash.clone())
            .unwrap_or_else(|| "inline-text".to_string());

        RecordEmbedding {
            record_id: record.id.clone(),
            backend,
            model,
            dimensions: embedding.len() as u32,
            embedding,
            source_text_hash,
            created_at: timestamp.clone(),
            updated_at: timestamp,
        }
    }

    fn block_on_embedding<F>(&self, future: F) -> Result<Vec<f32>, IngestError>
    where
        F: std::future::Future<Output = Result<Vec<f32>, crate::memory::embedding::EmbeddingError>>
            + Send,
    {
        self.block_on_runtime(future)
    }

    fn block_on_embedding_vec<F>(&self, future: F) -> Result<Vec<Vec<f32>>, IngestError>
    where
        F: std::future::Future<
                Output = Result<Vec<Vec<f32>>, crate::memory::embedding::EmbeddingError>,
            > + Send,
    {
        self.block_on_runtime(future)
    }

    fn block_on_runtime<F, T>(&self, future: F) -> Result<T, IngestError>
    where
        F: std::future::Future<Output = Result<T, crate::memory::embedding::EmbeddingError>> + Send,
        T: Send,
    {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            return tokio::task::block_in_place(|| handle.block_on(future)).map_err(|e| {
                IngestError::LayeredMemory(MemoryPipelineError::Summary(
                    crate::memory::summary::FactSummaryError::Generator(e.to_string()),
                ))
            });
        }

        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| {
                IngestError::LayeredMemory(MemoryPipelineError::Summary(
                    crate::memory::summary::FactSummaryError::Generator(format!(
                        "failed to create runtime for embedding generation: {e}"
                    )),
                ))
            })?
            .block_on(future)
            .map_err(|e| {
                IngestError::LayeredMemory(MemoryPipelineError::Summary(
                    crate::memory::summary::FactSummaryError::Generator(e.to_string()),
                ))
            })
    }
}

/// The vec0 table name for a vector config, or `None` when the config does
/// not select the sqlite-vec backend (or names an empty table). Host crates
/// wiring ingest/search/backfill against the same vector config share this
/// helper so the table-name rule lives in one place.
pub fn vector_table_name(vector: &crate::core::config::RootVectorConfig) -> Option<String> {
    (vector.backend == VectorBackend::SqliteVec)
        .then(|| vector.table.clone())
        .filter(|table| !table.trim().is_empty())
}

fn parse_builtin_dimensions(model: &str) -> usize {
    model
        .rsplit('-')
        .next()
        .and_then(|suffix| suffix.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(16)
}

fn builtin_embedding(text: &str, dimensions: usize) -> Vec<f32> {
    let mut values = vec![0.0f32; dimensions];
    for (index, byte) in text.bytes().enumerate() {
        let slot = (usize::from(byte) + index) % dimensions;
        values[slot] += f32::from(byte) / 255.0;
    }

    let magnitude = values.iter().map(|value| value * value).sum::<f32>().sqrt();
    if magnitude > 0.0 {
        for value in &mut values {
            *value /= magnitude;
        }
    }

    values
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
