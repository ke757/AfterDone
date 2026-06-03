use async_trait::async_trait;
use std::sync::Arc;

use futures::StreamExt;

use crate::agents::traits::SideCarAgent;
use crate::agents::types::{AgentOutput, RuntimeContext};
use crate::adapter::Transport;
use crate::workhub::{AgentType, GoalSummary};
use crate::error::{AppError, AppResult};
use crate::events::EventBridge;
use crate::llm::{
    LlmProvider,
    message::{ChatMessage, SystemMessage},
    stream::{StreamEventStream, EventBridgeStreamSink, StreamSink},
};
use crate::session::convert::session_lines_to_chat_messages;
use super::prompt::PromptTemplate;

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

        let system = SystemMessage {
            content: PromptTemplate::system_prompt(&AgentType::Summarizer).to_string(),
        };

        // 构造 user prompt：包含目标标题作为上下文
        let user_prompt = format!(
            "Goal context: the user is defining a goal titled \"{}\".\n\n\
             Please review the conversation history above and respond conversationally. \
             If you have enough information, produce the goal summary JSON. \
             If not, ask clarifying questions.",
            ctx.goal.title
        );

        let history = ctx.cell.get_lines();
        let mut messages = session_lines_to_chat_messages(&history);
        messages.push(ChatMessage::user(&user_prompt));

        let mut stream: StreamEventStream = llm.stream(&system, &messages).await?;
        let sink = EventBridgeStreamSink::new(emitter.clone(), workspace_id);
        let mut full_response = String::new();

        while let Some(event) = stream.next().await {
            if self.cancel_token.is_cancelled() {
                emitter.emit_agent_status(workspace_id, "summarizer", "canceled", "summarizing");
                ctx.cell.clear().await;
                return Err(AppError::Agent(
                    "Summarizer was canceled".to_string(),
                ));
            }

            let event = event?;

            // Accumulate text for session persistence
            if let crate::llm::events::StreamEvent::ContentBlockDelta {
                delta: crate::llm::events::ContentDelta::Text(ref text),
                ..
            } = &event
            {
                full_response.push_str(text);
            }

            sink.feed(event).await?;
        }

        // 记录 assistant 响应到 session
        ctx.cell.add_message("assistant", &full_response).await;
        let summary_draft = try_parse_goal_summary(&full_response);

        emitter.emit_agent_status(workspace_id, "summarizer", "completed", "summarizing");

        let has_summary = summary_draft.is_some();
        emitter.emit_agent_decision(
            workspace_id,
            if has_summary { "summary_ready" } else { "conversation_turn" },
            &full_response,
            summary_draft.as_ref().and_then(|s| serde_json::to_value(s).ok()),
        );

        Ok(AgentOutput::ConversationTurn {
            message: full_response,
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
