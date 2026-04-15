use rig::{agent::AgentBuilder, completion::CompletionModel};
use thiserror::Error;

use crate::agent::orchestration::{
    AgentSearchReport, AgentSearchRequest, AgentSearchRunner, AgentSearchError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RigBoundary {
    pub tool_name: &'static str,
    pub default_max_turns: usize,
    pub allows_truth_write: bool,
    pub allows_semantic_retrieval: bool,
    pub allows_rumination: bool,
}

impl Default for RigBoundary {
    fn default() -> Self {
        Self {
            tool_name: "internal_agent_search",
            default_max_turns: 4,
            allows_truth_write: false,
            allows_semantic_retrieval: false,
            allows_rumination: false,
        }
    }
}

#[derive(Debug, Error)]
pub enum RigAgentSearchError {
    #[error("internal agent-search orchestration failed")]
    Orchestration(#[from] AgentSearchError),
}

pub struct RigAgentSearchAdapter<O> {
    orchestrator: O,
    boundary: RigBoundary,
    static_context: Vec<String>,
}

impl<O> RigAgentSearchAdapter<O> {
    pub fn new(orchestrator: O) -> Self {
        Self {
            orchestrator,
            boundary: RigBoundary::default(),
            static_context: vec![
                "Use only the internal retrieve -> assemble -> score -> gate workflow.".to_string(),
                "Do not write shared truth, do not trigger semantic retrieval, and do not invoke rumination.".to_string(),
                "Return structured cited reports instead of freeform answers.".to_string(),
            ],
        }
    }

    pub fn boundary(&self) -> &RigBoundary {
        &self.boundary
    }

    pub fn static_context(&self) -> &[String] {
        &self.static_context
    }

    pub fn prepare_builder<M>(&self, builder: AgentBuilder<M>) -> AgentBuilder<M>
    where
        M: CompletionModel,
    {
        self.static_context
            .iter()
            .fold(
                builder
                    .description("Thin Rig adapter for cited agent-search orchestration")
                    .preamble(
                        "Delegate all cognition to the internal agent-search orchestrator and preserve gate diagnostics.",
                    )
                    .default_max_turns(self.boundary.default_max_turns),
                |builder, doc| builder.context(doc),
            )
    }
}

impl<O> RigAgentSearchAdapter<O>
where
    O: AgentSearchRunner,
{
    pub async fn run(
        &self,
        request: &AgentSearchRequest,
    ) -> Result<AgentSearchReport, RigAgentSearchError> {
        Ok(self.orchestrator.run(request)?)
    }
}
