use async_trait::async_trait;
use std::sync::Arc;

use rig::completion::message::Message as RigMessage;

use crate::agents::traits::SideCarAgent;
use crate::agents::types::{AgentOutput, GoalContext};
use crate::adapter::Transport;
use crate::workhub::{AgentType, GoalSummary};
use crate::error::{AppError, AppResult};
use crate::events::EventBridge;
use crate::llm::{LlmProvider, PromptTemplate};
use crate::memory::ChatMemory;

/// Goal Summarization Agent.
/// Supports multi-turn conversation: each invocation appends user input
/// to the goal's memory, and the LLM receives the full conversation
/// history so it can refine the summary incrementally.
pub struct GoalSummarizerAgent {
    cancel_token: tokio_util::sync::CancellationToken,
    memory: Arc<dyn ChatMemory>,
}

impl GoalSummarizerAgent {
    pub fn new(cancel_token: tokio_util::sync::CancellationToken, memory: Arc<dyn ChatMemory>) -> Self {
        Self { cancel_token, memory }
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

        emitter.emit_agent_status(goal_id, "summarizer", "running", "summarizing");

        // 将 user message 添加到 memory
        self.memory.add_message(
            goal_id,
            RigMessage::user(&ctx.goal.raw_input),
        );

        // 从 memory 中构建对话历史
        let history = self.memory.get_history(goal_id);
        let context = self.memory.to_db_context(goal_id, goal_id);

        let system_prompt = PromptTemplate::system_prompt(&AgentType::Summarizer);

        // If there's prior history (multi-turn), include a refinement instruction
        let user_prompt = if history.len() > 1 {
            format!(
                "[Conversation history shows {} prior exchanges.]\n\n\
                 Latest user input:\n{}\n\n\
                 Please update the goal summary considering all previous context. \
                 Incorporate the new information while preserving the structure.",
                history.len() - 1,
                ctx.goal.raw_input,
            )
        } else {
            format!(
                "Please summarize the following goal description:\n\n{}",
                ctx.goal.raw_input,
            )
        };
        
        // 使用流式处理向前端展示进度
        let mut stream = llm.stream(system_prompt, &user_prompt, &context).await?;
        let mut full_response = String::new();

        use futures::StreamExt;
        while let Some(chunk) = stream.next().await {
            if self.cancel_token.is_cancelled() {
                emitter.emit_agent_status(goal_id, "summarizer", "canceled", "summarizing");
                self.memory.clear(goal_id);
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

        // Store assistant response in memory for future turns
        self.memory.add_message(
            goal_id,
            RigMessage::assistant(&full_response),
        );

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
fn parse_summary_response(response: &str) -> AppResult<GoalSummary> {
    // 尝试从响应中提取JSON（可能被包裹在Markdown代码块中）
    let json_str = extract_json(response);

    match serde_json::from_str::<GoalSummary>(&json_str) {
        Ok(summary) => Ok(summary),
        Err(_) => Ok(GoalSummary {
            title: "Goal".to_string(),
            description: response.trim().to_string(),
            acceptance_criteria: vec!["Goal must be verifiably completed".to_string()],
            constraints: vec![],
            refinement_questions: vec![
                "Could you provide more details about this goal?".to_string()
            ],
        }),
    }
}

/// Extract JSON from a potentially markdown-wrapped response
fn extract_json(text: &str) -> String {
    let trimmed = text.trim();

    // 检查是否包含Markdown代码块换行
    if let Some(start) = trimmed.find("```json") {
        if let Some(end) = trimmed.rfind("```") {
            let json_start = start + 7;
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

    // 检查原始JSON对象
    if trimmed.starts_with('{') {
        if let Some(end) = trimmed.rfind('}') {
            return trimmed[..=end].to_string();
        }
    }

    trimmed.to_string()
}