//! Agent Result Memory — in-memory store for agent execution results
//!
//! session_id 作为主键，每个 session 存储有序的结果列表（最新在最后）。
//! 前端通过 session_id 查询最新的 agent 执行结果。

use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::workhub::AgentType;

/// 统一的结果状态（替代旧的 TurnStatus）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ResultStatus {
    /// 对话/处理进行中
    Active,
    /// 结果就绪，等待用户确认
    Ready,
    /// 用户已确认 / 已持久化
    Confirmed,
    /// 执行失败
    Failed,
    /// 已取消
    Cancelled,
}

/// 统一的结果存储结构 — 所有 Agent 类型的输出统一为此格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredResult {
    pub session_id: String,
    pub workspace_id: String,
    pub goal_id: String,
    pub agent_type: AgentType,
    /// AgentOutput 的序列化 JSON
    pub output: serde_json::Value,
    pub status: ResultStatus,
    pub created_at: String,
}

impl StoredResult {
    pub fn new(
        session_id: String,
        workspace_id: String,
        goal_id: String,
        agent_type: AgentType,
        output: serde_json::Value,
        status: ResultStatus,
    ) -> Self {
        Self {
            session_id,
            workspace_id,
            goal_id,
            agent_type,
            output,
            status,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Agent 执行结果的内存存储
///
/// key = session_id，value = 该 session 的有序结果列表。
/// 每次 agent turn 的输出追加到列表末尾。
pub struct ResultMemory {
    entries: RwLock<HashMap<String, Vec<StoredResult>>>,
}

impl ResultMemory {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }

    /// 存储一条结果（追加到 session 的列表末尾）
    pub async fn store(&self, result: StoredResult) {
        let mut entries = self.entries.write().await;
        entries
            .entry(result.session_id.clone())
            .or_default()
            .push(result);
    }

    /// 获取指定 session 的最新结果
    pub async fn get_latest(&self, session_id: &str) -> Option<StoredResult> {
        let entries = self.entries.read().await;
        entries.get(session_id).and_then(|v| v.last().cloned())
    }

    /// 获取指定 session 的所有结果
    pub async fn get_all(&self, session_id: &str) -> Vec<StoredResult> {
        let entries = self.entries.read().await;
        entries.get(session_id).cloned().unwrap_or_default()
    }

    /// 将指定 session 的最新结果标记为 Confirmed
    pub async fn mark_confirmed(&self, session_id: &str) {
        let mut entries = self.entries.write().await;
        if let Some(list) = entries.get_mut(session_id) {
            if let Some(last) = list.last_mut() {
                last.status = ResultStatus::Confirmed;
            }
        }
    }
}
