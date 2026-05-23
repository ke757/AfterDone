//! InMemorySession — 纯内存会话实现
//!
//! 不持久化，App 重启后丢失。用于 Summarizer 等临时对话场景。

use async_trait::async_trait;
use tokio::sync::RwLock;

use super::Session;
use crate::session::Message;

pub struct InMemorySession {
    messages: RwLock<Vec<Message>>,
}

impl InMemorySession {
    pub fn new() -> Self {
        Self {
            messages: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl Session for InMemorySession {
    async fn add_message(&self, msg: Message) {
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
