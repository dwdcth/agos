use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::Result;

use crate::core::config::{Config, RetrievalMode};

#[derive(Debug, Clone)]
pub struct AppContext {
    pub config: Config,
    pub readiness: RuntimeReadiness,
    db_path: PathBuf,
}

impl AppContext {
    pub fn load(config: Config) -> Result<Self> {
        Ok(Self {
            readiness: RuntimeReadiness::from_config(&config),
            db_path: resolve_home_path(&config.db_path),
            config,
        })
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
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
        match config.retrieval.mode {
            RetrievalMode::LexicalOnly => Self {
                configured_mode: RetrievalMode::LexicalOnly,
                effective_mode: RetrievalMode::LexicalOnly,
                ready: true,
                notes: vec![
                    "lexical_only remains the Phase 1 foundation default".to_string(),
                    "lexical retrieval dependency loading and index creation are deferred to a later phase"
                        .to_string(),
                ],
            },
            RetrievalMode::EmbeddingOnly => Self {
                configured_mode: RetrievalMode::EmbeddingOnly,
                effective_mode: RetrievalMode::EmbeddingOnly,
                ready: false,
                notes: vec![
                    format!(
                        "embedding_only is configured, but backend {:?} is reserved for a later phase",
                        config.embedding.backend
                    ),
                    "semantic retrieval remains explicit instead of collapsing to a boolean flag"
                        .to_string(),
                ],
            },
            RetrievalMode::Hybrid => Self {
                configured_mode: RetrievalMode::Hybrid,
                effective_mode: RetrievalMode::Hybrid,
                ready: false,
                notes: vec![
                    "hybrid is configured with lexical as the primary explanation channel"
                        .to_string(),
                    format!(
                        "embedding backend {:?} is reserved until semantic retrieval lands",
                        config.embedding.backend
                    ),
                ],
            },
        }
    }
}

fn resolve_home_path(path: &str) -> PathBuf {
    match path.strip_prefix("~/") {
        Some(suffix) => env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join(suffix))
            .unwrap_or_else(|| PathBuf::from(path)),
        None => PathBuf::from(path),
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
