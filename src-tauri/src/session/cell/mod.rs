//! SessionCell trait + 公共类型
//!
//! Cell 类似于 Jupyter Notebook 的代码块：划分 session 上下文的单元。
//! SessionCell 是 Cell 的抽象接口。不同 cell_type 的 concrete struct
//! 实现同一个 trait，通过 cell_type() / agent_type() 向外暴露差异信息。

pub mod result;
pub mod goal_summary;
pub mod build;
pub mod executor;
pub mod optimizer;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::workhub::AgentType;
use crate::session::Message;
use self::result::{ResultStatus, StoredResult};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CellType {
    GoalSummary,
    Build,
    Executor,
    Optimizer,
}

impl std::fmt::Display for CellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CellType::GoalSummary => write!(f, "goal_summary"),
            CellType::Build => write!(f, "build"),
            CellType::Executor => write!(f, "executor"),
            CellType::Optimizer => write!(f, "optimizer"),
        }
    }
}

impl TryFrom<&str> for CellType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "goal_summary" => Ok(CellType::GoalSummary),
            "build" => Ok(CellType::Build),
            "executor" => Ok(CellType::Executor),
            "optimizer" => Ok(CellType::Optimizer),
            _ => Err(format!("Unknown cell type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CellStatus {
    Active,
    Closed,
}

/// 前端列表用的摘要信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellInfo {
    pub cell_id: String,
    pub cell_type: CellType,
    pub agent_type: Option<String>,
    pub status: CellStatus,
    pub message_count: usize,
    pub result_status: Option<ResultStatus>,
}

/// SessionCell trait — 会话单元格的抽象接口
///
/// 每个 Cell 内部持有 session: Arc<dyn Session>，
/// 封装消息存取、结果管理、生命周期。
#[async_trait]
pub trait SessionCell: Send + Sync {
    // identity
    fn cell_id(&self) -> &str;
    fn workspace_id(&self) -> &str;
    fn node_id(&self) -> &str;

    // type info
    fn cell_type(&self) -> CellType;
    fn agent_type(&self) -> Option<AgentType>;

    // lifecycle
    fn status(&self) -> CellStatus;
    async fn close(&self);

    // 消息操作（委托给内部 session）
    async fn add_message(&self, role: &str, content: &str);
    fn get_history(&self) -> Vec<Message>;
    fn message_count(&self) -> usize;
    async fn clear(&self);

    // 结果存取
    async fn set_result(&self, result: StoredResult);
    fn get_result(&self) -> Option<StoredResult>;

    // 摘要信息
    fn to_info(&self) -> CellInfo {
        CellInfo {
            cell_id: self.cell_id().to_string(),
            cell_type: self.cell_type(),
            agent_type: self.agent_type().map(|a| a.to_string()),
            status: self.status(),
            message_count: self.message_count(),
            result_status: self.get_result().map(|r| r.status),
        }
    }
}
