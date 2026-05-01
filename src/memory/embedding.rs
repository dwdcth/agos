use rig::providers::openai;
use rig::embeddings::EmbeddingsBuilder;
use rig::client::EmbeddingsClient;
use rig::http_client::{HeaderMap, HeaderValue};
use thiserror::Error;

use crate::core::config::RootEmbeddingRuntimeConfig;

#[derive(Debug, Error)]
pub enum EmbeddingError {
    #[error("failed to build openai-compatible rig client: {0}")]
    ClientBuild(String),
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
        let api_key = self.config.api_key.as_deref().ok_or_else(|| {
            EmbeddingError::ClientBuild("api_key is missing for openai embedding provider".to_string())
        })?;

        let mut headers = HeaderMap::new();
        headers.insert(
            "User-Agent",
            HeaderValue::from_static("agent-memos/0.1.0"),
        );

        let mut builder = openai::Client::builder()
            .api_key(api_key)
            .http_headers(headers);

        if let Some(api_base) = &self.config.api_base {
            builder = builder.base_url(api_base);
        }

        let client = builder.build().map_err(|e| EmbeddingError::ClientBuild(e.to_string()))?;
        let model = client.embedding_model(&self.config.model);

        let embeddings = EmbeddingsBuilder::new(model)
            .document(text.to_string())
            .map_err(|e| EmbeddingError::Provider(e.to_string()))?
            .build()
            .await
            .map_err(|e| EmbeddingError::Provider(e.to_string()))?;

        // Extract the first embedding vector from (String, OneOrMany<Embedding>)
        let (_, one_or_many) = embeddings.into_iter().next().ok_or_else(|| {
            EmbeddingError::Provider("empty embedding data in response".to_string())
        })?;
        
        Ok(one_or_many.first().vec.into_iter().map(|v| v as f32).collect())
    }

    pub async fn generate_embeddings(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        let api_key = self.config.api_key.as_deref().ok_or_else(|| {
            EmbeddingError::ClientBuild("api_key is missing for openai embedding provider".to_string())
        })?;

        let mut headers = HeaderMap::new();
        headers.insert(
            "User-Agent",
            HeaderValue::from_static("agent-memos/0.1.0"),
        );

        let mut builder = openai::Client::builder()
            .api_key(api_key)
            .http_headers(headers);

        if let Some(api_base) = &self.config.api_base {
            builder = builder.base_url(api_base);
        }

        let client = builder.build().map_err(|e| EmbeddingError::ClientBuild(e.to_string()))?;
        let model = client.embedding_model(&self.config.model);

        let mut builder = EmbeddingsBuilder::new(model);
        for text in texts {
            builder = builder.document(text).map_err(|e| EmbeddingError::Provider(e.to_string()))?;
        }

        let embeddings = builder
            .build()
            .await
            .map_err(|e| EmbeddingError::Provider(e.to_string()))?;

        Ok(embeddings
            .into_iter()
            .map(|(_, one_or_many)| one_or_many.first().vec.into_iter().map(|v| v as f32).collect())
            .collect())
    }
}
