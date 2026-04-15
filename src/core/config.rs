use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use thiserror::Error;

const DEFAULT_DB_PATH: &str = "~/.agent-memos/agent-memos.db";
const DEFAULT_CONFIG_PATH: &str = "~/.config/agent-memos/config.toml";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalMode {
    #[default]
    LexicalOnly,
    EmbeddingOnly,
    Hybrid,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmbeddingBackend {
    #[default]
    Disabled,
    Reserved,
    Builtin,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct RetrievalConfig {
    pub mode: RetrievalMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct EmbeddingConfig {
    pub backend: EmbeddingBackend,
    pub model: Option<String>,
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub db_path: String,
    pub retrieval: RetrievalConfig,
    pub embedding: EmbeddingConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            db_path: DEFAULT_DB_PATH.to_string(),
            retrieval: RetrievalConfig::default(),
            embedding: EmbeddingConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        Self::load_from(&default_config_path())
    }

    pub fn load_from(path: &Path) -> Result<Self, ConfigError> {
        match fs::read_to_string(path) {
            Ok(contents) => toml::from_str(&contents).map_err(ConfigError::Parse),
            Err(source) if source.kind() == ErrorKind::NotFound => Ok(Self::default()),
            Err(source) => Err(ConfigError::Read {
                path: path.to_path_buf(),
                source,
            }),
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config from {path}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse config TOML")]
    Parse(#[source] toml::de::Error),
}

pub fn default_config_path() -> PathBuf {
    ProjectDirs::from("", "", "agent-memos")
        .map(|dirs| dirs.config_dir().join("config.toml"))
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_PATH))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_missing_path_uses_deterministic_defaults() {
        let path = PathBuf::from("definitely/missing/config.toml");

        let config = Config::load_from(&path).expect("missing path should fall back to defaults");

        assert_eq!(config.retrieval.mode, RetrievalMode::LexicalOnly);
        assert_eq!(config.embedding.backend, EmbeddingBackend::Disabled);
        assert_eq!(config.db_path, DEFAULT_DB_PATH);
    }

    #[test]
    fn config_parses_all_supported_modes() {
        for (mode, backend) in [
            ("lexical_only", "disabled"),
            ("embedding_only", "builtin"),
            ("hybrid", "reserved"),
        ] {
            let config = toml::from_str::<Config>(&format!(
                r#"
db_path = "/tmp/agent-memos.db"

[retrieval]
mode = "{mode}"

[embedding]
backend = "{backend}"
"#
            ))
            .expect("supported mode should parse");

            assert_eq!(config.retrieval.mode, mode.parse_mode());
        }
    }

    #[test]
    fn config_rejects_unknown_mode_strings() {
        let error = toml::from_str::<Config>(
            r#"
db_path = "/tmp/agent-memos.db"

[retrieval]
mode = "keyword_magic"

[embedding]
backend = "disabled"
"#,
        )
        .expect_err("unknown mode should be rejected");

        assert!(error.to_string().contains("keyword_magic"));
    }

    trait ParseMode {
        fn parse_mode(self) -> RetrievalMode;
    }

    impl ParseMode for &str {
        fn parse_mode(self) -> RetrievalMode {
            match self {
                "lexical_only" => RetrievalMode::LexicalOnly,
                "embedding_only" => RetrievalMode::EmbeddingOnly,
                "hybrid" => RetrievalMode::Hybrid,
                other => panic!("unsupported test mode: {other}"),
            }
        }
    }
}
