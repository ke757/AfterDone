use std::pin::Pin;

use futures::Stream;
use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::llm::message::AssistantMessage;

/// 流事件流的类型别名
pub type StreamEventStream =
    Pin<Box<dyn Stream<Item = AppResult<StreamEvent>> + Send>>;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// StreamEvent — Content Block 生命周期事件（前端接收的结构化流协议）
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// 流式传输事件，遵循 Anthropic/OpenAI 的 "Content Block" 生命周期模型：
/// ContentBlockStart → ContentBlockDelta* → ContentBlockStop
/// 最后一个 ContentBlockStop 之后 → MessageComplete
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum StreamEvent {
    /// 一个新的内容块开始
    ContentBlockStart {
        index: usize,
        block: ContentBlockHeader,
    },
    /// 内容块中的增量数据
    ContentBlockDelta {
        index: usize,
        delta: ContentDelta,
    },
    /// 内容块传输完成
    ContentBlockStop {
        index: usize,
    },
    /// 整条消息完成，携带完整组装后的 AssistantMessage
    MessageComplete {
        message: AssistantMessage,
    },
    /// 流异常终止
    StreamError {
        message: String,
    },
}

/// 内容块头部信息（标识块类型，不含积累内容）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ContentBlockHeader {
    Text,
    Thinking,
    ToolCall {
        id: String,
        /// name 在首次 NameDelta 前为 None
        name: Option<String>,
    },
}

/// 增量数据 — 内容块内的最小传输单元
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ContentDelta {
    /// 文本 token
    Text(String),
    /// 思考/推理 token
    Thinking(String),
    /// 工具调用名称片段
    ToolCallName(String),
    /// 工具调用 JSON 参数片段
    ToolCallArguments(String),
    /// 签名片段（Anthropic thinking signature）
    Signature(String),
}
