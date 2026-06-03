use std::sync::OnceLock;

use async_trait::async_trait;
use futures::StreamExt;
use rig::client::CompletionClient;

use crate::config::LlmConfig;
use crate::error::{AppError, AppResult};
use crate::llm::message::{AssistantMessage, ChatMessage, SystemMessage};
use crate::llm::adapt_rig::adapt_rig_stream;
use crate::llm::events::{StreamEvent, StreamEventStream};
use crate::llm::provider::LlmProvider;
pub struct RigProvider {
    config: LlmConfig,
    client: OnceLock<rig::providers::openai::Client>,
}

impl RigProvider {
    pub fn new(config: LlmConfig) -> Self {
        Self {
            config,
            client: OnceLock::new(),
        }
    }

    fn get_or_init_client(&self) -> AppResult<&rig::providers::openai::Client> {
        if let Some(client) = self.client.get() {
            return Ok(client);
        }

        if self.config.api_key.is_empty() {
            return Err(AppError::Llm("API key not configured".to_string()));
        }

        let mut builder =
            rig::providers::openai::Client::builder().api_key(&self.config.api_key);
        if !self.config.base_url.is_empty() {
            builder = builder.base_url(&self.config.base_url);
        }
        let client = builder.build().map_err(|e| {
            AppError::Llm(format!("Failed to create OpenAI client: {}", e))
        })?;

        let _ = self.client.set(client);
        Ok(self.client.get().unwrap())
    }

    fn build_agent(
        client: &rig::providers::openai::Client,
        config: &LlmConfig,
        system_prompt: &str,
    ) -> rig::agent::Agent<impl rig::completion::CompletionModel> {
        let mut builder = client
            .agent(&config.model)
            .preamble(system_prompt)
            .temperature(config.temperature as f64);
        if config.max_tokens > 0 {
            builder = builder.max_tokens(config.max_tokens as u64);
        }
        builder.build()
    }
}

#[async_trait]
impl LlmProvider for RigProvider {
    /// Non-streaming completion — internally uses streaming to preserve
    /// full AssistantContentBlock fidelity (Text/Thinking/ToolCall).
    async fn complete(
        &self,
        system: &SystemMessage,
        messages: &[ChatMessage],
    ) -> AppResult<AssistantMessage> {
        let mut stream = self.stream(system, messages).await?;

        while let Some(event) = stream.next().await {
            match event {
                Ok(StreamEvent::MessageComplete { message }) => {
                    return Ok(message);
                }
                Ok(StreamEvent::StreamError { message }) => {
                    return Err(AppError::Llm(message));
                }
                Err(e) => return Err(e),
                _ => continue,
            }
        }

        Err(AppError::Llm(
            "Stream ended without MessageComplete".to_string(),
        ))
    }

    async fn stream(
        &self,
        system: &SystemMessage,
        messages: &[ChatMessage],
    ) -> AppResult<StreamEventStream> {
        use rig::streaming::StreamingChat;

        let client = self.get_or_init_client()?;
        let agent = Self::build_agent(client, &self.config, &system.content);

        let rig_messages: Vec<rig::completion::Message> =
            messages.iter().map(chat_message_to_rig).collect();

        let (last, history) = rig_messages.split_last().ok_or_else(|| {
            AppError::Llm("No messages provided".to_string())
        })?;

        let response = agent
            .stream_chat(last.clone(), history.to_vec())
            .await;

        Ok(adapt_rig_stream(response))
    }
    fn model_name(&self) -> &str {
        &self.config.model
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Rig-specific conversion helpers
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// ChatMessage → rig::completion::Message
fn chat_message_to_rig(msg: &ChatMessage) -> rig::completion::Message {
    use crate::llm::message::{AssistantContentBlock, UserContentBlock};
    use rig::completion::Message;

    match msg {
        ChatMessage::User(m) => {
            let text = m
                .content
                .iter()
                .filter_map(|b| match b {
                    UserContentBlock::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");
            Message::user(text)
        }
        ChatMessage::Assistant(m) => {
            let text = m
                .content
                .iter()
                .filter_map(|b| match b {
                    AssistantContentBlock::Text(tc) => Some(tc.text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");
            Message::assistant(text)
        }
        ChatMessage::System(m) => Message::system(&m.content),
        ChatMessage::ToolResult(m) => Message::tool_result(&m.tool_call_id, &m.content),
    }
}
