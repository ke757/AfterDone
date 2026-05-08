use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ==================== Goal Types ====================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "snake_case")]
pub enum GoalStatus {
    Draft,
    Pinned,
    Building,
    Reached,
    Optimizing,
    Archived,
    Failed,
}

impl Default for GoalStatus {
    fn default() -> Self {
        GoalStatus::Draft
    }
}

impl std::fmt::Display for GoalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GoalStatus::Draft => write!(f, "draft"),
            GoalStatus::Pinned => write!(f, "pinned"),
            GoalStatus::Building => write!(f, "building"),
            GoalStatus::Reached => write!(f, "reached"),
            GoalStatus::Optimizing => write!(f, "optimizing"),
            GoalStatus::Archived => write!(f, "archived"),
            GoalStatus::Failed => write!(f, "failed"),
        }
    }
}

impl TryFrom<&str> for GoalStatus {
    type Error = String;
    fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
        match s {
            "draft" => Ok(GoalStatus::Draft),
            "pinned" => Ok(GoalStatus::Pinned),
            "building" => Ok(GoalStatus::Building),
            "reached" => Ok(GoalStatus::Reached),
            "optimizing" => Ok(GoalStatus::Optimizing),
            "archived" => Ok(GoalStatus::Archived),
            "failed" => Ok(GoalStatus::Failed),
            _ => Err(format!("Unknown goal status: {}", s)),
        }
    }
}

/// Goal
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Goal {
    pub id: String,
    pub title: String,              // 目标的标题，由 Summarizer agent 提炼生成
    pub summary: Option<String>,    // 目标的摘要/简介，由 Summarizer agent 提炼生成
    pub content: String,          // 目标的内容原文
    pub status: GoalStatus,         // 目标的当前状态
    pub created_at: String,
    pub updated_at: String,
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGoalInput {
    pub title: Option<String>,
    pub summary: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalSummary {
    pub title: String,
    pub description: String,                // 描述
    pub acceptance_criteria: Vec<String>,   // 验收标准
    pub constraints: Vec<String>,           // 约束
    pub refinement_questions: Vec<String>,  // 细化问题
}

// ==================== AgentLog Types ====================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    Summarizer,
    Builder,
    Executor,
    Optimizer,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::Summarizer => write!(f, "summarizer"),
            AgentType::Builder => write!(f, "builder"),
            AgentType::Executor => write!(f, "executor"),
            AgentType::Optimizer => write!(f, "optimizer"),
        }
    }
}

impl TryFrom<&str> for AgentType {
    type Error = String;
    fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
        match s {
            "summarizer" => Ok(AgentType::Summarizer),
            "builder" => Ok(AgentType::Builder),
            "executor" => Ok(AgentType::Executor),
            "optimizer" => Ok(AgentType::Optimizer),
            _ => Err(format!("Unknown agent type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AgentLog {
    pub id: String,
    pub goal_id: String,
    pub agent_type: String,
    pub phase: String,
    pub event_type: String,
    pub payload: Option<String>,
    pub created_at: String,
}

// ==================== WorkSpace / WorkNode Types ====================

/// Bug entry stored in BUG.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BugEntry {
    pub id: String,
    pub description: String,
    pub error_output: Option<String>,
    pub created_at: String,
    pub resolved: bool,
}

impl BugEntry {
    pub fn new(description: String, error_output: Option<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            description,
            error_output,
            created_at: chrono::Utc::now().to_rfc3339(),
            resolved: false,
        }
    }
}

/// WorkSpace: the workspace for a goal
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkSpace {
    pub id: String,
    pub goal_id: String,
    pub current_node_id: Option<String>,
    pub goal_md: Option<String>,
    pub plan_md: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// WorkNode: a milestone node in the node tree
///
/// node_type: "root" for workspace root, "version" for work version nodes
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkNode {
    pub id: String,
    pub workspace_id: String,
    pub parent_node_id: Option<String>,
    pub node_order: i32,
    pub node_type: String,
    pub status: String,
    pub bug_md: Option<String>,
    pub user_manual_md: Option<String>,
    pub conclusion_md: Option<String>,
    pub plan_summary: Option<String>,
    pub result_summary: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// WorkNode status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WorkNodeStatus {
    Planned,
    Running,
    Completed,
    Failed,
}

impl std::fmt::Display for WorkNodeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkNodeStatus::Planned => write!(f, "planned"),
            WorkNodeStatus::Running => write!(f, "running"),
            WorkNodeStatus::Completed => write!(f, "completed"),
            WorkNodeStatus::Failed => write!(f, "failed"),
        }
    }
}

impl TryFrom<&str> for WorkNodeStatus {
    type Error = String;
    fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
        match s {
            "planned" => Ok(WorkNodeStatus::Planned),
            "running" => Ok(WorkNodeStatus::Running),
            "completed" => Ok(WorkNodeStatus::Completed),
            "failed" => Ok(WorkNodeStatus::Failed),
            _ => Err(format!("Unknown worknode status: {}", s)),
        }
    }
}

/// GeneratorTask: tracks HarnessAgent tasks
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GeneratorTask {
    pub id: String,
    pub workspace_id: String,
    pub worknode_id: Option<String>,
    pub specification: String,
    pub status: String,
    pub result: Option<String>,
    pub user_manual: Option<String>,
    pub skills_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// GeneratorTask status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GeneratorTaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl std::fmt::Display for GeneratorTaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GeneratorTaskStatus::Pending => write!(f, "pending"),
            GeneratorTaskStatus::Running => write!(f, "running"),
            GeneratorTaskStatus::Completed => write!(f, "completed"),
            GeneratorTaskStatus::Failed => write!(f, "failed"),
        }
    }
}

/// Create WorkSpace input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkSpaceInput {
    pub goal_id: String,
    pub goal_md: Option<String>,
    pub plan_md: Option<String>,
}

/// Create WorkNode input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkNodeInput {
    pub workspace_id: String,
    pub parent_node_id: Option<String>,
    pub node_order: i32,
    pub milestone_id: Option<String>,
    /// 自定义 status（不填则默认 Planned）
    pub status: Option<String>,
    /// 节点类型: "root" | "version"（默认 "version"）
    pub node_type: Option<String>,
}

/// workspace_init 的返回结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceInitResult {
    pub workspace: WorkSpace,
    pub root_node_id: String,
    pub goal: Goal,
}

/// Create GeneratorTask input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGeneratorTaskInput {
    pub workspace_id: String,
    pub worknode_id: Option<String>,
    pub specification: String,
}

/// Update GeneratorTask input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGeneratorTaskInput {
    pub status: Option<String>,
    pub result: Option<String>,
    pub user_manual: Option<String>,
    pub skills_json: Option<String>,
}

/// Node tree structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkNodeTree {
    pub node: WorkNode,
    pub children: Vec<WorkNodeTree>,
}

/// Specification document for HarnessAgent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Specification {
    pub project_background: String,
    pub module_breakdown: Vec<ModuleSpec>,
    pub acceptance_criteria: Vec<String>,
}

/// Module specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSpec {
    pub name: String,
    pub description: String,
    pub skill_requirements: Vec<SkillRequirement>,
}

/// Skill requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRequirement {
    pub name: String,
    pub description: String,
    pub parameters_schema: Option<serde_json::Value>,
}

/// TodoList for OptimizerAgent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoList {
    pub direction_summary: String,
    pub items: Vec<TodoItem>,
}

/// Todo item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: String,
    pub description: String,
    pub priority: u32,
    pub completed: bool,
}

impl TodoList {
    pub fn new(direction_summary: String) -> Self {
        Self {
            direction_summary,
            items: Vec::new(),
        }
    }

    pub fn add_item(&mut self, description: String, priority: u32) {
        self.items.push(TodoItem {
            id: uuid::Uuid::new_v4().to_string(),
            description,
            priority,
            completed: false,
        });
    }
}
