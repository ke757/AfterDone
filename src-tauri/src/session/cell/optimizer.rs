//! OptimizerCell — 优化 Cell
//!
//! 绑定 Optimizer Agent，使用 JsonlSession。
//! 对已达成目标进行优化迭代，直到 solidified。

use async_trait::async_trait;
use tokio::sync::RwLock;

use super::{CellStatus, CellType, SessionCell};
use crate::session::{EffectType, SessionLine};
use crate::session::session::Session;
use crate::workhub::AgentType;

pub struct OptimizerCell {
    cell_id: String,
    workspace_id: String,
    node_id: String,
    session: Box<dyn Session>,
    status: RwLock<CellStatus>,
}

impl OptimizerCell {
    pub fn new(
        cell_id: String,
        workspace_id: String,
        node_id: String,
        session: Box<dyn Session>,
    ) -> Self {
        Self {
            cell_id,
            workspace_id,
            node_id,
            session,
            status: RwLock::new(CellStatus::Active),
        }
    }
}

#[async_trait]
impl SessionCell for OptimizerCell {
    fn cell_id(&self) -> &str { &self.cell_id }
    fn workspace_id(&self) -> &str { &self.workspace_id }
    fn node_id(&self) -> &str { &self.node_id }
    fn cell_type(&self) -> CellType { CellType::Optimizer }
    fn agent_type(&self) -> Option<AgentType> { Some(AgentType::Optimizer) }
    fn status(&self) -> CellStatus { self.status.blocking_read().clone() }

    async fn add_message(&self, role: &str, content: &str) {
        let line = SessionLine::Message {
            role: role.to_string(),
            content: content.to_string(),
            agent_type: AgentType::Optimizer.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        self.session.add_line(line).await;
    }

    async fn add_effect(&self, effect_type: EffectType, content: serde_json::Value) {
        let line = SessionLine::Effect {
            effect_type,
            agent_type: AgentType::Optimizer.to_string(),
            content,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        self.session.add_line(line).await;
    }

    fn get_lines(&self) -> Vec<SessionLine> { self.session.get_lines() }
    fn line_count(&self) -> usize { self.session.line_count() }
    async fn clear(&self) { self.session.clear().await; }
    async fn truncate(&self, at_index: usize) { self.session.truncate(at_index).await; }
    async fn close(&self) { *self.status.write().await = CellStatus::Closed; }
    fn latest_effect(&self, effect_type: EffectType) -> Option<SessionLine> {
        self.session.latest_effect(effect_type)
    }
}
