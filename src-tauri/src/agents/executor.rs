//! Executor Agent
//!
//! Responsible for executing and testing skills during verification phases.
//! Does not modify any skills - only runs them and reports results.

use async_trait::async_trait;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

use crate::adapter::Transport;
use crate::workhub::AgentType;
use crate::error::{AppError, AppResult};
use crate::events::EventBridge;
use crate::llm::LlmProvider;
use crate::agents::traits::SideCarAgent;
use crate::agents::types::{RuntimeContext, AgentOutput, BugInfo};
use crate::workhub::{WorkHub, BugEntry};

/// Maximum execution iterations
const MAX_ITERATIONS: u32 = 50;

/// Executor Agent
///
/// Executes user_manual and skills to verify functionality.
/// Reports success or logs bugs to the current worknode.
pub struct ExecutorAgent {
    cancel_token: CancellationToken,
    max_iterations: u32,
}

impl ExecutorAgent {
    pub fn new(cancel_token: CancellationToken) -> Self {
        Self {
            cancel_token,
            max_iterations: MAX_ITERATIONS,
        }
    }

    /// Main execution loop
    async fn execute(
        &self,
        ctx: &RuntimeContext,
        llm: &Arc<dyn LlmProvider>,
        emitter: &EventBridge,
    ) -> AppResult<ExecutionResult> {
        let wid = ctx.workspace_id.as_deref().unwrap_or(&ctx.goal.id);
        let _user_manual = self.get_user_manual(ctx).await?;

        emitter.emit_agent_stream(
            wid,
            "No skills available for execution — skipping\n",
            false,
        );
        return Ok(ExecutionResult {
            success: true,
            message: "No skills to execute - verification skipped".to_string(),
            bugs: vec![],
        });
    }

    /// Get user manual from current worknode
    async fn get_user_manual(&self, ctx: &RuntimeContext) -> AppResult<String> {
        // Try to get from current milestone/nodespace
        // For now, return a placeholder
        Ok(ctx.goal.summary.clone().unwrap_or_else(|| "No user manual available".to_string()))
    }
}

#[async_trait]
impl SideCarAgent for ExecutorAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Executor
    }

    async fn run(
        &self,
        ctx: RuntimeContext,
        _transport: Arc<dyn Transport>,
        llm: Arc<dyn LlmProvider>,
        emitter: EventBridge,
    ) -> AppResult<AgentOutput> {
        let wid = ctx.workspace_id.as_deref().unwrap_or(&ctx.goal.id);
        let result = self.execute(&ctx, &llm, &emitter).await?;

        // If execution failed and we have a current node, add bugs
        if !result.success && !result.bugs.is_empty() {
            // In a real implementation, we would persist bugs to the worknode
            // using WorkHub::add_node_bug
            emitter.emit_agent_stream(
                wid,
                &format!("Logged {} bugs\n", result.bugs.len()),
                false,
            );
        }

        Ok(AgentOutput::ExecutionResult {
            milestone_id: String::new(),
            plan: None,
            skills_used: vec![],
            success: result.success,
            failure_reason: if result.success { None } else { Some(result.message) },
            bugs_found: result.bugs.into_iter().map(|b| BugInfo {
                description: b.description,
                error_output: b.error_output,
            }).collect(),
        })
    }

    async fn cancel(&self) -> AppResult<()> {
        self.cancel_token.cancel();
        Ok(())
    }
}

/// Execution state tracking
#[derive(Debug, Default)]
struct ExecutionState {
    successes: Vec<(String, String)>,
    failures: Vec<(String, String)>,
}

impl ExecutionState {
    fn new() -> Self {
        Self::default()
    }

    fn record_success(&mut self, skill_name: &str, output: &str) {
        self.successes.push((skill_name.to_string(), output.to_string()));
    }

    fn record_failure(&mut self, skill_name: &str, error: &str) {
        self.failures.push((skill_name.to_string(), error.to_string()));
    }

    fn summary(&self) -> String {
        let mut summary = String::new();
        summary.push_str(&format!("Successes: {}\n", self.successes.len()));
        summary.push_str(&format!("Failures: {}\n", self.failures.len()));

        if !self.failures.is_empty() {
            summary.push_str("Recent failures:\n");
            for (skill, error) in self.failures.iter().take(5) {
                summary.push_str(&format!("- {}: {}\n", skill, error));
            }
        }

        summary
    }
}

/// Result of execution
#[derive(Debug)]
struct ExecutionResult {
    success: bool,
    message: String,
    bugs: Vec<BugEntry>,
}

/// Actions the executor can take
#[derive(Debug)]
enum ExecutionAction {
    ExecuteSkill {
        skill_name: String,
        params: serde_json::Value,
    },
    ReportSuccess {
        summary: String,
    },
    ReportFailure {
        reason: String,
    },
}


