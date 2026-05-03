use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::loader::config_dir;
use crate::error::AppResult;
use super::store::InMemorySession;
use super::jsonl_store::NodeSession;
use super::Session;

/// SessionManager 负责根据 Agent 类型路由到不同的会话实现
///
/// - Summarizer → InMemorySession (临时内存会话，按 workspace_id)
/// - Builder / Executor / Optimizer → NodeSession (持久化 JSONL，按 node_id)
pub struct SessionManager {
    nodes_dir: PathBuf,
    temp_sessions: RwLock<HashMap<String, Arc<InMemorySession>>>,
    node_sessions: RwLock<HashMap<String, Arc<NodeSession>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        let nodes_dir = config_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("nodes");

        Self {
            nodes_dir,
            temp_sessions: RwLock::new(HashMap::new()),
            node_sessions: RwLock::new(HashMap::new()),
        }
    }

    /// 获取或创建 Summarizer 临时内存会话
    pub async fn get_or_create_temp(&self, workspace_id: &str) -> Arc<dyn Session> {
        let mut sessions = self.temp_sessions.write().await;
        if let Some(session) = sessions.get(workspace_id) {
            return session.clone();
        }
        let session = Arc::new(InMemorySession::new(workspace_id.to_string()));
        sessions.insert(workspace_id.to_string(), session.clone());
        session
    }

    /// 获取或创建 Node 持久化会话
    pub fn get_or_create_node(&self, node_id: &str) -> AppResult<Arc<dyn Session>> {
        let mut sessions = self.node_sessions.blocking_write();
        if let Some(session) = sessions.get(node_id) {
            return Ok(session.clone());
        }
        let session = Arc::new(NodeSession::new(node_id, &self.nodes_dir)?);
        sessions.insert(node_id.to_string(), session.clone());
        Ok(session)
    }

    /// 清除 Summarizer 临时会话
    pub async fn clear_temp(&self, workspace_id: &str) {
        let mut sessions = self.temp_sessions.write().await;
        sessions.remove(workspace_id);
    }
}
