use async_trait::async_trait;
use std::sync::Arc;

use crate::agents::traits::SideCarAgent;
use crate::agents::types::{AgentOutput, RuntimeContext};
use crate::adapter::Transport;
use crate::workhub::{AgentType, GoalSummary};
use crate::error::{AppError, AppResult};
use crate::events::EventBridge;
use crate::llm::{LlmProvider, PromptTemplate};

/// Goal Summarization Agent — 对话式产品分析师
///
/// 从 session 中读取完整对话历史。每次 `run()` 处理一轮对话：将最新 user 消息 + 历史上下文发给 LLM。
///
/// 对话流程由 Supervisor 的 `deliver_message()` 驱动：
///   用户发消息 → deliver_message → run() → 返回 ConversationTurn
///   用户继续对话 → deliver_message → run() → 返回 ConversationTurn
///   用户确认 → confirm_summary → 持久化到 DB
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
        ctx: RuntimeContext,
        _transport: Arc<dyn Transport>,
        llm: Arc<dyn LlmProvider>,
        emitter: EventBridge,
    ) -> AppResult<AgentOutput> {
        let workspace_id = ctx.workspace_id.as_deref().unwrap_or(&ctx.goal.id);

        emitter.emit_agent_status(workspace_id, "summarizer", "running", "summarizing");

        let history = ctx.cell.get_history();

        let system_prompt = PromptTemplate::system_prompt(&AgentType::Summarizer);

        // 构造 user prompt：包含目标标题作为上下文
        let user_prompt = format!(
            "Goal context: the user is defining a goal titled \"{}\".\n\n\
             Please review the conversation history above and respond conversationally. \
             If you have enough information, produce the goal summary JSON. \
             If not, ask clarifying questions.",
            ctx.goal.title
        );

        // LLM 流式调用
        let mut stream = llm.stream(system_prompt, &user_prompt, &history).await?;
        let mut full_response = String::new();

        use futures::StreamExt;
        while let Some(chunk) = stream.next().await {
            if self.cancel_token.is_cancelled() {
                emitter.emit_agent_status(workspace_id, "summarizer", "canceled", "summarizing");
                ctx.cell.clear().await;
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
        ctx.cell.add_message("assistant", &full_response).await;

        // 尝试解析 GoalSummary
        let summary_draft = try_parse_goal_summary(&full_response);

        emitter.emit_agent_status(workspace_id, "summarizer", "completed", "summarizing");

        let message = full_response;
        let has_summary = summary_draft.is_some();

        emitter.emit_agent_decision(
            workspace_id,
            if has_summary { "summary_ready" } else { "conversation_turn" },
            &message,
            summary_draft.as_ref().and_then(|s| serde_json::to_value(s).ok()),
        );

        Ok(AgentOutput::ConversationTurn {
            message,
            summary_draft,
        })
    }

    async fn cancel(&self) -> AppResult<()> {
        self.cancel_token.cancel();
        Ok(())
    }
}

/// 尝试从 LLM 响应中解析 GoalSummary。
/// 如果能解析出完整的 JSON → 返回 Some；否则返回 None（说明这是一条普通对话消息）。
fn try_parse_goal_summary(response: &str) -> Option<GoalSummary> {
    let json_str = extract_json(response);

    serde_json::from_str::<GoalSummary>(&json_str).ok()
}

/// 从文本中提取 JSON（支持 ```json 代码块和裸 JSON）
fn extract_json(text: &str) -> String {
    let trimmed = text.trim();

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

    if trimmed.starts_with('{') {
        if let Some(end) = trimmed.rfind('}') {
            return trimmed[..=end].to_string();
        }
    }

    trimmed.to_string()
}
