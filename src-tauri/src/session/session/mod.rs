//! Session trait — 会话消息的存取抽象
//!
//! 提供统一的 add_message / get_history / clear 接口，
//! 由 InMemorySession 或 JsonlSession 实现不同的持久化策略。

pub mod in_memory;
pub mod jsonl;

use async_trait::async_trait;
use crate::session::Message;

#[async_trait]
pub trait Session: Send + Sync {
    async fn add_message(&self, msg: Message);
    fn get_history(&self) -> Vec<Message>;
    fn message_count(&self) -> usize;
    async fn clear(&self);
}
