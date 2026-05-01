use rig::{client::EmbeddingsClient, embeddings::EmbeddingModel};
use thiserror::Error;

use crate::core::config::RootEmbeddingRuntimeConfig;
use crate::core::rig_client::build_embedding_client;

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
        let api_key = self.config.api_key.as_deref().ok_or_else(|| {
            EmbeddingError::RequestBuild("api_key is missing for embedding provider".to_string())
        })?;

        let client = build_embedding_client(api_key, self.config.api_base.as_deref())
            .map_err(EmbeddingError::Provider)?;

        let ndims = self.config.dimensions.unwrap_or(0) as usize;
        let model = client.embedding_model_with_ndims(&self.config.model, ndims);

        let mut results = model
            .embed_texts(vec![text.to_string()])
            .await
            .map_err(|e| EmbeddingError::Provider(e.to_string()))?;

        results
            .pop()
            .map(|emb| emb.vec.into_iter().map(|v| v as f32).collect())
            .ok_or_else(|| EmbeddingError::Provider("empty embedding data".to_string()))
    }

    pub async fn generate_embeddings(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        let api_key = self.config.api_key.as_deref().ok_or_else(|| {
            EmbeddingError::RequestBuild("api_key is missing for embedding provider".to_string())
        })?;

        let client = build_embedding_client(api_key, self.config.api_base.as_deref())
            .map_err(EmbeddingError::Provider)?;

        let ndims = self.config.dimensions.unwrap_or(0) as usize;
        let model = client.embedding_model_with_ndims(&self.config.model, ndims);

        let results = model
            .embed_texts(texts)
            .await
            .map_err(|e| EmbeddingError::Provider(e.to_string()))?;

        Ok(results
            .into_iter()
            .map(|emb| emb.vec.into_iter().map(|v| v as f32).collect())
            .collect())
    }
}
