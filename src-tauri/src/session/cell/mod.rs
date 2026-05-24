//! SessionCell trait + 公共类型
//!
//! Cell 类似于 Jupyter Notebook 的代码块：划分 session 上下文的单元。
//! SessionCell 是 Cell 的抽象接口。不同 cell_type 的 concrete struct
//! 实现同一个 trait，通过 cell_type() / agent_type() 向外暴露差异信息。

pub mod goal_summary;
pub mod build;
pub mod executor;
pub mod optimizer;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::workhub::AgentType;
use crate::session::{EffectType, SessionLine};


/// Cell类型名定义
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
    pub line_count: usize,
    pub has_result: bool,
    pub last_result_at: Option<String>,
}

/// SessionCell trait — 会话单元格的抽象接口
///
/// 每个 Cell 内部持有 session: Arc<dyn Session>，
/// 封装消息/effect 存取、结果管理、生命周期。
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

    // session lines CRUD
    /// 追加一条对话消息到 session。
    /// `role` 为 "user" / "assistant" / "system"，`content` 为消息正文。
    async fn add_message(&self, role: &str, content: &str);

    /// 追加一条副作用记录到 session（agent 不可见，仅后端/前端使用）。
    /// `effect_type` 区分 result / file_change 等语义类别，`content` 为自定义载荷。
    async fn add_effect(&self, effect_type: EffectType, content: serde_json::Value);

    /// 返回 session 中所有行（Message + Effect），按追加顺序排列。
    fn get_lines(&self) -> Vec<SessionLine>;

    /// 当前 session 的总行数（仅 Message + Effect，不含 JSONL 首行 CellMeta）。
    fn line_count(&self) -> usize;

    /// 清空所有 Message 和 Effect 行（保留 JSONL 首行 CellMeta）。
    async fn clear(&self);

    /// 从指定行号截断，移除 >=at_index 的所有行（0-indexed，仅针对 SessionLine）。
    /// 用于 redo 场景：用户编辑消息后丢弃后续内容重新生成。
    async fn truncate(&self, at_index: usize);

    /// 从 session 中反向查找最新一条指定类型的 Effect 行。
    /// 常用于获取最近一次 Agent 运行的结果摘要（EffectType::Result）。
    fn latest_effect(&self, effect_type: EffectType) -> Option<SessionLine>;

    // 摘要信息
    fn to_info(&self) -> CellInfo {
        let lines = self.get_lines();
        let last_result = lines
            .iter()
            .rev()
            .find(|l| matches!(l, SessionLine::Effect { effect_type: EffectType::Result, .. }));
        CellInfo {
            cell_id: self.cell_id().to_string(),
            cell_type: self.cell_type(),
            agent_type: self.agent_type().map(|a| a.to_string()),
            status: self.status(),
            line_count: self.line_count(),
            has_result: last_result.is_some(),
            last_result_at: last_result.map(|l| l.created_at_str().to_string()),
        }
    }
}
