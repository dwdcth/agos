use crate::core::{
    config::{EmbeddingBackend, RetrievalMode},
    status::{CapabilityState, StatusReport},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandPath {
    Init,
    Doctor,
    Ingest,
    Search,
    AgentSearch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorReport {
    pub ready: bool,
    pub failures: Vec<String>,
    pub warnings: Vec<String>,
}

impl DoctorReport {
    pub fn evaluate(status: &StatusReport, command_path: CommandPath) -> Self {
        let mut failures = Vec::new();
        let mut warnings = Vec::new();

        match (status.configured_mode, status.embedding_backend) {
            (RetrievalMode::EmbeddingOnly, EmbeddingBackend::Disabled) => failures.push(
                "embedding_only requires a non-disabled embedding backend".to_string(),
            ),
            (RetrievalMode::Hybrid, EmbeddingBackend::Disabled) => failures.push(
                "hybrid requires an embedding backend for the secondary path".to_string(),
            ),
            (RetrievalMode::LexicalOnly, EmbeddingBackend::Reserved) => warnings.push(
                "embedding backend is configured but unused while retrieval.mode=lexical_only"
                    .to_string(),
            ),
            _ => {}
        }

        if blocks_reserved_semantic_modes(command_path) {
            match (status.configured_mode, status.embedding_backend) {
                (RetrievalMode::EmbeddingOnly, EmbeddingBackend::Reserved) => failures.push(
                    "embedding_only is reserved but not implemented in Phase 1".to_string(),
                ),
                (RetrievalMode::Hybrid, EmbeddingBackend::Reserved) => failures.push(
                    "hybrid keeps lexical as the primary baseline, but the embedding secondary path is reserved in Phase 1"
                        .to_string(),
                ),
                _ => {}
            }
        }

        if requires_operational_readiness(command_path) {
            failures.extend(operational_readiness_failures(status));
        }

        if matches!(status.schema_state, CapabilityState::Missing) {
            warnings.push(
                "database schema is not initialized yet; run `agent-memos init` to create it"
                    .to_string(),
            );
        }

        if matches!(
            status.lexical_dependency_state,
            CapabilityState::NotBuiltInPhase1
        ) {
            warnings.push(
                "lexical dependency loading is deferred until the retrieval phase".to_string(),
            );
        }

        if matches!(status.index_readiness, CapabilityState::NotBuiltInPhase1) {
            warnings
                .push("retrieval indexes are reserved and not built in Phase 1".to_string());
        }

        Self {
            ready: failures.is_empty(),
            failures,
            warnings,
        }
    }

    pub fn render_text(&self) -> String {
        let mut lines = vec![format!("ready: {}", self.ready)];

        lines.push("failures:".to_string());
        if self.failures.is_empty() {
            lines.push("  - none".to_string());
        } else {
            lines.extend(self.failures.iter().map(|failure| format!("  - {failure}")));
        }

        lines.push("warnings:".to_string());
        if self.warnings.is_empty() {
            lines.push("  - none".to_string());
        } else {
            lines.extend(self.warnings.iter().map(|warning| format!("  - {warning}")));
        }

        lines.join("\n")
    }
}

fn blocks_reserved_semantic_modes(command_path: CommandPath) -> bool {
    matches!(
        command_path,
        CommandPath::Doctor | CommandPath::Ingest | CommandPath::Search | CommandPath::AgentSearch
    )
}

fn requires_operational_readiness(command_path: CommandPath) -> bool {
    matches!(
        command_path,
        CommandPath::Ingest | CommandPath::Search | CommandPath::AgentSearch
    )
}

fn operational_readiness_failures(status: &StatusReport) -> Vec<String> {
    if !matches!(status.configured_mode, RetrievalMode::LexicalOnly) {
        return Vec::new();
    }

    let mut failures = Vec::new();

    if matches!(status.schema_state, CapabilityState::Missing) {
        failures.push(
            "database schema is not initialized yet; run `agent-memos init` to create it"
                .to_string(),
        );
    }

    if matches!(status.base_table_state, CapabilityState::Missing) {
        failures.push("foundation base tables are incomplete or missing".to_string());
    }

    if matches!(status.index_readiness, CapabilityState::Missing) {
        failures.push("lexical sidecar indexes are missing or incomplete".to_string());
    }

    if let Some(note) = status
        .readiness_notes
        .iter()
        .find(|note| note.starts_with("schema inspection failed for existing database file"))
    {
        failures.push(note.clone());
    }

    failures
}
