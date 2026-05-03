use async_trait::async_trait;
use futures::Stream;

use crate::config::LlmConfig;
use crate::session::Message;
use crate::error::{AppError, AppResult};
use crate::llm::provider::{LLMStream, LlmProvider, StreamChunk};

/// Rig-based implementation of LlmProvider.
/// Uses rig-core for LLM completions.
pub struct RigProvider {
    config: LlmConfig,
}

impl RigProvider {
    pub fn new(config: LlmConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl LlmProvider for RigProvider {
    async fn complete(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        context: &[Message],
    ) -> AppResult<String> {
        let full_prompt = build_prompt_from_context(context, user_prompt);
        self.call_rig(system_prompt, &full_prompt).await
    }

    async fn stream(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        context: &[Message],
    ) -> AppResult<LLMStream> {
        let full_prompt = build_prompt_from_context(context, user_prompt);
        let result = self.call_rig(system_prompt, &full_prompt).await?;

        let stream = futures::stream::once(async move {
            Ok(StreamChunk {
                delta: result,
                finished: true,
            })
        });

        Ok(Box::pin(stream))
    }

    fn model_name(&self) -> &str {
        &self.config.model
    }
}

impl RigProvider {
    /// Make an LLM call using rig-core.
    async fn call_rig(&self, system_prompt: &str, user_prompt: &str) -> AppResult<String> {
        use rig::client::CompletionClient;  // Required for .agent() method in rig v0.33+
        use rig::completion::Prompt;
        use rig::providers::openai::Client;

        if self.config.api_key.is_empty() {
            return Err(AppError::Llm("API key not configured".to_string()));
        }

        // Create OpenAI client (rig v0.33+ returns Result)
        let client = Client::new(&self.config.api_key)
            .map_err(|e| AppError::Llm(format!("Failed to create OpenAI client: {}", e)))?;

        // Build an agent with system prompt as preamble
        let agent = client.agent(&self.config.model)
            .preamble(system_prompt)
            .build();

        // Send the prompt and get the response
        let response = agent
            .prompt(user_prompt)
            .await
            .map_err(|e| AppError::Llm(format!("LLM call failed: {}", e)))?;

        Ok(response)
    }
}

/// Build a combined prompt from message history and current user input
fn build_prompt_from_context(context: &[Message], user_prompt: &str) -> String {
    let mut parts = Vec::new();

    for msg in context {
        let role_label = match msg.role.as_str() {
            "user" => "User",
            "agent_summarizer" => "Summarizer",
            "agent_executor" => "Executor",
            "agent_optimizer" => "Optimizer",
            "system" => "System",
            _ => "Assistant",
        };
        parts.push(format!("[{}]: {}", role_label, msg.content));
    }

    parts.push(format!("[User]: {}", user_prompt));
    parts.join("\n\n")
}
