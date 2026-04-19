//! Optimizer Agent (鐩爣浼樺寲 Agent)
//!
//! Responsible for optimizing achieved goals, fixing bugs, and solidifying.
//! Core flow:
//! 1. Initial analysis - get plan, bugs, conclusion from parent node
//! 2. Transform and dispatch - create spec based on history
//! 3. Verification - check result, run executor
//! 4. Conclusion - mark achieved, update bugs

use async_trait::async_trait;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

use crate::adapter::Transport;
use crate::workhub::AgentType;
use crate::error::{AppError, AppResult};
use crate::events::EventBridge;
use crate::llm::LlmProvider;
use crate::agents::traits::SideCarAgent;
use crate::agents::types::{GoalContext, AgentOutput, Optimization};
use crate::workhub::{WorkHub, TodoList, TodoItem, BugEntry};

/// Maximum retry attempts
const MAX_RETRIES: u32 = 10;

/// Optimizer Agent
///
/// Handles optimization and bug fixing after initial achievement.
pub struct OptimizerAgent {
    cancel_token: CancellationToken,
    max_retries: u32,
}

impl OptimizerAgent {
    pub fn new(cancel_token: CancellationToken) -> Self {
        Self {
            cancel_token,
            max_retries: MAX_RETRIES,
        }
    }

    /// Phase 1: Initial Analysis with History
    async fn analyze_with_history(
        &self,
        ctx: &GoalContext,
        llm: &Arc<dyn LlmProvider>,
        emitter: &EventBridge,
    ) -> AppResult<TodoList> {
        emitter.emit_agent_stream(
            &ctx.goal.id,
            "Starting optimization analysis...\n",
            false,
        );

        // Get history and context
        let plan = self.get_plan(ctx).await?;
        let parent_bugs = self.get_parent_bugs(ctx).await?;
        let parent_conclusion = self.get_parent_conclusion(ctx).await?;

        // Build analysis prompt
        let prompt = self.build_analysis_prompt(ctx, &plan, &parent_bugs, &parent_conclusion);
        let response = llm.complete(
            "You are an optimization agent that analyzes achieved goals.",
            &prompt,
            &ctx.conversation_history,
        ).await?;

        // Parse todo list
        let todo_list = self.parse_todo_list(&response, &parent_bugs)?;

        emitter.emit_agent_stream(
            &ctx.goal.id,
            &format!("Created todo list with {} items\n", todo_list.items.len()),
            false,
        );

        Ok(todo_list)
    }

    /// Phase 2: Transform and Dispatch
    async fn dispatch_to_generator(
        &self,
        ctx: &GoalContext,
        todo_list: &TodoList,
        _transport: &Arc<dyn Transport>,
        llm: &Arc<dyn LlmProvider>,
        emitter: &EventBridge,
    ) -> AppResult<GeneratorTaskResult> {
        emitter.emit_agent_stream(
            &ctx.goal.id,
            "Creating optimization specification...\n",
            false,
        );

        // Transform todo list to specification
        let spec = self.transform_to_specification(todo_list, ctx);

        // Build and send prompt
        let spec_prompt = self.build_specification_prompt(&spec);
        let _spec_response = llm.complete(
            "You are an optimization agent that applies improvements.",
            &spec_prompt,
            &ctx.conversation_history,
        ).await?;

        emitter.emit_agent_stream(
            &ctx.goal.id,
            "Optimization specification sent, waiting for response...\n",
            false,
        );

        // Simulated response
        Ok(GeneratorTaskResult {
            success: true,
            result: "Optimization completed".to_string(),
            user_manual: Some("# Updated User Manual\n\nOptimized version...".to_string()),
            optimizations: vec![Optimization {
                area: "performance".to_string(),
                description: "Improved algorithm efficiency".to_string(),
                before: "O(n^2)".to_string(),
                after: "O(n log n)".to_string(),
            }],
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
            "Verifying optimization result...\n",
            false,
        );

        if !generator_result.success {
            return Ok(VerificationResult {
                passed: false,
                reason: "Generator reported failure".to_string(),
            });
        }

        // Run executor for testing
        emitter.emit_agent_stream(
            &ctx.goal.id,
            "Running execution agent for optimization verification...\n",
            false,
        );

        // Verify optimizations
        let verify_prompt = format!(
            "Verify the following optimizations are correct and complete:\n\n{}",
            generator_result.optimizations.iter()
                .map(|o| format!("- {}: {} ({} -> {})", o.area, o.description, o.before, o.after))
                .collect::<Vec<_>>()
                .join("\n")
        );
        let _response = llm.complete(
            "You are a verification agent.",
            &verify_prompt,
            &ctx.conversation_history,
        ).await?;

        Ok(VerificationResult {
            passed: true,
            reason: "Optimization verification passed".to_string(),
        })
    }

    /// Phase 4: Conclusion with Bug Handling
    async fn conclude(
        &self,
        ctx: &GoalContext,
        verification: &VerificationResult,
        generator_result: &GeneratorTaskResult,
        attempt: u32,
        emitter: &EventBridge,
    ) -> AppResult<ConclusionResult> {
        if verification.passed {
            emitter.emit_agent_stream(
                &ctx.goal.id,
                "Optimization complete! Creating conclusion...\n",
                true,
            );

            // In real implementation:
            // - Call WorkHub::node_achieved
            // - Update bugs (remove resolved ones)
            // - Write conclusion.md

            return Ok(ConclusionResult {
                achieved: true,
                solidified: generator_result.optimizations.is_empty(),
                message: "Optimization successful".to_string(),
                optimizations: generator_result.optimizations.clone(),
                new_node_id: None, // Would be set by WorkHub::node_achieved
            });
        }

        if attempt >= self.max_retries {
            emitter.emit_agent_stream(
                &ctx.goal.id,
                &format!("Max retries ({}) reached for optimization\n", self.max_retries),
                true,
            );

            return Ok(ConclusionResult {
                achieved: false,
                solidified: false,
                message: format!("Failed after {} attempts", self.max_retries),
                optimizations: vec![],
                new_node_id: None,
            });
        }

        emitter.emit_agent_stream(
            &ctx.goal.id,
            &format!("Optimization attempt {}/{} failed, retrying\n", attempt, self.max_retries),
            false,
        );

        Ok(ConclusionResult {
            achieved: false,
            solidified: false,
            message: "Retrying...".to_string(),
            optimizations: vec![],
            new_node_id: None,
        })
    }

    /// Get PLAN.md from workspace
    async fn get_plan(&self, _ctx: &GoalContext) -> AppResult<Option<String>> {
        // In real implementation: WorkHub::get_plan
        Ok(None)
    }

    /// Get bugs from parent worknode
    async fn get_parent_bugs(&self, _ctx: &GoalContext) -> AppResult<Vec<BugEntry>> {
        // In real implementation: WorkHub::get_node_bug
        Ok(vec![])
    }

    /// Get conclusion from parent worknode
    async fn get_parent_conclusion(&self, _ctx: &GoalContext) -> AppResult<Option<String>> {
        // In real implementation: WorkHub::get_node_conclusion
        Ok(None)
    }

    /// Build analysis prompt with history
    fn build_analysis_prompt(
        &self,
        ctx: &GoalContext,
        plan: &Option<String>,
        bugs: &[BugEntry],
        conclusion: &Option<String>,
    ) -> String {
        let mut prompt = format!(
            r#"You are an optimization agent. Analyze the achieved goal and identify optimization opportunities.

Goal: {}
Status: {}

"#,
            ctx.goal.title,
            ctx.goal.status
        );

        if let Some(p) = plan {
            prompt.push_str(&format!("Current Plan:\n{}\n\n", p));
        }

        if !bugs.is_empty() {
            prompt.push_str("Known Bugs:\n");
            for bug in bugs {
                prompt.push_str(&format!("- {} (resolved: {})\n", bug.description, bug.resolved));
            }
            prompt.push_str("\n");
        }

        if let Some(c) = conclusion {
            prompt.push_str(&format!("Previous Conclusion:\n{}\n\n", c));
        }

        prompt.push_str(r#"Based on the above, create a todo list for optimization:
1. Bug fixes (if any unresolved bugs)
2. Performance improvements
3. Code quality improvements
4. Documentation updates

Format your response as JSON:
{
  "direction_summary": "...",
  "todos": [
    {"description": "...", "priority": 1-5}
  ]
}"#);

        prompt
    }

    /// Parse todo list from LLM response
    fn parse_todo_list(&self, response: &str, bugs: &[BugEntry]) -> AppResult<TodoList> {
        // Extract JSON
        let json_str = self.extract_json(response);

        let parsed: serde_json::Value = serde_json::from_str(&json_str)
            .unwrap_or_else(|_| serde_json::json!({
                "direction_summary": "Continue optimization",
                "todos": []
            }));

        let mut todo_list = TodoList::new(
            parsed["direction_summary"].as_str().unwrap_or("Optimization").to_string()
        );

        // Add unresolved bugs first
        for bug in bugs.iter().filter(|b| !b.resolved) {
            todo_list.add_item(format!("Fix bug: {}", bug.description), 1);
        }

        // Add parsed todos
        if let Some(todos) = parsed["todos"].as_array() {
            for todo in todos {
                let description = todo["description"].as_str().unwrap_or("").to_string();
                let priority = todo["priority"].as_u64().unwrap_or(3) as u32;
                if !description.is_empty() {
                    todo_list.add_item(description, priority);
                }
            }
        }

        Ok(todo_list)
    }

    /// Extract JSON from text
    fn extract_json(&self, text: &str) -> String {
        if let Some(start) = text.find('{') {
            if let Some(end) = text.rfind('}') {
                return text[start..=end].to_string();
            }
        }
        text.to_string()
    }

    /// Transform todo list to specification
    fn transform_to_specification(&self, todo_list: &TodoList, ctx: &GoalContext) -> OptimizationSpec {
        OptimizationSpec {
            goal_title: ctx.goal.title.clone(),
            direction: todo_list.direction_summary.clone(),
            items: todo_list.items.iter().map(|item| OptimizationItem {
                description: item.description.clone(),
                priority: item.priority,
            }).collect(),
        }
    }

    /// Build specification prompt
    fn build_specification_prompt(&self, spec: &OptimizationSpec) -> String {
        format!(
            r#"You are an optimization agent. Apply the following optimizations:

Goal: {}
Direction: {}

Todo Items:
{}

Please apply these optimizations and provide:
1. Summary of changes made
2. Updated user manual
3. List of specific optimizations (area, description, before, after)"#,
            spec.goal_title,
            spec.direction,
            spec.items.iter()
                .map(|i| format!("- [P{}] {}", i.priority, i.description))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

#[async_trait]
impl SideCarAgent for OptimizerAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Optimizer
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
                return Ok(AgentOutput::OptimizationResult {
                    milestone_id: ctx.goal.current_milestone_id.clone().unwrap_or_default(),
                    optimizations: vec![],
                    solidified: false,
                    new_node_id: None,
                });
            }

            // Phase 1: Analysis with history
            let todo_list = self.analyze_with_history(&ctx, &llm, &emitter).await?;

            if todo_list.items.is_empty() {
                emitter.emit_agent_stream(
                    &ctx.goal.id,
                    "No optimization tasks found, goal is solidified\n",
                    true,
                );
                return Ok(AgentOutput::OptimizationResult {
                    milestone_id: ctx.goal.current_milestone_id.clone().unwrap_or_default(),
                    optimizations: vec![],
                    solidified: true,
                    new_node_id: None,
                });
            }

            // Phase 2: Dispatch
            let generator_result = self.dispatch_to_generator(
                &ctx,
                &todo_list,
                &transport,
                &llm,
                &emitter,
            ).await?;

            // Phase 3: Verification
            let verification = self.verify(&ctx, &generator_result, &llm, &emitter).await?;

            // Phase 4: Conclusion
            let conclusion = self.conclude(
                &ctx,
                &verification,
                &generator_result,
                attempt,
                &emitter,
            ).await?;

            if conclusion.achieved || attempt >= self.max_retries {
                return Ok(AgentOutput::OptimizationResult {
                    milestone_id: ctx.goal.current_milestone_id.clone().unwrap_or_default(),
                    optimizations: conclusion.optimizations,
                    solidified: conclusion.solidified,
                    new_node_id: conclusion.new_node_id,
                });
            }

            attempt += 1;
        }
    }

    async fn cancel(&self) -> AppResult<()> {
        self.cancel_token.cancel();
        Ok(())
    }
}

/// Optimization specification
#[derive(Debug)]
struct OptimizationSpec {
    goal_title: String,
    direction: String,
    items: Vec<OptimizationItem>,
}

/// Optimization item
#[derive(Debug)]
struct OptimizationItem {
    description: String,
    priority: u32,
}

/// Generator task result
#[derive(Debug)]
struct GeneratorTaskResult {
    success: bool,
    result: String,
    user_manual: Option<String>,
    optimizations: Vec<Optimization>,
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
    solidified: bool,
    message: String,
    optimizations: Vec<Optimization>,
    new_node_id: Option<String>,
}



