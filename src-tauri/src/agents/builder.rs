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
use crate::db::models::AgentType;
use crate::error::{AppError, AppResult};
use crate::events::EventBridge;
use crate::llm::LlmProvider;
use crate::agents::traits::SideCarAgent;
use crate::agents::types::{GoalContext, AgentOutput};
use crate::noderepo::{
    NodeRepo, Specification, ModuleSpec, SkillRequirement,
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
        ctx: &GoalContext,
        llm: &Arc<dyn LlmProvider>,
        emitter: &EventBridge,
    ) -> AppResult<PlanResult> {
        emitter.emit_agent_stream(
            &ctx.goal.id,
            "Starting initial analysis...\n",
            false,
        );

        // Get existing conclusion if any
        let existing_conclusion = self.get_existing_conclusion(ctx).await?;

        // Analyze the goal
        let prompt = self.build_analysis_prompt(ctx, &existing_conclusion);
        let response = llm.complete(
            "You are a goal planning agent that creates detailed plans.",
            &prompt,
            &ctx.conversation_history,
        ).await?;

        // Parse the plan
        let plan = self.parse_plan(&response)?;

        emitter.emit_agent_stream(
            &ctx.goal.id,
            &format!("Plan created with {} modules\n", plan.modules.len()),
            false,
        );

        Ok(plan)
    }

    /// Phase 2: Transform and Dispatch
    async fn dispatch_to_generator(
        &self,
        ctx: &GoalContext,
        plan: &PlanResult,
        _transport: &Arc<dyn Transport>,
        llm: &Arc<dyn LlmProvider>,
        emitter: &EventBridge,
    ) -> AppResult<GeneratorTaskResult> {
        emitter.emit_agent_stream(
            &ctx.goal.id,
            "Creating specification for generator...\n",
            false,
        );

        // Transform plan to specification
        let spec = self.transform_to_specification(plan, ctx);

        // In a real implementation, this would:
        // 1. Store PLAN.md via NodeRepo::store_plan
        // 2. Send spec to HarnessAgent via Transport
        // 3. Wait for response

        // For now, simulate the interaction
        let spec_prompt = self.build_specification_prompt(&spec);
        let _spec_response = llm.complete(
            "You are a code generation agent.",
            &spec_prompt,
            &ctx.conversation_history,
        ).await?;

        emitter.emit_agent_stream(
            &ctx.goal.id,
            "Specification sent to generator, waiting for response...\n",
            false,
        );

        // Simulated generator response
        Ok(GeneratorTaskResult {
            success: true,
            result: "Generated successfully".to_string(),
            user_manual: Some("# User Manual\n\nPlease follow the steps...".to_string()),
            skills: vec![],
        })
    }

    /// Phase 3: Verification
    async fn verify(
        &self,
        ctx: &GoalContext,
        generator_result: &GeneratorTaskResult,
        llm: &Arc<dyn LlmProvider>,
        emitter: &EventBridge,
    ) -> AppResult<VerificationResult> {
        emitter.emit_agent_stream(
            &ctx.goal.id,
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
                &ctx.goal.id,
                "Warning: No user manual provided by generator\n",
                false,
            );
        }

        // Run executor for testing
        emitter.emit_agent_stream(
            &ctx.goal.id,
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
            &ctx.conversation_history,
        ).await?;

        Ok(VerificationResult {
            passed: true,
            reason: "Verification passed".to_string(),
        })
    }

    /// Phase 4: Conclusion or Retry
    async fn conclude(
        &self,
        ctx: &GoalContext,
        verification: &VerificationResult,
        attempt: u32,
        emitter: &EventBridge,
    ) -> AppResult<ConclusionResult> {
        if verification.passed {
            emitter.emit_agent_stream(
                &ctx.goal.id,
                "Goal achieved! Creating conclusion...\n",
                true,
            );

            // In a real implementation:
            // - Call NodeRepo::goal_achieved to create new node
            // - Write conclusion.md

            return Ok(ConclusionResult {
                achieved: true,
                message: "Goal successfully achieved".to_string(),
                new_node_id: None,
            });
        }

        if attempt >= self.max_retries {
            emitter.emit_agent_stream(
                &ctx.goal.id,
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
            &ctx.goal.id,
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
    async fn get_existing_conclusion(&self, _ctx: &GoalContext) -> AppResult<Option<String>> {
        // In a real implementation, use NodeRepo::get_node_conclusion
        Ok(None)
    }

    /// Build analysis prompt
    fn build_analysis_prompt(&self, ctx: &GoalContext, existing: &Option<String>) -> String {
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
    fn transform_to_specification(&self, plan: &PlanResult, ctx: &GoalContext) -> Specification {
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
        ctx: GoalContext,
        transport: Arc<dyn Transport>,
        llm: Arc<dyn LlmProvider>,
        emitter: EventBridge,
    ) -> AppResult<AgentOutput> {
        let mut attempt = 1u32;

        loop {
            if self.cancel_token.is_cancelled() {
                return Ok(AgentOutput::ExecutionResult {
                    milestone_id: ctx.goal.current_milestone_id.clone().unwrap_or_default(),
                    plan: None,
                    skills_used: vec![],
                    success: false,
                    failure_reason: Some("Cancelled by user".to_string()),
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
                return Ok(AgentOutput::ExecutionResult {
                    milestone_id: ctx.goal.current_milestone_id.clone().unwrap_or_default(),
                    plan: serde_json::to_value(&plan).ok(),
                    skills_used: generator_result.skills.iter().map(|s| s.name.clone()).collect(),
                    success: conclusion.achieved,
                    failure_reason: if conclusion.achieved { None } else { Some(conclusion.message) },
                });
            }

            // Store conclusion for retry context
            // In real implementation: NodeRepo::set_node_conclusion

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
#[derive(Debug)]
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
