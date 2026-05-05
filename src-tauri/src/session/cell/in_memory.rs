//! InMemoryCell — Summarizer 专用内存 Cell
//!
//! 不持久化，App 重启后丢失。
//! 用户确认后 Goal 通过 SummarizerService 写入 DB。

use tokio::sync::RwLock;
use async_trait::async_trait;

use super::{CellStatus, SessionCell};
use super::result::StoredResult;
use crate::session::Message;
use crate::workhub::AgentType;

pub struct InMemoryCell {
    cell_id: String,
    workspace_id: String,
    node_id: String,
    agent_type: AgentType,
    status: RwLock<CellStatus>,
    messages: RwLock<Vec<Message>>,
    result: RwLock<Option<StoredResult>>,
}

impl InMemoryCell {
    pub fn new(
        cell_id: String,
        workspace_id: String,
        node_id: String,
        agent_type: AgentType,
    ) -> Self {
        Self {
            cell_id,
            workspace_id,
            node_id,
            agent_type,
            status: RwLock::new(CellStatus::Active),
            messages: RwLock::new(Vec::new()),
            result: RwLock::new(None),
        }
    }
}

#[async_trait]
impl SessionCell for InMemoryCell {
    fn cell_id(&self) -> &str { &self.cell_id }
    fn agent_type(&self) -> AgentType { self.agent_type.clone() }
    fn status(&self) -> CellStatus { self.status.blocking_read().clone() }
    fn workspace_id(&self) -> &str { &self.workspace_id }
    fn node_id(&self) -> &str { &self.node_id }

    async fn add_message(&self, role: &str, content: &str) {
        let msg = Message {
            role: role.to_string(),
            content: content.to_string(),
            agent_type: self.agent_type.to_string(),
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

    async fn close(&self) {
        *self.status.write().await = CellStatus::Closed;
    }

    async fn set_result(&self, result: StoredResult) {
        *self.result.write().await = Some(result);
    }

    fn get_result(&self) -> Option<StoredResult> {
        self.result.blocking_read().clone()
    }
}
