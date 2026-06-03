use async_trait::async_trait;

use crate::error::AppResult;
use crate::events::EventBridge;
use crate::llm::events::{ContentDelta, StreamEvent};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// StreamSink — StreamEvent 消费者的通用抽象
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// StreamSink 接收 StreamEvent 并路由到合适的输出通道。
/// Agent 通过此 trait 消费流事件，无需关心底层传输（Tauri Event / WebSocket / Mock）。
#[async_trait]
pub trait StreamSink: Send + Sync {
    async fn on_text_delta(&self, text: &str);
    async fn on_thinking_delta(&self, thinking: &str);
    async fn on_tool_call_start(&self, index: usize, id: &str, name: Option<&str>);
    async fn on_tool_call_args(&self, index: usize, args: &str);
    async fn on_block_end(&self, index: usize);
    async fn on_message_complete(&self, full_text: &str);
    async fn on_error(&self, message: &str);

    /// 消费一个 StreamEvent，分发到各处理方法
    async fn feed(&self, event: StreamEvent) -> AppResult<()> {
        match event {
            StreamEvent::ContentBlockStart { .. }
            | StreamEvent::ContentBlockStop { .. } => {
                // 生命周期事件由具体实现按需处理
            }
            StreamEvent::ContentBlockDelta { index: _, delta } => match delta {
                ContentDelta::Text(text) => self.on_text_delta(&text).await,
                ContentDelta::Thinking(thinking) => self.on_thinking_delta(&thinking).await,
                ContentDelta::ToolCallName(name) => {
                    self.on_tool_call_start(0, "", Some(&name)).await
                }
                ContentDelta::ToolCallArguments(args) => {
                    self.on_tool_call_args(0, &args).await
                }
                ContentDelta::Signature(_sig) => {}
            },
            StreamEvent::MessageComplete { message } => {
                let text = message.extract_text();
                self.on_message_complete(&text).await;
            }
            StreamEvent::StreamError { message } => {
                self.on_error(&message).await;
            }
        }
        Ok(())
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// EventBridgeStreamSink — 将 StreamEvent 转发到 Tauri 前端
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub struct EventBridgeStreamSink {
    bridge: EventBridge,
    workspace_id: String,
}

impl EventBridgeStreamSink {
    pub fn new(bridge: EventBridge, workspace_id: impl Into<String>) -> Self {
        Self {
            bridge,
            workspace_id: workspace_id.into(),
        }
    }
}

#[async_trait]
impl StreamSink for EventBridgeStreamSink {
    async fn on_text_delta(&self, text: &str) {
        self.bridge
            .emit_agent_stream(&self.workspace_id, text, false);
    }

    async fn on_thinking_delta(&self, thinking: &str) {
        self.bridge
            .emit_agent_stream(&self.workspace_id, thinking, false);
    }

    async fn on_tool_call_start(&self, _index: usize, _id: &str, _name: Option<&str>) {
        // 前端尚未支持 tool call 渲染，暂不推送
    }

    async fn on_tool_call_args(&self, _index: usize, _args: &str) {
        // 前端尚未支持 tool call 渲染，暂不推送
    }

    async fn on_block_end(&self, _index: usize) {}

    async fn on_message_complete(&self, _full_text: &str) {
        self.bridge
            .emit_agent_stream(&self.workspace_id, "", true);
    }

    async fn on_error(&self, message: &str) {
        self.bridge
            .emit_agent_stream(&self.workspace_id, message, false);
    }
}
