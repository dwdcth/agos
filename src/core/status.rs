use std::{
    fmt,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags};

use crate::core::{
    app::AppContext,
    config::{EmbeddingBackend, RetrievalMode},
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct DatabaseInspection {
    schema_version: Option<u32>,
    schema_state: CapabilityState,
    base_table_state: CapabilityState,
    note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityState {
    Ready,
    Deferred,
    Disabled,
    Missing,
    NotApplicable,
    NotBuiltInPhase1,
}

impl CapabilityState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Deferred => "deferred",
            Self::Disabled => "disabled",
            Self::Missing => "missing",
            Self::NotApplicable => "not_applicable",
            Self::NotBuiltInPhase1 => "not_built_in_phase_1",
        }
    }
}

impl fmt::Display for CapabilityState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusReport {
    pub db_path: PathBuf,
    pub schema_version: Option<u32>,
    pub configured_mode: RetrievalMode,
    pub effective_mode: RetrievalMode,
    pub embedding_backend: EmbeddingBackend,
    pub schema_state: CapabilityState,
    pub lexical_dependency_state: CapabilityState,
    pub embedding_dependency_state: CapabilityState,
    pub base_table_state: CapabilityState,
    pub index_readiness: CapabilityState,
    pub ready: bool,
    pub readiness_notes: Vec<String>,
}

impl StatusReport {
    pub fn collect(app: &AppContext) -> Result<Self> {
        let db_path = app.db_path().to_path_buf();
        let db_exists = db_path.exists();
        let inspection = if db_exists {
            inspect_database(&db_path).unwrap_or_else(|error| DatabaseInspection {
                schema_version: None,
                schema_state: CapabilityState::Missing,
                base_table_state: CapabilityState::Missing,
                note: Some(format!(
                    "schema inspection failed for existing database file: {error:#}"
                )),
            })
        } else {
            DatabaseInspection {
                schema_version: None,
                schema_state: CapabilityState::Missing,
                base_table_state: CapabilityState::Missing,
                note: None,
            }
        };

        let lexical_dependency_state = match app.config.retrieval.mode {
            RetrievalMode::LexicalOnly | RetrievalMode::Hybrid => {
                CapabilityState::NotBuiltInPhase1
            }
            RetrievalMode::EmbeddingOnly => CapabilityState::NotApplicable,
        };

        let embedding_dependency_state =
            embedding_dependency_state(app.config.retrieval.mode, app.config.embedding.backend);
        let index_readiness = match app.config.retrieval.mode {
            RetrievalMode::LexicalOnly | RetrievalMode::Hybrid => {
                CapabilityState::NotBuiltInPhase1
            }
            RetrievalMode::EmbeddingOnly => CapabilityState::NotApplicable,
        };

        let mut readiness_notes = app.readiness.notes.clone();
        if let Some(note) = inspection.note {
            readiness_notes.push(note);
        }
        if !db_exists {
            readiness_notes.push(
                "database has not been initialized yet; run `agent-memos init` to create the Phase 1 schema"
                    .to_string(),
            );
        } else if !matches!(inspection.base_table_state, CapabilityState::Ready) {
            readiness_notes
                .push("foundation base tables are incomplete or missing".to_string());
        }

        let ready = app.readiness.ready
            && matches!(inspection.schema_state, CapabilityState::Ready)
            && matches!(inspection.base_table_state, CapabilityState::Ready);

        Ok(Self {
            db_path,
            schema_version: inspection.schema_version,
            configured_mode: app.readiness.configured_mode,
            effective_mode: app.readiness.effective_mode,
            embedding_backend: app.config.embedding.backend,
            schema_state: inspection.schema_state,
            lexical_dependency_state,
            embedding_dependency_state,
            base_table_state: inspection.base_table_state,
            index_readiness,
            ready,
            readiness_notes,
        })
    }

    pub fn render_text(&self) -> String {
        let mut lines = vec![
            "database:".to_string(),
            format!("  path: {}", self.db_path.display()),
            format!("  exists: {}", self.db_path.exists()),
            format!("  schema_state: {}", self.schema_state),
            format!(
                "  schema_version: {}",
                self.schema_version
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string())
            ),
            format!("  base_table_state: {}", self.base_table_state),
            "retrieval:".to_string(),
            format!(
                "  configured_mode: {}",
                retrieval_mode_label(self.configured_mode)
            ),
            format!(
                "  effective_mode: {}",
                retrieval_mode_label(self.effective_mode)
            ),
            format!(
                "  embedding_backend: {}",
                embedding_backend_label(self.embedding_backend)
            ),
            format!("  ready: {}", self.ready),
            "dependencies:".to_string(),
            format!(
                "  lexical_dependency_state: {}",
                self.lexical_dependency_state
            ),
            format!(
                "  embedding_dependency_state: {}",
                self.embedding_dependency_state
            ),
            format!("  index_readiness: {}", self.index_readiness),
        ];

        if !self.readiness_notes.is_empty() {
            lines.push("notes:".to_string());
            lines.extend(
                self.readiness_notes
                    .iter()
                    .map(|note| format!("  - {note}")),
            );
        }

        lines.join("\n")
    }
}

fn inspect_database(path: &Path) -> Result<DatabaseInspection> {
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("failed to inspect sqlite database at {}", path.display()))?;
    let schema_version = conn
        .query_row("PRAGMA user_version;", [], |row| row.get::<_, u32>(0))
        .context("failed to read schema version")?;
    let base_table_exists = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'memory_records')",
            [],
            |row| row.get::<_, i64>(0),
        )
        .context("failed to inspect base table readiness")?
        != 0;

    let schema_state = if schema_version > 0 {
        CapabilityState::Ready
    } else {
        CapabilityState::Missing
    };
    let base_table_state = if base_table_exists {
        CapabilityState::Ready
    } else {
        CapabilityState::Missing
    };

    Ok(DatabaseInspection {
        schema_version: Some(schema_version),
        schema_state,
        base_table_state,
        note: None,
    })
}

fn embedding_dependency_state(
    mode: RetrievalMode,
    backend: EmbeddingBackend,
) -> CapabilityState {
    match (mode, backend) {
        (RetrievalMode::LexicalOnly, EmbeddingBackend::Disabled) => CapabilityState::Disabled,
        (RetrievalMode::LexicalOnly, EmbeddingBackend::Reserved) => CapabilityState::Deferred,
        (RetrievalMode::EmbeddingOnly | RetrievalMode::Hybrid, EmbeddingBackend::Disabled) => {
            CapabilityState::Missing
        }
        (
            RetrievalMode::EmbeddingOnly | RetrievalMode::Hybrid,
            EmbeddingBackend::Reserved,
        ) => CapabilityState::Deferred,
    }
}

pub fn retrieval_mode_label(mode: RetrievalMode) -> &'static str {
    match mode {
        RetrievalMode::LexicalOnly => "lexical_only",
        RetrievalMode::EmbeddingOnly => "embedding_only",
        RetrievalMode::Hybrid => "hybrid",
    }
}

pub fn embedding_backend_label(backend: EmbeddingBackend) -> &'static str {
    match backend {
        EmbeddingBackend::Disabled => "disabled",
        EmbeddingBackend::Reserved => "reserved",
    }
}
