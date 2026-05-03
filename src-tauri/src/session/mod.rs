//! Session Module — 重构后的会话系统
//!
//! 两种会话模式:
//! - InMemorySession: Summarizer 临时内存会话（不持久化）
//! - NodeSession: Builder/Executor/Optimizer 关联 WorkNode 的持久化会话（JSONL）
//!
//! 参考 OpenClaw 的 session 设计: 每 node 独立会话, JSONL 持久化

pub mod types;
pub mod store;
pub mod jsonl_store;
pub mod manager;

use async_trait::async_trait;

pub use types::Message;
pub use manager::SessionManager;

/// Session trait — Agent 对话会话接口
///
/// 替代旧的 ChatMemory trait。
/// 会话在创建时绑定 scope（workspace_id 或 node_id），
/// 因此方法签名中不再需要 goal_id 参数。
#[async_trait]
pub trait Session: Send + Sync {
    /// 添加一条消息到会话
    async fn add_message(&self, role: &str, content: &str, agent_type: &str);

    /// 获取完整会话历史
    fn get_history(&self) -> Vec<Message>;

    /// 消息数量
    fn message_count(&self) -> usize;

    /// 清空会话
    async fn clear(&self);
}
