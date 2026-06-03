use async_trait::async_trait;

use crate::error::AppResult;
use crate::llm::message::{AssistantMessage, SystemMessage, ChatMessage};
use crate::llm::events::StreamEventStream;

/// LLM provider trait — abstracts over different LLM backends.
/// Uses typed ChatMessage for input and produces AssistantMessage / StreamEventStream.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Non-streaming completion — returns a fully assembled AssistantMessage
    async fn complete(
        &self,
        system: &SystemMessage,
        messages: &[ChatMessage],
    ) -> AppResult<AssistantMessage>;

    /// Streaming completion — returns a stream of StreamEvent (ContentBlock lifecycle)
    async fn stream(
        &self,
        system: &SystemMessage,
        messages: &[ChatMessage],
    ) -> AppResult<StreamEventStream>;

    /// Get the model name being used（预留）
    fn model_name(&self) -> &str;
}
