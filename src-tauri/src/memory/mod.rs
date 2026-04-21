//! Memory Module
//!
//! In-memory chat memory using rig-core's `Message` type.
//! Supports multi-turn conversations per goal, allowing agents
//! to carry context across successive invocations.

mod store;

pub use store::InMemoryChatMemory;

use rig::completion::message::Message;
use crate::chat::Message as DbMessage;

/// Trait for agent memory — stores conversation history per goal.
pub trait ChatMemory: Send + Sync {
    /// Add a message to the conversation memory for the given goal.
    fn add_message(&self, goal_id: &str, message: Message);

    /// Get the full conversation history for a goal.
    fn get_history(&self, goal_id: &str) -> Vec<Message>;

    /// Clear conversation history for a goal.
    fn clear(&self, goal_id: &str);

    /// Check if memory exists for a goal.
    fn has_history(&self, goal_id: &str) -> bool;

    /// Number of messages stored for a goal.
    fn message_count(&self, goal_id: &str) -> usize;

    /// Convert rig Message history to DB-compatible Message context
    /// for use with the LlmProvider interface.
    fn to_db_context(&self, goal_id: &str, goal_id_for_db: &str) -> Vec<DbMessage>;
}

/// Convert a rig-core `Message` into the role string used by our DB `Message`.
fn rig_message_to_db(msg: &Message, goal_id: &str) -> DbMessage {
    let (role, content) = match msg {
        Message::System { content } => ("system".to_string(), content.clone()),
        Message::User { content: inner } => {
            let text = inner
                .iter()
                .filter_map(|c| match c {
                    rig::completion::message::UserContent::Text(t) => Some(t.text.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");
            ("user".to_string(), text)
        }
        Message::Assistant { content: inner, .. } => {
            let text = inner
                .iter()
                .filter_map(|c| match c {
                    rig::completion::message::AssistantContent::Text(t) => Some(t.text.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");
            ("assistant".to_string(), text)
        }
    };

    DbMessage {
        id: uuid::Uuid::new_v4().to_string(),
        goal_id: goal_id.to_string(),
        role,
        content,
        metadata: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    }
}