use async_trait::async_trait;
use std::sync::Arc;

use crate::adapter::Transport;
use crate::workhub::AgentType;
use crate::events::EventBridge;
use crate::llm::LlmProvider;
use crate::error::AppResult;
use crate::agents::types::{GoalContext, AgentOutput};

/// Core trait for all SideCar Agents.
/// Agents are pure `run() → output` functions.
/// The supervisor handles spawning, cancellation, persistence, and phase transitions.
#[async_trait]
pub trait SideCarAgent: Send + Sync {
    /// Which agent type this is
    fn agent_type(&self) -> AgentType;

    /// Run the agent to completion. Returns the agent's output.
    /// Receives:
    /// - ctx: the goal context loaded from DB
    /// - transport: for communicating with Harness Agents
    /// - llm: for LLM completions
    /// - emitter: for streaming events to the frontend
    async fn run(
        &self,
        ctx: GoalContext,
        transport: Arc<dyn Transport>,
        llm: Arc<dyn LlmProvider>,
        emitter: EventBridge,
    ) -> AppResult<AgentOutput>;

    /// Cancel the running agent gracefully
    async fn cancel(&self) -> AppResult<()>;
}
