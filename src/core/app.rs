use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::Result;

use crate::core::config::{Config, EmbeddingBackend, RetrievalMode};

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
            RetrievalMode::LexicalOnly => {
                let mut notes = vec![
                    "lexical_only uses the Phase 2 lexical-first baseline".to_string(),
                    "lexical dependency loading and index readiness are provided by the lexical path after initialization"
                        .to_string(),
                ];

                match (config.embedding.backend, config.embedding.model.as_deref()) {
                    (EmbeddingBackend::Disabled, _) => {}
                    (EmbeddingBackend::Reserved, _) => notes.push(
                        "embedding foundation remains reserved until the optional semantic substrate is implemented"
                            .to_string(),
                    ),
                    (EmbeddingBackend::Builtin, Some(model)) => notes.push(format!(
                        "builtin embedding backend is configured with model '{model}' as an optional second-channel foundation"
                    )),
                    (EmbeddingBackend::Builtin, None) => notes.push(
                        "builtin embedding backend is configured, but no embedding model is set yet"
                            .to_string(),
                    ),
                }

                Self {
                    configured_mode: RetrievalMode::LexicalOnly,
                    effective_mode: RetrievalMode::LexicalOnly,
                    ready: true,
                    notes,
                }
            }
            RetrievalMode::EmbeddingOnly => Self {
                configured_mode: RetrievalMode::EmbeddingOnly,
                effective_mode: RetrievalMode::EmbeddingOnly,
                ready: false,
                notes: vec![
                    embedding_only_note(config),
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
                    hybrid_note(config),
                ],
            },
        }
    }
}

fn embedding_only_note(config: &Config) -> String {
    match (config.embedding.backend, config.embedding.model.as_deref()) {
        (EmbeddingBackend::Disabled, _) => {
            "embedding_only is configured, but a non-disabled embedding backend is required"
                .to_string()
        }
        (EmbeddingBackend::Reserved, _) => {
            "embedding_only is configured, but the embedding substrate is still reserved until a later phase"
                .to_string()
        }
        (EmbeddingBackend::Builtin, Some(model)) => format!(
            "embedding_only is configured with builtin model '{model}', but semantic-primary retrieval remains gated until dual-channel retrieval lands"
        ),
        (EmbeddingBackend::Builtin, None) => {
            "embedding_only is configured for the builtin backend, but no embedding model is set"
                .to_string()
        }
    }
}

fn hybrid_note(config: &Config) -> String {
    match (config.embedding.backend, config.embedding.model.as_deref()) {
        (EmbeddingBackend::Disabled, _) => {
            "hybrid is configured, but an embedding backend is required for the secondary path"
                .to_string()
        }
        (EmbeddingBackend::Reserved, _) => {
            "embedding backend Reserved remains foundation-only until dual-channel retrieval lands"
                .to_string()
        }
        (EmbeddingBackend::Builtin, Some(model)) => format!(
            "builtin embedding model '{model}' is configured, but hybrid fusion remains gated until the dual-channel retrieval phase"
        ),
        (EmbeddingBackend::Builtin, None) => {
            "hybrid is configured for the builtin backend, but no embedding model is set"
                .to_string()
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
    use crate::core::config::{
        Config, EmbeddingBackend, EmbeddingConfig, RetrievalConfig, RootVectorConfig, VectorBackend,
    };

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
            vector: RootVectorConfig {
                backend: VectorBackend::None,
                table: String::new(),
                similarity: String::new(),
            },
            ..Default::default()
        };

        let lexical_readiness = RuntimeReadiness::from_config(&lexical);
        assert!(lexical_readiness.ready);
        assert_eq!(
            lexical_readiness.configured_mode,
            RetrievalMode::LexicalOnly
        );
        assert_eq!(lexical_readiness.effective_mode, RetrievalMode::LexicalOnly);
        assert!(
            lexical_readiness
                .notes
                .iter()
                .any(|note| note.contains("Phase 2 lexical-first baseline")),
            "lexical_only should describe the real Phase 2 lexical baseline: {:?}",
            lexical_readiness.notes
        );
        assert!(
            lexical_readiness.notes.iter().all(|note| !note
                .contains("lexical retrieval dependency loading and index creation are deferred")),
            "lexical_only should reject stale deferred lexical wording: {:?}",
            lexical_readiness.notes
        );

        let embedding_only = Config {
            db_path: "memory.db".to_string(),
            retrieval: RetrievalConfig {
                mode: RetrievalMode::EmbeddingOnly,
            },
            embedding: EmbeddingConfig {
                backend: EmbeddingBackend::Builtin,
                model: Some("hash-64".to_string()),
                endpoint: None,
            },
            vector: RootVectorConfig {
                backend: VectorBackend::SqliteVec,
                table: "object_embeddings_vec".to_string(),
                similarity: "cosine".to_string(),
            },
            ..Default::default()
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
        assert!(
            embedding_readiness
                .notes
                .iter()
                .any(|note| note.contains("semantic-primary retrieval remains gated")),
            "embedding_only should stay explicitly deferred: {:?}",
            embedding_readiness.notes
        );

        let hybrid = Config {
            db_path: "memory.db".to_string(),
            retrieval: RetrievalConfig {
                mode: RetrievalMode::Hybrid,
            },
            embedding: EmbeddingConfig {
                backend: EmbeddingBackend::Builtin,
                model: Some("hash-64".to_string()),
                endpoint: None,
            },
            vector: RootVectorConfig {
                backend: VectorBackend::SqliteVec,
                table: "object_embeddings_vec".to_string(),
                similarity: "cosine".to_string(),
            },
            ..Default::default()
        };

        let hybrid_readiness = RuntimeReadiness::from_config(&hybrid);
        assert!(!hybrid_readiness.ready);
        assert_eq!(hybrid_readiness.configured_mode, RetrievalMode::Hybrid);
        assert_eq!(hybrid_readiness.effective_mode, RetrievalMode::Hybrid);
        assert!(
            hybrid_readiness
                .notes
                .iter()
                .any(|note| note.contains("hybrid fusion remains gated")),
            "hybrid should keep semantic capability deferred: {:?}",
            hybrid_readiness.notes
        );
    }
}
