//! ExecutorCell — 执行 Cell
//!
//! 绑定 Executor Agent，使用 JsonlSession。
//! 执行生成的代码/脚本，收集 bug 并反馈。

use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;

use super::{CellStatus, CellType, SessionCell};
use super::result::StoredResult;
use crate::session::Message;
use crate::session::session::Session;
use crate::workhub::AgentType;

pub struct ExecutorCell {
    cell_id: String,
    workspace_id: String,
    node_id: String,
    session: Arc<dyn Session>,
    status: RwLock<CellStatus>,
    result: RwLock<Option<StoredResult>>,
}

impl ExecutorCell {
    pub fn new(
        cell_id: String,
        workspace_id: String,
        node_id: String,
        session: Arc<dyn Session>,
    ) -> Self {
        Self {
            cell_id,
            workspace_id,
            node_id,
            session,
            status: RwLock::new(CellStatus::Active),
            result: RwLock::new(None),
        }
    }
}

#[async_trait]
impl SessionCell for ExecutorCell {
    fn cell_id(&self) -> &str { &self.cell_id }
    fn workspace_id(&self) -> &str { &self.workspace_id }
    fn node_id(&self) -> &str { &self.node_id }
    fn cell_type(&self) -> CellType { CellType::Executor }
    fn agent_type(&self) -> Option<AgentType> { Some(AgentType::Executor) }
    fn status(&self) -> CellStatus { self.status.blocking_read().clone() }

    async fn add_message(&self, role: &str, content: &str) {
        let msg = Message {
            role: role.to_string(),
            content: content.to_string(),
            agent_type: AgentType::Executor.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        self.session.add_message(msg).await;
    }

    fn get_history(&self) -> Vec<Message> { self.session.get_history() }
    fn message_count(&self) -> usize { self.session.message_count() }
    async fn clear(&self) { self.session.clear().await; }
    async fn close(&self) { *self.status.write().await = CellStatus::Closed; }
    async fn set_result(&self, result: StoredResult) { *self.result.write().await = Some(result); }
    fn get_result(&self) -> Option<StoredResult> { self.result.blocking_read().clone() }
}
