use async_trait::async_trait;
use futures::Stream;

use crate::db::models::Message;
use crate::error::AppResult;

/// A streaming chunk from the LLM
pub struct StreamChunk {
    pub delta: String,
    pub finished: bool,
}

/// Type alias for the LLM stream
pub type LLMStream = std::pin::Pin<Box<dyn Stream<Item = AppResult<StreamChunk>> + Send>>;

/// LLM provider trait — abstracts over different LLM backends.
/// The primary implementation wraps the `rig` crate.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Non-streaming completion
    async fn complete(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        context: &[Message],
    ) -> AppResult<String>;

    /// Streaming completion
    async fn stream(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        context: &[Message],
    ) -> AppResult<LLMStream>;

    /// Get the model name being used
    fn model_name(&self) -> &str;
}
