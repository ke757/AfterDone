//! SessionCell — 会话单元格抽象
//!
//! Cell 类似于 Jupyter Notebook 的代码块：划分 session 上下文的单元。
//! 不同 cell_type 对应不同的前端交互和后端行为。
//!
//! 两种实现：
//! - InMemoryCell: Summarizer 专用，纯内存
//! - JsonlCell: Builder/Executor/Optimizer 专用，JSONL 持久化

pub mod manager;
pub mod in_memory;
pub mod jsonl_cell;
pub mod result;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::workhub::AgentType;
use crate::session::Message;
use self::result::{ResultStatus, StoredResult};

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
    pub agent_type: AgentType,
    pub status: CellStatus,
    pub message_count: usize,
    pub result_status: Option<ResultStatus>,
}

/// SessionCell trait — 会话单元格的抽象接口
///
/// 每个 Cell 内部持有 session，封装消息存取、结果管理、生命周期。
#[async_trait]
pub trait SessionCell: Send + Sync {
    fn cell_id(&self) -> &str;
    fn agent_type(&self) -> AgentType;
    fn status(&self) -> CellStatus;
    fn workspace_id(&self) -> &str;
    fn node_id(&self) -> &str;

    // 消息操作（委托给内部 session）
    async fn add_message(&self, role: &str, content: &str);
    fn get_history(&self) -> Vec<Message>;
    fn message_count(&self) -> usize;
    async fn clear(&self);

    // 生命周期
    async fn close(&self);

    // 结果存取
    async fn set_result(&self, result: StoredResult);
    fn get_result(&self) -> Option<StoredResult>;

    // 摘要信息
    fn to_info(&self) -> CellInfo {
        CellInfo {
            cell_id: self.cell_id().to_string(),
            agent_type: self.agent_type(),
            status: self.status(),
            message_count: self.message_count(),
            result_status: self.get_result().map(|r| r.status),
        }
    }
}
