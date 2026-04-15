use anyhow::Result;

use crate::core::config::{Config, RetrievalMode};

#[derive(Debug, Clone)]
pub struct AppContext {
    pub config: Config,
    pub readiness: RuntimeReadiness,
}

impl AppContext {
    pub fn load(config: Config) -> Result<Self> {
        Ok(Self {
            readiness: RuntimeReadiness::from_config(&config),
            config,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeReadiness {
    pub configured_mode: RetrievalMode,
    pub effective_mode: RetrievalMode,
    pub ready: bool,
    pub notes: Vec<String>,
}

impl RuntimeReadiness {
    pub fn from_config(config: &Config) -> Self {
        Self {
            configured_mode: config.retrieval.mode,
            effective_mode: RetrievalMode::LexicalOnly,
            ready: false,
            notes: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::config::{Config, EmbeddingBackend, EmbeddingConfig, RetrievalConfig};

    use super::*;

    #[test]
    fn config_runtime_readiness_preserves_semantic_intent() {
        let lexical = Config {
            db_path: "memory.db".to_string(),
            retrieval: RetrievalConfig {
                mode: RetrievalMode::LexicalOnly,
            },
            embedding: EmbeddingConfig {
                backend: EmbeddingBackend::Disabled,
                model: None,
                endpoint: None,
            },
        };

        let lexical_readiness = RuntimeReadiness::from_config(&lexical);
        assert!(lexical_readiness.ready);
        assert_eq!(lexical_readiness.configured_mode, RetrievalMode::LexicalOnly);
        assert_eq!(lexical_readiness.effective_mode, RetrievalMode::LexicalOnly);

        let embedding_only = Config {
            db_path: "memory.db".to_string(),
            retrieval: RetrievalConfig {
                mode: RetrievalMode::EmbeddingOnly,
            },
            embedding: EmbeddingConfig {
                backend: EmbeddingBackend::Reserved,
                model: Some("future-model".to_string()),
                endpoint: Some("http://localhost:11434".to_string()),
            },
        };

        let embedding_readiness = RuntimeReadiness::from_config(&embedding_only);
        assert!(!embedding_readiness.ready);
        assert_eq!(
            embedding_readiness.configured_mode,
            RetrievalMode::EmbeddingOnly
        );
        assert_eq!(
            embedding_readiness.effective_mode,
            RetrievalMode::EmbeddingOnly
        );
        assert!(!embedding_readiness.notes.is_empty());
    }
}
