use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;

use crate::session::Message;
use super::Session;

/// Summarizer 临时内存会话 — 不持久化，Agent 结束后即丢弃
#[derive(Debug, Clone)]
pub struct InMemorySession {
    pub workspace_id: String,
    messages: Arc<RwLock<Vec<Message>>>,
}

impl InMemorySession {
    pub fn new(workspace_id: String) -> Self {
        Self {
            workspace_id,
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl Session for InMemorySession {
    async fn add_message(&self, role: &str, content: &str, agent_type: &str) {
        let msg = Message {
            role: role.to_string(),
            content: content.to_string(),
            agent_type: agent_type.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        self.messages.write().await.push(msg);
    }

    fn get_history(&self) -> Vec<Message> {
        self.messages.blocking_read().clone()
    }

    fn message_count(&self) -> usize {
        self.messages.blocking_read().len()
    }

    async fn clear(&self) {
        self.messages.write().await.clear();
    }
}
