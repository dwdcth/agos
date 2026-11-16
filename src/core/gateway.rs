//! Unified HTTP model gateway client for `agent-memos` (`agos`).
//!
//! Abstracts over local model gateway services (`pa-modeld` / Ollama / vLLM /
//! llama-server) and standard cloud OpenAI-compatible providers using HTTP APIs.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use crate::core::config::{EmbeddingBackend, RootEmbeddingRuntimeConfig};
use crate::core::jev::{JevClientConfig, JevClientError, JevHttpClient, normalize_gateway_root};
use crate::core::rig_client::{
    CallUsage, CompletionsHandle, StructuredCompletion, build_completions_client,
};
use crate::memory::embedding::{EmbeddingError, RemoteHttpEmbeddingGenerator};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelProtocol {
    #[default]
    OpenAi,
    Jev,
    Ollama,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelEndpointConfig {
    pub base_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    pub model: String,
    #[serde(default)]
    pub protocol: ModelProtocol,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<u64>,
}

/// Unified HTTP model gateway supporting Chat Completions, Embeddings, and Jev decisions.
#[derive(Clone)]
pub struct ModelGatewayClient {
    completions: CompletionsHandle,
    embeddings: Option<RemoteHttpEmbeddingGenerator>,
    jev: JevHttpClient,
    default_model: String,
}

impl ModelGatewayClient {
    pub fn new(endpoint: &ModelEndpointConfig) -> Result<Self, String> {
        let root = normalize_gateway_root(&endpoint.base_url);
        let openai_base = format!("{root}/v1");
        let api_key = endpoint.api_key.as_deref().unwrap_or("local-no-key");
        let completions =
            build_completions_client(api_key, Some(&openai_base), endpoint.timeout_seconds)?;
        let jev = JevHttpClient::new(&JevClientConfig {
            base_url: root.clone(),
            api_key: endpoint.api_key.clone(),
            model: Some(endpoint.model.clone()),
            timeout_seconds: endpoint.timeout_seconds,
        })
        .map_err(|e| e.to_string())?;
        let embeddings = RemoteHttpEmbeddingGenerator::from_config(&RootEmbeddingRuntimeConfig {
            provider: EmbeddingBackend::RemoteHttp.as_str().to_string(),
            model: endpoint.model.clone(),
            api_base: Some(openai_base),
            api_key: Some(api_key.to_string()),
            dimensions: None,
            batch_size: None,
            timeout_seconds: endpoint.timeout_seconds,
        })
        .ok();

        Ok(Self {
            completions,
            embeddings,
            jev,
            default_model: endpoint.model.clone(),
        })
    }

    pub fn jev(&self) -> &JevHttpClient {
        &self.jev
    }

    pub fn completions(&self) -> &CompletionsHandle {
        &self.completions
    }

    pub async fn complete_text(
        &self,
        model: Option<&str>,
        system: &str,
        prompt: &str,
    ) -> Result<(String, CallUsage), String> {
        self.completions
            .complete_text_with_usage(StructuredCompletion {
                model: model.unwrap_or(&self.default_model).to_string(),
                preamble: system.to_string(),
                prompt: prompt.to_string(),
                temperature: None,
                max_tokens: None,
                additional_params: None,
                image: None,
            })
            .await
    }

    pub async fn complete_json<T>(
        &self,
        model: Option<&str>,
        system: &str,
        prompt: &str,
    ) -> Result<(T, CallUsage), String>
    where
        T: JsonSchema + DeserializeOwned + Send + Sync,
    {
        self.completions
            .complete_json_with_usage(StructuredCompletion {
                model: model.unwrap_or(&self.default_model).to_string(),
                preamble: system.to_string(),
                prompt: prompt.to_string(),
                temperature: None,
                max_tokens: None,
                additional_params: None,
                image: None,
            })
            .await
    }

    pub async fn embed(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        let emb_gen = self.embeddings.as_ref().ok_or_else(|| {
            EmbeddingError::RequestBuild("embedding gateway not initialized".into())
        })?;
        emb_gen.generate_embeddings(texts).await
    }

    pub async fn list_models(&self) -> Result<Vec<String>, JevClientError> {
        self.jev.list_models().await
    }
}
