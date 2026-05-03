use async_trait::async_trait;
use std::sync::Arc;

use crate::agents::traits::SideCarAgent;
use crate::agents::types::{AgentOutput, GoalContext};
use crate::adapter::Transport;
use crate::workhub::{AgentType, GoalSummary};
use crate::error::{AppError, AppResult};
use crate::events::EventBridge;
use crate::llm::{LlmProvider, PromptTemplate};

/// Goal Summarization Agent.
/// Supports multi-turn conversation: each invocation appends user input
/// to the goal's session, and the LLM receives the full conversation
/// history so it can refine the summary incrementally.
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
        let workspace_id = ctx.workspace_id.as_deref().unwrap_or(&ctx.goal.id);

        emitter.emit_agent_status(workspace_id, "summarizer", "running", "summarizing");

        // 记录用户输入到 session
        ctx.session.add_message("user", &ctx.goal.raw_input, "summarizer").await;

        let history = ctx.session.get_history();
        let msg_count = ctx.session.message_count();

        let system_prompt = PromptTemplate::system_prompt(&AgentType::Summarizer);

        // If there's prior history (multi-turn), include a refinement instruction
        let user_prompt = if msg_count > 1 {
            format!(
                "[Conversation history shows {} prior exchanges.]\n\n\
                 Latest user input:\n{}\n\n\
                 Please update the goal summary considering all previous context. \
                 Incorporate the new information while preserving the structure.",
                msg_count - 1,
                ctx.goal.raw_input,
            )
        } else {
            format!(
                "Please summarize the following goal description:\n\n{}",
                ctx.goal.raw_input,
            )
        };

        // 使用流式处理向前端展示进度
        let mut stream = llm.stream(system_prompt, &user_prompt, &history).await?;
        let mut full_response = String::new();

        use futures::StreamExt;
        while let Some(chunk) = stream.next().await {
            if self.cancel_token.is_cancelled() {
                emitter.emit_agent_status(workspace_id, "summarizer", "canceled", "summarizing");
                ctx.session.clear().await;
                return Err(AppError::Agent("Summarizer was canceled".to_string()));
            }

            match chunk {
                Ok(chunk) => {
                    if !chunk.delta.is_empty() {
                        emitter.emit_agent_stream(workspace_id, &chunk.delta, false);
                        full_response.push_str(&chunk.delta);
                    }
                    if chunk.finished {
                        emitter.emit_agent_stream(workspace_id, "", true);
                    }
                }
                Err(e) => {
                    emitter.emit_agent_status(workspace_id, "summarizer", "error", "summarizing");
                    return Err(e);
                }
            }
        }

        // 记录 assistant 响应到 session
        ctx.session.add_message("assistant", &full_response, "summarizer").await;

        let summary = parse_summary_response(&full_response)?;

        emitter.emit_agent_status(workspace_id, "summarizer", "completed", "summarizing");
        emitter.emit_agent_decision(
            workspace_id,
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
    // 将大型语言模型（LLM）的响应解析为 GoalSummary。
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
    // 从可能被Markdown包裹的响应中提取JSON
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