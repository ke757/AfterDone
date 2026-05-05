//! Agent 执行结果类型（从 memory 模块迁入）
//!
//! Cell 自带 result: Option<StoredResult>，替代独立的 ResultMemory。

use serde::{Deserialize, Serialize};
use crate::workhub::AgentType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ResultStatus {
    Active,
    Ready,
    Confirmed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredResult {
    pub session_id: String,
    pub workspace_id: String,
    pub goal_id: String,
    pub agent_type: AgentType,
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
