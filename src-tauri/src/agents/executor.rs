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
use crate::agents::types::{GoalContext, AgentOutput, BugInfo};
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
        ctx: &GoalContext,
        llm: &Arc<dyn LlmProvider>,
        emitter: &EventBridge,
    ) -> AppResult<ExecutionResult> {
        let wid = ctx.workspace_id.as_deref().unwrap_or(&ctx.goal.id);
        let user_manual = self.get_user_manual(ctx).await?;
        let skills = &ctx.available_skills;

        if skills.is_empty() {
            emitter.emit_agent_stream(
                wid,
                "No skills available for execution\n",
                false,
            );
            return Ok(ExecutionResult {
                success: true,
                message: "No skills to execute - verification skipped".to_string(),
                bugs: vec![],
            });
        }

        emitter.emit_agent_stream(
            wid,
            &format!("Starting execution with {} skills\n", skills.len()),
            false,
        );

        // Build execution context
        let mut state = ExecutionState::new();
        let mut bugs: Vec<BugEntry> = Vec::new();

        // ReAct loop
        for iteration in 0..self.max_iterations {
            if self.cancel_token.is_cancelled() {
                return Ok(ExecutionResult {
                    success: false,
                    message: "Execution cancelled".to_string(),
                    bugs,
                });
            }

            // Get next action from LLM
            let action = self.decide_action(
                ctx,
                &user_manual,
                skills,
                &state,
                llm,
            ).await?;

            match action {
                ExecutionAction::ExecuteSkill { skill_name, params } => {
                    emitter.emit_agent_stream(
                        wid,
                        &format!("Executing skill: {} with params: {:?}\n", skill_name, params),
                        false,
                    );

                    // Execute the skill (via Transport to HarnessAgent)
                    let result = self.execute_skill(&skill_name, &params).await;

                    match result {
                        Ok(output) => {
                            state.record_success(&skill_name, &output);
                            emitter.emit_agent_stream(
                                wid,
                                &format!("Skill {} executed successfully\n", skill_name),
                                false,
                            );
                        }
                        Err(e) => {
                            let error_msg = e.to_string();
                            state.record_failure(&skill_name, &error_msg);

                            let bug = BugEntry::new(
                                format!("Skill {} failed", skill_name),
                                Some(error_msg.clone()),
                            );
                            bugs.push(bug);

                            emitter.emit_agent_stream(
                                wid,
                                &format!("Skill {} failed: {}\n", skill_name, error_msg),
                                false,
                            );
                        }
                    }
                }

                ExecutionAction::ReportSuccess { summary } => {
                    emitter.emit_agent_stream(
                        wid,
                        &format!("Execution complete: {}\n", summary),
                        true,
                    );
                    return Ok(ExecutionResult {
                        success: true,
                        message: summary,
                        bugs,
                    });
                }

                ExecutionAction::ReportFailure { reason } => {
                    emitter.emit_agent_stream(
                        wid,
                        &format!("Execution failed: {}\n", reason),
                        true,
                    );
                    return Ok(ExecutionResult {
                        success: false,
                        message: reason,
                        bugs,
                    });
                }
            }

            // Check if we've made progress
            if iteration > 0 && iteration % 10 == 0 {
                emitter.emit_agent_stream(
                    wid,
                    &format!("Execution iteration {}/{}\n", iteration, self.max_iterations),
                    false,
                );
            }
        }

        // Max iterations reached
        Ok(ExecutionResult {
            success: false,
            message: format!("Max iterations ({}) reached", self.max_iterations),
            bugs,
        })
    }

    /// Get user manual from current worknode
    async fn get_user_manual(&self, ctx: &GoalContext) -> AppResult<String> {
        // Try to get from current milestone/nodespace
        // For now, return a placeholder
        Ok(ctx.goal.summary.clone().unwrap_or_else(|| "No user manual available".to_string()))
    }

    /// Decide next action using LLM
    async fn decide_action(
        &self,
        ctx: &GoalContext,
        user_manual: &str,
        skills: &[crate::workhub::Skill],
        state: &ExecutionState,
        llm: &Arc<dyn LlmProvider>,
    ) -> AppResult<ExecutionAction> {
        let skill_list: Vec<String> = skills.iter()
            .map(|s| format!("- {}: {}", s.name, s.description))
            .collect();

        let prompt = format!(
            r#"You are an execution agent. Based on the following context, decide the next action.

User Manual:
{}

Available Skills:
{}

Execution History:
{}

What should you do next?
- If you need to execute a skill, respond with: EXECUTE <skill_name> <json_params>
- If all tasks are complete, respond with: SUCCESS <summary>
- If there's an unrecoverable failure, respond with: FAILURE <reason>

Respond with only one line."#,
            user_manual,
            skill_list.join("\n"),
            state.summary()
        );

        let response = llm.complete(
            "You are an execution agent that decides the next action.",
            &prompt,
            &ctx.session.get_history(),
        ).await?;
        self.parse_action(&response, skills)
    }

    /// Parse LLM response into execution action
    fn parse_action(
        &self,
        response: &str,
        skills: &[crate::workhub::Skill],
    ) -> AppResult<ExecutionAction> {
        let response = response.trim();

        if response.starts_with("EXECUTE ") {
            let rest = response.strip_prefix("EXECUTE ").unwrap();
            let parts: Vec<&str> = rest.splitn(2, ' ').collect();

            if parts.is_empty() {
                return Err(AppError::Agent("Invalid EXECUTE command".to_string()));
            }

            let skill_name = parts[0].to_string();
            let params = if parts.len() > 1 {
                serde_json::from_str(parts[1]).unwrap_or(serde_json::json!({}))
            } else {
                serde_json::json!({})
            };

            // Validate skill exists
            if !skills.iter().any(|s| s.name == skill_name) {
                return Err(AppError::Agent(format!("Unknown skill: {}", skill_name)));
            }

            Ok(ExecutionAction::ExecuteSkill { skill_name, params })
        } else if response.starts_with("SUCCESS ") {
            let summary = response.strip_prefix("SUCCESS ").unwrap().to_string();
            Ok(ExecutionAction::ReportSuccess { summary })
        } else if response.starts_with("FAILURE ") {
            let reason = response.strip_prefix("FAILURE ").unwrap().to_string();
            Ok(ExecutionAction::ReportFailure { reason })
        } else {
            // Default to reporting failure if response is unclear
            Ok(ExecutionAction::ReportFailure {
                reason: format!("Unclear action response: {}", response),
            })
        }
    }

    /// Execute a skill (placeholder - would communicate with HarnessAgent)
    async fn execute_skill(&self, _skill_name: &str, _params: &serde_json::Value) -> AppResult<String> {
        // In a real implementation, this would send a request to HarnessAgent
        // via the Transport trait to execute the skill
        //
        // For now, return a simulated success
        Ok(format!("Skill {} executed with params: {}", _skill_name, _params))
    }
}

#[async_trait]
impl SideCarAgent for ExecutorAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Executor
    }

    async fn run(
        &self,
        ctx: GoalContext,
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
            milestone_id: ctx.goal.current_milestone_id.clone().unwrap_or_default(),
            plan: None,
            skills_used: ctx.available_skills.iter().map(|s| s.name.clone()).collect(),
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


