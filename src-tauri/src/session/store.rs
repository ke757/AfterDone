//! In-memory store for `ChatMemory`, keyed by goal_id.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use rig::completion::message::Message;
use crate::chat::Message as DbMessage;
use super::{ChatMemory, rig_message_to_db};

/// Thread-safe in-memory implementation of [`ChatMemory`].
///
/// Each goal maintains its own independent conversation history.
/// All operations are O(n) for context building and O(1) for append.
#[derive(Debug, Clone)]
pub struct InMemoryChatMemory {
    inner: Arc<RwLock<HashMap<String, Vec<Message>>>>,
}

impl InMemoryChatMemory {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryChatMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatMemory for InMemoryChatMemory {
    fn add_message(&self, goal_id: &str, message: Message) {
        let mut map = self.inner.blocking_write();
        map.entry(goal_id.to_string())
            .or_default()
            .push(message);
    }

    fn get_history(&self, goal_id: &str) -> Vec<Message> {
        let map = self.inner.blocking_read();
        map.get(goal_id).cloned().unwrap_or_default()
    }

    fn clear(&self, goal_id: &str) {
        let mut map = self.inner.blocking_write();
        map.remove(goal_id);
    }

    fn has_history(&self, goal_id: &str) -> bool {
        let map = self.inner.blocking_read();
        map.get(goal_id).map(|v| !v.is_empty()).unwrap_or(false)
    }

    fn message_count(&self, goal_id: &str) -> usize {
        let map = self.inner.blocking_read();
        map.get(goal_id).map(|v| v.len()).unwrap_or(0)
    }

    fn to_db_context(&self, goal_id: &str, goal_id_for_db: &str) -> Vec<DbMessage> {
        let map = self.inner.blocking_read();
        match map.get(goal_id) {
            Some(messages) => messages
                .iter()
                .map(|m| rig_message_to_db(m, goal_id_for_db))
                .collect(),
            None => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rig::completion::message::Message as RigMessage;

    #[test]
    fn test_add_and_retrieve_messages() {
        let memory = InMemoryChatMemory::new();

        memory.add_message("goal-1", RigMessage::user("hello"));
        memory.add_message("goal-1", RigMessage::assistant("world"));

        assert_eq!(memory.message_count("goal-1"), 2);
        assert_eq!(memory.message_count("goal-2"), 0);

        let history = memory.get_history("goal-1");
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_clear_history() {
        let memory = InMemoryChatMemory::new();

        memory.add_message("goal-1", RigMessage::user("hello"));
        assert!(memory.has_history("goal-1"));

        memory.clear("goal-1");
        assert!(!memory.has_history("goal-1"));
        assert_eq!(memory.message_count("goal-1"), 0);
    }

    #[test]
    fn test_isolation_between_goals() {
        let memory = InMemoryChatMemory::new();

        memory.add_message("goal-1", RigMessage::user("msg1"));
        memory.add_message("goal-2", RigMessage::user("msg2"));

        assert_eq!(memory.message_count("goal-1"), 1);
        assert_eq!(memory.message_count("goal-2"), 1);

        memory.clear("goal-1");
        assert_eq!(memory.message_count("goal-1"), 0);
        assert_eq!(memory.message_count("goal-2"), 1);
    }

    #[test]
    fn test_to_db_context() {
        let memory = InMemoryChatMemory::new();

        memory.add_message("goal-1", RigMessage::system("You are a helper"));
        memory.add_message("goal-1", RigMessage::user("Hello"));
        memory.add_message("goal-1", RigMessage::assistant("Hi there"));

        let ctx = memory.to_db_context("goal-1", "goal-1");
        assert_eq!(ctx.len(), 3);
        assert_eq!(ctx[0].role, "system");
        assert_eq!(ctx[0].content, "You are a helper");
        assert_eq!(ctx[1].role, "user");
        assert_eq!(ctx[1].content, "Hello");
        assert_eq!(ctx[2].role, "assistant");
        assert_eq!(ctx[2].content, "Hi there");
    }
}