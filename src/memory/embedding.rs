use std::time::Duration;

use thiserror::Error;

use crate::core::config::{
    REMOTE_EMBEDDING_DEFAULT_BATCH_SIZE, REMOTE_EMBEDDING_DEFAULT_TIMEOUT_SECONDS,
    REMOTE_EMBEDDING_MAX_RETRIES, RootEmbeddingRuntimeConfig,
};
use crate::core::rig_client::build_embedding_client;

/// Environment variable consulted when `[embedding] api_key` is absent.
/// Keys are never hardcoded; they come from config or the environment.
pub const REMOTE_EMBEDDING_API_KEY_ENV: &str = "AGENT_MEMOS_EMBEDDING_API_KEY";

#[derive(Debug, Error)]
pub enum EmbeddingError {
    #[error("failed to build embedding request: {0}")]
    RequestBuild(String),
    #[error("embedding provider error: {0}")]
    Provider(String),
}

pub struct RigEmbeddingGenerator {
    config: RootEmbeddingRuntimeConfig,
}

impl RigEmbeddingGenerator {
    pub fn new(config: RootEmbeddingRuntimeConfig) -> Self {
        Self { config }
    }

    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let mut results = self.generate_embeddings(vec![text.to_string()]).await?;
        results
            .pop()
            .ok_or_else(|| EmbeddingError::Provider("empty embedding data".to_string()))
    }

    pub async fn generate_embeddings(
        &self,
        texts: Vec<String>,
    ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        let api_key = self.config.api_key.as_deref().ok_or_else(|| {
            EmbeddingError::RequestBuild("api_key is missing for embedding provider".to_string())
        })?;

        let client = build_embedding_client(api_key, self.config.api_base.as_deref())
            .map_err(EmbeddingError::Provider)?;

        // dimensions 为 0 或缺省时不传维度（provider 默认）
        let ndims = self
            .config
            .dimensions
            .filter(|dims| *dims > 0)
            .map(|dims| dims as usize);
        let model = client.embedding(&self.config.model, ndims);

        let response = model
            .call(texts)
            .await
            .map_err(|e| EmbeddingError::Provider(e.to_string()))?;

        Ok(response
            .embeddings
            .into_iter()
            .map(|emb| emb.vec.into_iter().map(|v| v as f32).collect())
            .collect())
    }
}

#[derive(serde::Serialize)]
struct RemoteEmbeddingRequest<'a> {
    model: &'a str,
    input: &'a [String],
}

#[derive(serde::Deserialize)]
struct RemoteEmbeddingResponse {
    #[serde(default)]
    data: Vec<RemoteEmbeddingData>,
}

#[derive(serde::Deserialize)]
struct RemoteEmbeddingData {
    index: Option<usize>,
    embedding: Vec<f32>,
}

/// Embedding generator for any OpenAI-compatible `POST {base_url}/embeddings`
/// endpoint (SiliconFlow `BAAI/bge-m3` is the reference deployment).
///
/// Unlike the rig-backed generator this one owns its transport policy so the
/// grounding constraints are explicit and testable:
/// - input batches are capped (bge-m3 provider cap: 32 texts per request);
/// - transient failures (connection errors, timeouts, 5xx, 429) are retried
///   twice with linear backoff; other 4xx fail fast;
/// - every batch is validated: one vector per input text, dimension
///   consistency across the batch, and against `dimensions` when configured.
pub struct RemoteHttpEmbeddingGenerator {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
    expected_dimensions: Option<usize>,
    batch_size: usize,
    max_retries: u32,
}

impl RemoteHttpEmbeddingGenerator {
    pub fn from_config(config: &RootEmbeddingRuntimeConfig) -> Result<Self, EmbeddingError> {
        let base_url = config
            .api_base
            .as_deref()
            .map(str::trim)
            .map(|url| url.trim_end_matches('/'))
            .filter(|url| !url.is_empty())
            .ok_or_else(|| {
                EmbeddingError::RequestBuild(
                    "api_base is missing for the remote_http embedding backend".to_string(),
                )
            })?
            .to_string();

        let api_key = config
            .api_key
            .as_deref()
            .map(str::trim)
            .filter(|key| !key.is_empty())
            .map(ToString::to_string)
            .or_else(|| {
                std::env::var(REMOTE_EMBEDDING_API_KEY_ENV)
                    .ok()
                    .and_then(|key| {
                        let key = key.trim().to_string();
                        (!key.is_empty()).then_some(key)
                    })
            })
            .ok_or_else(|| {
                EmbeddingError::RequestBuild(format!(
                    "api_key is missing: set [embedding] api_key or the {REMOTE_EMBEDDING_API_KEY_ENV} environment variable"
                ))
            })?;

        let timeout_seconds = config
            .timeout_seconds
            .filter(|secs| *secs > 0)
            .unwrap_or(REMOTE_EMBEDDING_DEFAULT_TIMEOUT_SECONDS);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .build()
            .map_err(|e| EmbeddingError::RequestBuild(format!("reqwest client build: {e}")))?;

        Ok(Self {
            client,
            base_url,
            api_key,
            model: config.model.clone(),
            expected_dimensions: config
                .dimensions
                .filter(|dims| *dims > 0)
                .map(|dims| dims as usize),
            batch_size: config
                .batch_size
                .filter(|size| *size > 0)
                .map(|size| size as usize)
                .unwrap_or(REMOTE_EMBEDDING_DEFAULT_BATCH_SIZE as usize),
            max_retries: REMOTE_EMBEDDING_MAX_RETRIES,
        })
    }

    pub fn batch_size(&self) -> usize {
        self.batch_size
    }

    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let mut results = self.generate_embeddings(vec![text.to_string()]).await?;
        results
            .pop()
            .ok_or_else(|| EmbeddingError::Provider("empty embedding data".to_string()))
    }

    pub async fn generate_embeddings(
        &self,
        texts: Vec<String>,
    ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let mut embeddings = Vec::with_capacity(texts.len());
        for batch in texts.chunks(self.batch_size) {
            let batch = batch.to_vec();
            let batch_embeddings = self.generate_batch_with_retry(batch).await?;
            embeddings.extend(batch_embeddings);
        }
        Ok(embeddings)
    }

    async fn generate_batch_with_retry(
        &self,
        batch: Vec<String>,
    ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        let mut attempt: u32 = 0;
        loop {
            match self.generate_batch(&batch).await {
                Ok(embeddings) => return Ok(embeddings),
                Err(error) if attempt < self.max_retries && error.is_retryable() => {
                    attempt += 1;
                    tokio::time::sleep(Duration::from_millis(250 * u64::from(attempt))).await;
                }
                Err(error) => return Err(error),
            }
        }
    }

    async fn generate_batch(&self, batch: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        let url = format!("{}/embeddings", self.base_url);
        let body = RemoteEmbeddingRequest {
            model: &self.model,
            input: batch,
        };

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| EmbeddingError::Provider(format!("{url} request failed: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            let excerpt: String = body.chars().take(200).collect();
            return Err(EmbeddingError::Provider(format!(
                "{url} returned {status}: {excerpt}"
            )));
        }

        let payload: RemoteEmbeddingResponse = response
            .json()
            .await
            .map_err(|e| EmbeddingError::Provider(format!("{url} response parse failed: {e}")))?;

        let mut indexed: Vec<(usize, Vec<f32>)> = payload
            .data
            .into_iter()
            .map(|item| (item.index.unwrap_or(usize::MAX), item.embedding))
            .collect();
        indexed.sort_by_key(|(index, _)| *index);

        if indexed.len() != batch.len() {
            return Err(EmbeddingError::Provider(format!(
                "{url} returned {} embeddings for {} input texts",
                indexed.len(),
                batch.len()
            )));
        }

        let mut embeddings = Vec::with_capacity(indexed.len());
        let mut expected: Option<usize> = self.expected_dimensions;
        for (position, (_, embedding)) in indexed.into_iter().enumerate() {
            let dimensions = embedding.len();
            if dimensions == 0 {
                return Err(EmbeddingError::Provider(format!(
                    "{url} returned an empty embedding for input {position}"
                )));
            }
            if let Some(expected) = expected {
                if dimensions != expected {
                    return Err(EmbeddingError::Provider(format!(
                        "{url} dimension mismatch for input {position}: expected {expected}, got {dimensions}"
                    )));
                }
            } else {
                expected = Some(dimensions);
            }
            embeddings.push(embedding);
        }

        Ok(embeddings)
    }
}

impl EmbeddingError {
    /// Transient provider conditions worth retrying: connection/timeout
    /// failures, rate limits, and server errors. Authentication/validation
    /// failures (other 4xx) fail fast.
    fn is_retryable(&self) -> bool {
        let EmbeddingError::Provider(message) = self else {
            return false;
        };
        message.contains("request failed:")
            || message.contains("returned 429")
            || message.contains("returned 5")
    }
}
