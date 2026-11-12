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
    OpenAi,
    RemoteHttp,
}

impl EmbeddingBackend {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Reserved => "reserved",
            Self::Builtin => "builtin",
            Self::OpenAi => "openai",
            Self::RemoteHttp => "remote_http",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct RetrievalConfig {
    pub mode: RetrievalMode,
}

/// Remote-HTTP embedding defaults sized for the SiliconFlow BAAI/bge-m3
/// deployment: the provider caps a single /embeddings request at 32 input
/// texts, answers in ~1024 dimensions, and retries absorb transient 5xx/429.
pub const REMOTE_EMBEDDING_DEFAULT_BATCH_SIZE: u32 = 32;
pub const REMOTE_EMBEDDING_DEFAULT_TIMEOUT_SECONDS: u64 = 30;
pub const REMOTE_EMBEDDING_MAX_RETRIES: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct EmbeddingConfig {
    pub backend: EmbeddingBackend,
    pub model: Option<String>,
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
    /// Expected vector dimensions (bge-m3: 1024). When set, generated
    /// embeddings are validated against it before being persisted.
    pub dimensions: Option<u32>,
    /// Max texts per remote /embeddings request (provider cap: 32).
    pub batch_size: Option<u32>,
    /// Per-request timeout in seconds for the remote embedding endpoint.
    pub timeout_seconds: Option<u64>,
}

impl EmbeddingConfig {
    /// Flatten into the runtime embedding config used by generators.
    pub fn to_runtime_config(&self) -> RootEmbeddingRuntimeConfig {
        RootEmbeddingRuntimeConfig {
            provider: self.backend.as_str().to_string(),
            model: self.model.clone().unwrap_or_default(),
            api_base: self.endpoint.clone(),
            api_key: self.api_key.clone(),
            dimensions: self.dimensions,
            batch_size: self.batch_size,
            timeout_seconds: self.timeout_seconds,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MemorySummaryBackend {
    #[default]
    Auto,
    RuleBased,
    Rig,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct PromptsConfig {
    pub summary_preamble: Option<String>,
    pub simulation_preamble: Option<String>,
    pub agent_preamble: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct MemoryConfig {
    pub summary_backend: MemorySummaryBackend,
}

impl MemoryConfig {
    pub fn effective_summary_backend(&self, llm: &RootLlmConfig) -> MemorySummaryBackend {
        match self.summary_backend {
            MemorySummaryBackend::Auto if llm.is_configured() => MemorySummaryBackend::Rig,
            MemorySummaryBackend::Auto => MemorySummaryBackend::RuleBased,
            backend => backend,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct CognitionConfig {
    /// Whether the default `agent-search` path automatically feeds its report
    /// into the rumination loop (enqueue trigger + drain short-cycle) after
    /// execution. On by default; the daemon can disable it to own scheduling.
    pub auto_rumination: bool,
}

impl Default for CognitionConfig {
    fn default() -> Self {
        Self {
            auto_rumination: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub db_path: String,
    pub dict_path: Option<String>,
    pub retrieval: RetrievalConfig,
    pub embedding: EmbeddingConfig,
    pub llm: RootLlmConfig,
    pub memory: MemoryConfig,
    pub cognition: CognitionConfig,
    pub vector: RootVectorConfig,
    pub prompts: PromptsConfig,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct RootRuntimeConfig {
    pub store: RootStoreConfig,
    pub llm: RootLlmConfig,
    pub embedding: RootEmbeddingRuntimeConfig,
    pub vector: RootVectorConfig,
    pub cognition: CognitionConfig,
    pub prompts: PromptsConfig,
}

impl RootRuntimeConfig {
    pub fn load_from(path: &Path) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        toml::from_str(&contents).map_err(ConfigError::Parse)
    }

    pub fn to_config(&self) -> Config {
        Config {
            db_path: self.store.sqlite_path.clone(),
            dict_path: self.store.dict_path.clone(),
            retrieval: RetrievalConfig {
                mode: RetrievalMode::LexicalOnly, // Default to lexical for safety
            },
            embedding: EmbeddingConfig {
                backend: match self.embedding.provider.as_str() {
                    "openai" => EmbeddingBackend::OpenAi,
                    "builtin" => EmbeddingBackend::Builtin,
                    "remote_http" => EmbeddingBackend::RemoteHttp,
                    _ => EmbeddingBackend::Disabled,
                },
                model: Some(self.embedding.model.clone()),
                endpoint: self.embedding.api_base.clone(),
                api_key: self.embedding.api_key.clone(),
                dimensions: self.embedding.dimensions,
                batch_size: self.embedding.batch_size,
                timeout_seconds: self.embedding.timeout_seconds,
            },
            llm: self.llm.clone(),
            memory: MemoryConfig::default(),
            cognition: self.cognition.clone(),
            vector: self.vector.clone(),
            prompts: self.prompts.clone(),
        }
    }

    pub fn retrieval_mode_variants(&self) -> Vec<RetrievalModeVariant> {
        let base = self.to_config();
        vec![
            RetrievalModeVariant {
                name: "lexical_only".to_string(),
                db_path: self.store.sqlite_path.clone(),
                mode: RetrievalMode::LexicalOnly,
                embedding_backend: EmbeddingBackend::Disabled,
                llm: self.llm.clone(),
                embedding: Some(self.embedding.clone()),
                vector: Some(self.vector.clone()),
            },
            RetrievalModeVariant {
                name: "embedding_only".to_string(),
                db_path: self.store.sqlite_path.clone(),
                mode: RetrievalMode::EmbeddingOnly,
                embedding_backend: base.embedding.backend,
                llm: self.llm.clone(),
                embedding: Some(self.embedding.clone()),
                vector: Some(self.vector.clone()),
            },
            RetrievalModeVariant {
                name: "hybrid".to_string(),
                db_path: self.store.sqlite_path.clone(),
                mode: RetrievalMode::Hybrid,
                embedding_backend: base.embedding.backend,
                llm: self.llm.clone(),
                embedding: Some(self.embedding.clone()),
                vector: Some(self.vector.clone()),
            },
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct RootStoreConfig {
    pub backend: String,
    pub sqlite_path: String,
    pub wal_mode: bool,
    pub dict_path: Option<String>,
}

impl Default for RootStoreConfig {
    fn default() -> Self {
        Self {
            backend: "sqlite".to_string(),
            sqlite_path: DEFAULT_DB_PATH.to_string(),
            wal_mode: true,
            dict_path: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct RootLlmConfig {
    pub provider: String,
    pub model: String,
    pub api_base: Option<String>,
    pub api_key: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub timeout_seconds: Option<u64>,
    pub enable_thinking: Option<bool>,
}

impl RootLlmConfig {
    pub fn is_configured(&self) -> bool {
        !self.provider.trim().is_empty()
            && !self.model.trim().is_empty()
            && self
                .api_key
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct RootEmbeddingRuntimeConfig {
    pub provider: String,
    pub model: String,
    pub api_base: Option<String>,
    pub api_key: Option<String>,
    pub dimensions: Option<u32>,
    pub batch_size: Option<u32>,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VectorBackend {
    #[default]
    None,
    SqliteVec,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct RootVectorConfig {
    pub backend: VectorBackend,
    pub table: String,
    pub similarity: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RetrievalModeVariant {
    pub name: String,
    pub db_path: String,
    pub mode: RetrievalMode,
    pub embedding_backend: EmbeddingBackend,
    pub llm: RootLlmConfig,
    pub embedding: Option<RootEmbeddingRuntimeConfig>,
    pub vector: Option<RootVectorConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            db_path: DEFAULT_DB_PATH.to_string(),
            dict_path: None,
            retrieval: RetrievalConfig::default(),
            embedding: EmbeddingConfig::default(),
            llm: RootLlmConfig::default(),
            memory: MemoryConfig::default(),
            cognition: CognitionConfig::default(),
            vector: RootVectorConfig::default(),
            prompts: PromptsConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        Self::load_from(&default_config_path())
    }

    pub fn load_from(path: &Path) -> Result<Self, ConfigError> {
        let contents = match fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(source) if source.kind() == ErrorKind::NotFound => return Ok(Self::default()),
            Err(source) => {
                return Err(ConfigError::Read {
                    path: path.to_path_buf(),
                    source,
                });
            }
        };

        // Two config schemas coexist:
        // - the root runtime schema (config/agent-memos.toml) keyed by the
        //   presence of a [store] block, embedding configured as
        //   `[embedding] provider = "..."`;
        // - the legacy flat schema (top-level db_path) with
        //   `[embedding] backend = "..."`.
        // Because every struct is #[serde(default)], both parse as either
        // type — the [store] marker decides which interpretation wins, so a
        // legacy file keeps its db_path/embedding block instead of silently
        // falling back to root-schema defaults.
        let value: toml::Value = toml::from_str(&contents).map_err(ConfigError::Parse)?;

        if value.get("store").is_some() {
            let runtime: RootRuntimeConfig =
                value.clone().try_into().map_err(ConfigError::Parse)?;
            let mut config = runtime.to_config();
            // A root-schema file may still carry explicit [retrieval],
            // [memory] or [cognition] overrides; preserve them.
            if let Ok(legacy) = value.clone().try_into::<Config>() {
                if contents.contains("[retrieval]") {
                    config.retrieval = legacy.retrieval;
                }
                if contents.contains("[memory]") {
                    config.memory = legacy.memory;
                }
                if contents.contains("[cognition]") {
                    config.cognition = legacy.cognition;
                }
            }
            return Ok(config);
        }

        value.try_into().map_err(ConfigError::Parse)
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
        assert_eq!(config.vector.backend, VectorBackend::None);
        assert_eq!(config.memory.summary_backend, MemorySummaryBackend::Auto);
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
    fn memory_summary_backend_auto_uses_llm_when_fully_configured() {
        let config = Config {
            llm: RootLlmConfig {
                provider: "openai".to_string(),
                model: "gpt-4o-mini".to_string(),
                api_key: Some("sk-test".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        assert!(config.llm.is_configured());
        assert_eq!(
            config.memory.effective_summary_backend(&config.llm),
            MemorySummaryBackend::Rig
        );
    }

    #[test]
    fn memory_summary_backend_auto_falls_back_without_llm_credentials() {
        let config = Config::default();

        assert!(!config.llm.is_configured());
        assert_eq!(
            config.memory.effective_summary_backend(&config.llm),
            MemorySummaryBackend::RuleBased
        );
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

    #[test]
    fn config_parses_memory_summary_backend_and_llm_block() {
        let config = toml::from_str::<Config>(
            r#"
db_path = "/tmp/agent-memos.db"

[retrieval]
mode = "lexical_only"

[embedding]
backend = "disabled"

[memory]
summary_backend = "rig"

[llm]
provider = "openai"
model = "gpt-4o-mini"
api_key = "sk-test"
"#,
        )
        .expect("memory and llm blocks should parse");

        assert_eq!(config.memory.summary_backend, MemorySummaryBackend::Rig);
        assert_eq!(config.llm.provider, "openai");
        assert_eq!(config.llm.model, "gpt-4o-mini");
        assert_eq!(config.llm.api_key.as_deref(), Some("sk-test"));
    }

    trait ParseMode {
        fn parse_mode(self) -> RetrievalMode;
    }

    #[test]
    fn root_config_maps_remote_http_provider_and_cognition_defaults_on() {
        let runtime = toml::from_str::<RootRuntimeConfig>(
            r#"
[store]
backend = "sqlite"
sqlite_path = "/tmp/agent-memos.db"

[embedding]
provider = "remote_http"
model = "BAAI/bge-m3"
api_base = "https://api.siliconflow.cn/v1"
api_key = "sk-test"
dimensions = 1024
batch_size = 32
timeout_seconds = 30

[vector]
backend = "sqlite_vec"
table = "object_embeddings_vec"
similarity = "cosine"
"#,
        )
        .expect("remote_http root config should parse");

        let config = runtime.to_config();
        assert_eq!(config.embedding.backend, EmbeddingBackend::RemoteHttp);
        assert_eq!(config.embedding.model.as_deref(), Some("BAAI/bge-m3"));
        assert_eq!(
            config.embedding.endpoint.as_deref(),
            Some("https://api.siliconflow.cn/v1")
        );
        assert_eq!(config.embedding.dimensions, Some(1024));
        assert_eq!(config.embedding.batch_size, Some(32));
        assert_eq!(config.embedding.timeout_seconds, Some(30));
        assert_eq!(config.vector.backend, VectorBackend::SqliteVec);
        assert_eq!(
            config.vector.table, "object_embeddings_vec",
            "vector table name should carry through for the vec0 index"
        );
        assert!(
            config.cognition.auto_rumination,
            "auto_rumination defaults to true when no [cognition] block is present"
        );
    }

    #[test]
    fn cognition_auto_rumination_can_be_disabled_and_preserved_from_legacy_block() {
        let config = toml::from_str::<Config>(
            r#"
db_path = "/tmp/agent-memos.db"

[cognition]
auto_rumination = false
"#,
        )
        .expect("legacy cognition block should parse");

        assert!(!config.cognition.auto_rumination);

        let runtime_style = Config::load_from(&{
            let path = std::env::temp_dir().join("agent-memos-cognition-preserve-test.toml");
            std::fs::write(
                &path,
                r#"
[store]
backend = "sqlite"
sqlite_path = "/tmp/agent-memos.db"

[cognition]
auto_rumination = false
"#,
            )
            .expect("write temp config");
            path
        })
        .expect("runtime config should load");
        assert!(
            !runtime_style.cognition.auto_rumination,
            "explicit [cognition] in a root-schema file must be preserved over the default"
        );
        let _ = std::fs::remove_file(
            std::env::temp_dir().join("agent-memos-cognition-preserve-test.toml"),
        );
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
