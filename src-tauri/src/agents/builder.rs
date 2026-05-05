//! Builder Agent (原型构建 Agent)
//!
//! Responsible for goal planning and initial achievement.
//! Core flow:
//! 1. Initial analysis - get conclusion, plan, create PLAN.md
//! 2. Transform and dispatch - create spec, send to generator
//! 3. Verification - check generator result, run executor
//! 4. Conclusion - mark achieved or retry

use async_trait::async_trait;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

use crate::adapter::Transport;
use crate::adapter::generator::{
    ExecuteParams, ExecuteResponse, StatusParams, StatusResponse,
    METHOD_EXECUTE, METHOD_STATUS,
};
use crate::session::Message;
use crate::workhub::AgentType;
use crate::error::{AppError, AppResult};
use crate::events::EventBridge;
use crate::llm::LlmProvider;
use crate::agents::traits::SideCarAgent;
use crate::agents::types::{RuntimeContext, AgentOutput};
use crate::workhub::{
    WorkHub, Specification, ModuleSpec, SkillRequirement,
};

/// Maximum retry attempts for generator
const MAX_RETRIES: u32 = 10;

/// Builder Agent
///
/// Handles the initial goal achievement process.
pub struct BuilderAgent {
    cancel_token: CancellationToken,
    max_retries: u32,
}

impl BuilderAgent {
    pub fn new(cancel_token: CancellationToken) -> Self {
        Self {
            cancel_token,
            max_retries: MAX_RETRIES,
        }
    }

    /// Phase 1: Initial Analysis
    async fn analyze(
        &self,
        ctx: &RuntimeContext,
        llm: &Arc<dyn LlmProvider>,
        emitter: &EventBridge,
    ) -> AppResult<PlanResult> {
        let wid = ctx.workspace_id.as_deref().unwrap_or(&ctx.goal.id);
        emitter.emit_agent_stream(
            wid,
            "Starting initial analysis...\n",
            false,
        );

        // Get existing conclusion if any
        let existing_conclusion = self.get_existing_conclusion(ctx).await?;

        // Analyze the goal
        let prompt = self.build_analysis_prompt(ctx, &existing_conclusion);
        let history = ctx.session.get_history();
        let response = llm.complete(
            "You are a goal planning agent that creates detailed plans.",
            &prompt,
            &history,
        ).await?;

        // Parse the plan
        let plan = self.parse_plan(&response)?;

        emitter.emit_agent_stream(
            wid,
            &format!("Plan created with {} modules\n", plan.modules.len()),
            false,
        );

        Ok(plan)
    }

    /// Phase 2: Transform and Dispatch
    async fn dispatch_to_generator(
        &self,
        ctx: &RuntimeContext,
        plan: &PlanResult,
        transport: &Arc<dyn Transport>,
        llm: &Arc<dyn LlmProvider>,
        emitter: &EventBridge,
    ) -> AppResult<GeneratorTaskResult> {
        let wid = ctx.workspace_id.as_deref().unwrap_or(&ctx.goal.id);
        emitter.emit_agent_stream(
            wid,
            "Creating specification for generator...\n",
            false,
        );

        // Transform plan to specification
        let spec = self.transform_to_specification(plan, ctx);

        // Create task ID
        let task_id = uuid::Uuid::new_v4().to_string();
        let workspace_id = ctx.workspace_id.clone().unwrap_or_default();
        let worknode_id = ctx.current_node_id.clone();

        // Build execute params
        let params = ExecuteParams::new(
            task_id.clone(),
            workspace_id,
            worknode_id,
            spec.clone(),
        );

        // Send to generator via Transport
        let response_value = transport
            .send_request(METHOD_EXECUTE, serde_json::to_value(&params)?)
            .await?;

        let execute_response: ExecuteResponse = serde_json::from_value(response_value)
            .map_err(|e| AppError::Transport(format!("Invalid execute response: {}", e)))?;

        if !execute_response.accepted {
            return Err(AppError::Transport(
                execute_response.message.unwrap_or_else(|| "Generator rejected task".to_string())
            ));
        }

        emitter.emit_agent_stream(
            wid,
            &format!("Task {} accepted, waiting for completion...\n", task_id),
            false,
        );

        // Poll for completion
        let result = self.wait_for_completion(transport, &task_id, wid, emitter).await?;

        // Optionally, use LLM to enhance the result
        let enhanced_result = self.enhance_generator_result(&result, llm, &ctx.session.get_history()).await?;

        Ok(enhanced_result)
    }

    /// Wait for generator task completion
    async fn wait_for_completion(
        &self,
        transport: &Arc<dyn Transport>,
        task_id: &str,
        workspace_id: &str,
        emitter: &EventBridge,
    ) -> AppResult<GeneratorTaskResult> {
        let mut attempts = 0u32;
        let max_attempts = 600; // 10 minutes at 1-second intervals
        let poll_interval = std::time::Duration::from_secs(1);

        loop {
            if self.cancel_token.is_cancelled() {
                // Try to cancel the task
                let _ = transport.send_request(
                    crate::adapter::generator::METHOD_CANCEL,
                    serde_json::json!({ "task_id": task_id }),
                ).await;
                return Err(AppError::Cancelled("Task cancelled by user".to_string()));
            }

            let status_params = StatusParams { task_id: task_id.to_string() };
            let status_value = transport
                .send_request(METHOD_STATUS, serde_json::to_value(&status_params)?)
                .await?;

            let status: StatusResponse = serde_json::from_value(status_value)
                .map_err(|e| AppError::Transport(format!("Invalid status response: {}", e)))?;

            match status.status {
                crate::adapter::generator::GeneratorTaskStatus::Completed => {
                    emitter.emit_agent_stream(
                        workspace_id,
                        "Generator task completed successfully\n",
                        false,
                    );
                    let result = status.result.ok_or_else(|| {
                        AppError::Transport("Generator completed but no result provided".to_string())
                    })?;
                    return Ok(GeneratorTaskResult {
                        success: true,
                        result: result.summary,
                        user_manual: result.user_manual,
                        skills: result.skills.into_iter().map(|s| SkillRequirement {
                            name: s.name,
                            description: s.description,
                            parameters_schema: None,
                        }).collect(),
                    });
                }
                crate::adapter::generator::GeneratorTaskStatus::Failed => {
                    let error = status.error.unwrap_or_else(|| "Unknown error".to_string());
                    emitter.emit_agent_stream(
                        workspace_id,
                        &format!("Generator task failed: {}\n", error),
                        false,
                    );
                    return Ok(GeneratorTaskResult {
                        success: false,
                        result: error.clone(),
                        user_manual: None,
                        skills: vec![],
                    });
                }
                crate::adapter::generator::GeneratorTaskStatus::Cancelled => {
                    return Err(AppError::Cancelled("Task was cancelled".to_string()));
                }
                crate::adapter::generator::GeneratorTaskStatus::Running => {
                    if let Some(step) = &status.current_step {
                        emitter.emit_agent_stream(
                            workspace_id,
                            &format!("[{}%] {}\n", status.progress, step),
                            false,
                        );
                    }
                }
                crate::adapter::generator::GeneratorTaskStatus::Pending => {
                    // Still pending, wait
                }
            }

            attempts += 1;
            if attempts >= max_attempts {
                return Err(AppError::Transport("Generator task timed out".to_string()));
            }

            tokio::time::sleep(poll_interval).await;
        }
    }

    /// Enhance generator result with LLM
    async fn enhance_generator_result(
        &self,
        result: &GeneratorTaskResult,
        llm: &Arc<dyn LlmProvider>,
        history: &[Message],
    ) -> AppResult<GeneratorTaskResult> {
        if !result.success {
            return Ok(result.clone());
        }

        // Use LLM to validate/enhance user manual
        if let Some(manual) = &result.user_manual {
            let prompt = format!(
                "Review and improve this user manual. Make it clearer and more actionable:\n\n{}",
                manual
            );
            let _response = llm.complete(
                "You are a technical writer specializing in user documentation.",
                &prompt,
                history,
            ).await?;
            // For now, keep original manual
            // In production, could use LLM response to enhance
        }

        Ok(result.clone())
    }

    /// Phase 3: Verification
    async fn verify(
        &self,
        ctx: &RuntimeContext,
        generator_result: &GeneratorTaskResult,
        llm: &Arc<dyn LlmProvider>,
        emitter: &EventBridge,
    ) -> AppResult<VerificationResult> {
        let wid = ctx.workspace_id.as_deref().unwrap_or(&ctx.goal.id);
        emitter.emit_agent_stream(
            wid,
            "Verifying generator result...\n",
            false,
        );

        // Check for negative feedback
        if !generator_result.success {
            return Ok(VerificationResult {
                passed: false,
                reason: "Generator reported failure".to_string(),
            });
        }

        // Check user manual exists
        if generator_result.user_manual.is_none() {
            emitter.emit_agent_stream(
                wid,
                "Warning: No user manual provided by generator\n",
                false,
            );
        }

        // Run executor for testing
        emitter.emit_agent_stream(
            wid,
            "Running execution agent for verification...\n",
            false,
        );

        // In a real implementation, this would invoke ExecutorAgent
        // For now, simulate a successful verification
        let verify_prompt = format!(
            "Verify the following output from a generator agent. Is it complete and correct?\n\n{}",
            generator_result.result
        );
        let _response = llm.complete(
            "You are a verification agent.",
            &verify_prompt,
            &ctx.session.get_history(),
        ).await?;

        Ok(VerificationResult {
            passed: true,
            reason: "Verification passed".to_string(),
        })
    }

    /// Phase 4: Conclusion or Retry
    async fn conclude(
        &self,
        ctx: &RuntimeContext,
        verification: &VerificationResult,
        attempt: u32,
        emitter: &EventBridge,
    ) -> AppResult<ConclusionResult> {
        let wid = ctx.workspace_id.as_deref().unwrap_or(&ctx.goal.id);
        if verification.passed {
            emitter.emit_agent_stream(
                wid,
                "Goal achieved! Creating conclusion...\n",
                true,
            );

            // In a real implementation:
            // - Call WorkHub::goal_achieved to create new node
            // - Write conclusion.md

            return Ok(ConclusionResult {
                achieved: true,
                message: "Goal successfully achieved".to_string(),
                new_node_id: None,
            });
        }

        if attempt >= self.max_retries {
            emitter.emit_agent_stream(
                wid,
                &format!("Max retries ({}) reached, marking as failed\n", self.max_retries),
                true,
            );

            return Ok(ConclusionResult {
                achieved: false,
                message: format!("Failed after {} attempts", self.max_retries),
                new_node_id: None,
            });
        }

        emitter.emit_agent_stream(
            wid,
            &format!("Attempt {}/{} failed, will retry\n", attempt, self.max_retries),
            false,
        );

        Ok(ConclusionResult {
            achieved: false,
            message: "Retrying...".to_string(),
            new_node_id: None,
        })
    }

    /// Get existing conclusion from current worknode
    async fn get_existing_conclusion(&self, _ctx: &RuntimeContext) -> AppResult<Option<String>> {
        // In a real implementation, use WorkHub::get_node_conclusion
        Ok(None)
    }

    /// Build analysis prompt
    fn build_analysis_prompt(&self, ctx: &RuntimeContext, existing: &Option<String>) -> String {
        let mut prompt = format!(
            r#"You are a goal planning agent. Analyze the following goal and create a detailed plan.

Goal: {}
Raw Input: {}

"#,
            ctx.goal.title,
            ctx.goal.raw_input
        );

        if let Some(conclusion) = existing {
            prompt.push_str(&format!("Previous Conclusion:\n{}\n\n", conclusion));
        }

        prompt.push_str(r#"Create a plan with:
1. Key requirements analysis
2. Module breakdown (list each module)
3. Dependencies between modules
4. Acceptance criteria

Format your response as JSON:
{
  "analysis": "...",
  "modules": [
    {"name": "...", "description": "...", "dependencies": [...]}
  ],
  "acceptance_criteria": [...]
}"#);

        prompt
    }

    /// Parse plan from LLM response
    fn parse_plan(&self, response: &str) -> AppResult<PlanResult> {
        // Extract JSON from response
        let json_str = self.extract_json(response);

        let parsed: serde_json::Value = serde_json::from_str(&json_str)
            .unwrap_or_else(|_| serde_json::json!({
                "analysis": response,
                "modules": [],
                "acceptance_criteria": []
            }));

        let analysis = parsed["analysis"].as_str().unwrap_or("").to_string();
        let modules = parsed["modules"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|m| ModuleInfo {
                        name: m["name"].as_str().unwrap_or("unknown").to_string(),
                        description: m["description"].as_str().unwrap_or("").to_string(),
                        dependencies: m["dependencies"]
                            .as_array()
                            .map(|d| d.iter().filter_map(|s| s.as_str().map(String::from)).collect())
                            .unwrap_or_default(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        let acceptance_criteria = parsed["acceptance_criteria"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|s| s.as_str().map(String::from)).collect())
            .unwrap_or_default();

        Ok(PlanResult {
            analysis,
            modules,
            acceptance_criteria,
        })
    }

    /// Extract JSON from text
    fn extract_json(&self, text: &str) -> String {
        // Find JSON block
        if let Some(start) = text.find('{') {
            if let Some(end) = text.rfind('}') {
                return text[start..=end].to_string();
            }
        }
        text.to_string()
    }

    /// Transform plan to specification for generator
    fn transform_to_specification(&self, plan: &PlanResult, ctx: &RuntimeContext) -> Specification {
        Specification {
            project_background: format!(
                "Project: {}\n\n{}",
                ctx.goal.title, plan.analysis
            ),
            module_breakdown: plan.modules.iter().map(|m| ModuleSpec {
                name: m.name.clone(),
                description: m.description.clone(),
                skill_requirements: vec![],  // Would be derived from analysis
            }).collect(),
            acceptance_criteria: plan.acceptance_criteria.clone(),
        }
    }

    /// Build specification prompt for generator
    fn build_specification_prompt(&self, spec: &Specification) -> String {
        format!(
            r#"You are a code generation agent. Here is your task:

Project Background:
{}

Module Breakdown:
{}

Acceptance Criteria:
{}

Please generate the required code and provide a user manual.
After generation, respond with:
1. A summary of what was created
2. A user manual (USER_MANUAL.md format)
3. Any skills that should be saved for future use"#,
            spec.project_background,
            spec.module_breakdown.iter()
                .map(|m| format!("- {}: {}", m.name, m.description))
                .collect::<Vec<_>>()
                .join("\n"),
            spec.acceptance_criteria.join("\n")
        )
    }
}

#[async_trait]
impl SideCarAgent for BuilderAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Executor  // Builder is a type of executor
    }

    async fn run(
        &self,
        ctx: RuntimeContext,
        transport: Arc<dyn Transport>,
        llm: Arc<dyn LlmProvider>,
        emitter: EventBridge,
    ) -> AppResult<AgentOutput> {
        let mut attempt = 1u32;

        loop {
            if self.cancel_token.is_cancelled() {
                return Ok(AgentOutput::BuilderResult {
                    milestone_id: ctx.goal.current_milestone_id.clone().unwrap_or_default(),
                    plan: None,
                    skills_used: vec![],
                    success: false,
                    failure_reason: Some("Cancelled by user".to_string()),
                    new_node_id: None,
                });
            }

            // Phase 1: Analysis
            let plan = self.analyze(&ctx, &llm, &emitter).await?;

            // Phase 2: Dispatch
            let generator_result = self.dispatch_to_generator(
                &ctx,
                &plan,
                &transport,
                &llm,
                &emitter,
            ).await?;

            // Phase 3: Verification
            let verification = self.verify(&ctx, &generator_result, &llm, &emitter).await?;

            // Phase 4: Conclusion
            let conclusion = self.conclude(&ctx, &verification, attempt, &emitter).await?;

            if conclusion.achieved || attempt >= self.max_retries {
                return Ok(AgentOutput::BuilderResult {
                    milestone_id: ctx.goal.current_milestone_id.clone().unwrap_or_default(),
                    plan: serde_json::to_value(&plan).ok(),
                    skills_used: generator_result.skills.iter().map(|s| s.name.clone()).collect(),
                    success: conclusion.achieved,
                    failure_reason: if conclusion.achieved { None } else { Some(conclusion.message) },
                    new_node_id: conclusion.new_node_id,
                });
            }

            // Store conclusion for retry context
            // In real implementation: WorkHub::set_node_conclusion

            attempt += 1;
        }
    }

    async fn cancel(&self) -> AppResult<()> {
        self.cancel_token.cancel();
        Ok(())
    }
}

/// Plan result from analysis
#[derive(Debug, Clone, serde::Serialize)]
struct PlanResult {
    analysis: String,
    modules: Vec<ModuleInfo>,
    acceptance_criteria: Vec<String>,
}

/// Module information
#[derive(Debug, Clone, serde::Serialize)]
struct ModuleInfo {
    name: String,
    description: String,
    dependencies: Vec<String>,
}

/// Generator task result
#[derive(Debug, Clone)]
struct GeneratorTaskResult {
    success: bool,
    result: String,
    user_manual: Option<String>,
    skills: Vec<SkillRequirement>,
}

/// Verification result
#[derive(Debug)]
struct VerificationResult {
    passed: bool,
    reason: String,
}

/// Conclusion result
#[derive(Debug)]
struct ConclusionResult {
    achieved: bool,
    message: String,
    new_node_id: Option<String>,
}
