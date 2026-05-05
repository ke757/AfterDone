use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::workhub::{AgentType, Goal, Milestone, Skill, AgentLog};
use crate::session::Session;

/// Context passed to any agent on each run.
/// Constructed by AgentSupervisor from the database before spawning.
#[derive(Clone)]
pub struct RuntimeContext {
    pub goal: Goal,
    pub current_milestone: Option<Milestone>,
    pub available_skills: Vec<Skill>,
    pub agent_logs: Vec<AgentLog>,
    pub metadata: HashMap<String, serde_json::Value>,
    /// WorkSpace ID for the goal (if initialized)
    pub workspace_id: Option<String>,
    /// Current WorkNode ID (if any)
    pub current_node_id: Option<String>,
    /// Agent session — records conversation history per scope
    pub session: Arc<dyn Session>,
}

/// Output from an agent run
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentOutput {
    /// 一轮对话的中间结果：Agent 回复了消息，Summary 可能就绪或未就绪
    /// 用于 deliver_message 模式下的多轮对话
    ConversationTurn {
        message: String,
        /// 如果 LLM 已生成完整 GoalSummary JSON，此处有值
        summary_draft: Option<crate::workhub::GoalSummary>,
    },
    /// Output from the Summarizer agent (确认后的最终结果)
    GoalSummary {
        title: String,
        description: String,
        acceptance_criteria: Vec<String>,
        constraints: Vec<String>,
        refinement_questions: Vec<String>,
    },
    /// Output from the Builder agent (原型构建)
    BuilderResult {
        milestone_id: String,
        plan: Option<serde_json::Value>,
        skills_used: Vec<String>,
        success: bool,
        failure_reason: Option<String>,
        /// New worknode created (if successful)
        new_node_id: Option<String>,
    },
    /// Output from the Executor agent
    ExecutionResult {
        milestone_id: String,
        plan: Option<serde_json::Value>,
        skills_used: Vec<String>,
        success: bool,
        failure_reason: Option<String>,
        /// Bugs found during execution
        bugs_found: Vec<BugInfo>,
    },
    /// Output from the Optimizer agent
    OptimizationResult {
        milestone_id: String,
        optimizations: Vec<Optimization>,
        solidified: bool,
        /// New worknode created (if successful)
        new_node_id: Option<String>,
    },
}

/// Bug information from execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BugInfo {
    pub description: String,
    pub error_output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Optimization {
    pub area: String,
    pub description: String,
    pub before: String,
    pub after: String,
}

/// Status of a running agent task
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Starting,       // 已收到，安排开始
    Running,        // 正在运行
    Canceling,
    Completed,
    Failed,
}

/// Handle to a running agent task
pub struct AgentTaskHandle {
    pub agent_type: AgentType,
    pub cancel_token: tokio_util::sync::CancellationToken,
    pub status: AgentStatus,
    pub started_at: chrono::DateTime<chrono::Utc>,
}
