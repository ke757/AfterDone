use async_trait::async_trait;
use std::sync::Arc;

use crate::agents::traits::SideCarAgent;
use crate::agents::types::{AgentOutput, GoalContext};
use crate::comm::Transport;
use crate::db::models::{AgentType, GoalSummary};
use crate::error::{AppError, AppResult};
use crate::events::EventBridge;
use crate::llm::{LlmProvider, PromptTemplate};

/// Goal Summarization Agent.
/// Takes raw user input, generates a structured goal summary and refinement questions.
pub struct GoalSummarizerAgent {
    cancel_token: tokio_util::sync::CancellationToken,
}

impl GoalSummarizerAgent {
    pub fn new(cancel_token: tokio_util::sync::CancellationToken) -> Self {
        Self { cancel_token }
    }
}

#[async_trait]
impl SideCarAgent for GoalSummarizerAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Summarizer
    }

    async fn run(
        &self,
        ctx: GoalContext,
        _transport: Arc<dyn Transport>,
        llm: Arc<dyn LlmProvider>,
        emitter: EventBridge,
    ) -> AppResult<AgentOutput> {
        let goal_id = &ctx.goal.id;

        // Emit status: started
        emitter.emit_agent_status(goal_id, "summarizer", "running", "summarizing");

        let system_prompt = PromptTemplate::system_prompt(&AgentType::Summarizer);
        let user_prompt = format!(
            "Please summarize the following goal description:\n\n{}",
            ctx.goal.raw_input
        );

        // Use streaming to show progress to the frontend
        let mut stream = llm.stream(system_prompt, &user_prompt, &[]).await?;
        let mut full_response = String::new();

        use futures::StreamExt;
        while let Some(chunk) = stream.next().await {
            if self.cancel_token.is_cancelled() {
                emitter.emit_agent_status(goal_id, "summarizer", "canceled", "summarizing");
                return Err(AppError::Agent("Summarizer was canceled".to_string()));
            }

            match chunk {
                Ok(chunk) => {
                    if !chunk.delta.is_empty() {
                        emitter.emit_agent_stream(goal_id, &chunk.delta, false);
                        full_response.push_str(&chunk.delta);
                    }
                    if chunk.finished {
                        emitter.emit_agent_stream(goal_id, "", true);
                    }
                }
                Err(e) => {
                    emitter.emit_agent_status(goal_id, "summarizer", "error", "summarizing");
                    return Err(e);
                }
            }
        }

        // Parse the LLM response as a GoalSummary
        let summary = parse_summary_response(&full_response)?;

        emitter.emit_agent_status(goal_id, "summarizer", "completed", "summarizing");
        emitter.emit_agent_decision(
            goal_id,
            "goal_summarized",
            "Generated structured goal summary",
            serde_json::to_value(&summary).ok(),
        );

        Ok(AgentOutput::GoalSummary {
            title: summary.title,
            description: summary.description,
            acceptance_criteria: summary.acceptance_criteria,
            constraints: summary.constraints,
            refinement_questions: summary.refinement_questions,
        })
    }

    async fn cancel(&self) -> AppResult<()> {
        self.cancel_token.cancel();
        Ok(())
    }
}

/// Parse the LLM response into a GoalSummary.
/// The LLM is prompted to return JSON, but we handle common issues.
fn parse_summary_response(response: &str) -> AppResult<GoalSummary> {
    // Try to extract JSON from the response (may be wrapped in markdown code blocks)
    let json_str = extract_json(response);

    match serde_json::from_str::<GoalSummary>(&json_str) {
        Ok(summary) => Ok(summary),
        Err(_) => {
            // Fallback: create a basic summary from the raw response
            Ok(GoalSummary {
                title: "Goal".to_string(),
                description: response.trim().to_string(),
                acceptance_criteria: vec!["Goal must be verifiably completed".to_string()],
                constraints: vec![],
                refinement_questions: vec![
                    "Could you provide more details about this goal?".to_string()
                ],
            })
        }
    }
}

/// Extract JSON from a potentially markdown-wrapped response
fn extract_json(text: &str) -> String {
    let trimmed = text.trim();

    // Check for markdown code block wrapping
    if let Some(start) = trimmed.find("```json") {
        if let Some(end) = trimmed.rfind("```") {
            let json_start = start + 7; // skip "```json"
            return trimmed[json_start..end].trim().to_string();
        }
    }

    if let Some(start) = trimmed.find("```") {
        if let Some(end) = trimmed.rfind("```") {
            if start != end {
                let json_start = start + 3;
                return trimmed[json_start..end].trim().to_string();
            }
        }
    }

    // Check for raw JSON object
    if trimmed.starts_with('{') {
        if let Some(end) = trimmed.rfind('}') {
            return trimmed[..=end].to_string();
        }
    }

    trimmed.to_string()
}
