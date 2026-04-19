use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ==================== Goal Types ====================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "snake_case")]
pub enum GoalStatus {
    Draft,
    Summarizing,
    Pinned,
    Executing,
    Achieved,
    Optimizing,
    Solidified,
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
            GoalStatus::Summarizing => write!(f, "summarizing"),
            GoalStatus::Pinned => write!(f, "pinned"),
            GoalStatus::Executing => write!(f, "executing"),
            GoalStatus::Achieved => write!(f, "achieved"),
            GoalStatus::Optimizing => write!(f, "optimizing"),
            GoalStatus::Solidified => write!(f, "solidified"),
            GoalStatus::Failed => write!(f, "failed"),
        }
    }
}

impl TryFrom<&str> for GoalStatus {
    type Error = String;
    fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
        match s {
            "draft" => Ok(GoalStatus::Draft),
            "summarizing" => Ok(GoalStatus::Summarizing),
            "pinned" => Ok(GoalStatus::Pinned),
            "executing" => Ok(GoalStatus::Executing),
            "achieved" => Ok(GoalStatus::Achieved),
            "optimizing" => Ok(GoalStatus::Optimizing),
            "solidified" => Ok(GoalStatus::Solidified),
            "failed" => Ok(GoalStatus::Failed),
            _ => Err(format!("Unknown goal status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Goal {
    pub id: String,
    pub title: String,
    pub summary: Option<String>,
    pub raw_input: String,
    pub status: String,
    pub parent_goal_id: Option<String>,
    pub fork_context: Option<String>,
    pub current_milestone_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGoalInput {
    pub raw_input: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGoalInput {
    pub title: Option<String>,
    pub summary: Option<String>,
    pub raw_input: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalSummary {
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub constraints: Vec<String>,
    pub refinement_questions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalTree {
    pub goal: Goal,
    pub children: Vec<GoalTree>,
}

// ==================== Milestone Types ====================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MilestoneStatus {
    Planned,
    InProgress,
    Completed,
    Failed,
    Pruned,
}

impl std::fmt::Display for MilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MilestoneStatus::Planned => write!(f, "planned"),
            MilestoneStatus::InProgress => write!(f, "in_progress"),
            MilestoneStatus::Completed => write!(f, "completed"),
            MilestoneStatus::Failed => write!(f, "failed"),
            MilestoneStatus::Pruned => write!(f, "pruned"),
        }
    }
}

impl TryFrom<&str> for MilestoneStatus {
    type Error = String;
    fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
        match s {
            "planned" => Ok(MilestoneStatus::Planned),
            "in_progress" => Ok(MilestoneStatus::InProgress),
            "completed" => Ok(MilestoneStatus::Completed),
            "failed" => Ok(MilestoneStatus::Failed),
            "pruned" => Ok(MilestoneStatus::Pruned),
            _ => Err(format!("Unknown milestone status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Milestone {
    pub id: String,
    pub goal_id: String,
    pub milestone_order: i32,
    pub parent_milestone_id: Option<String>,
    pub plan: Option<String>,
    pub plan_summary: Option<String>,
    pub skill_path: Option<String>,
    pub status: String,
    pub result_summary: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub steps: Vec<PlanStep>,
    pub dependencies: Vec<(usize, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub id: String,
    pub description: String,
    pub skill_requirements: Vec<SkillRequirement>,
    pub status: PlanStepStatus,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PlanStepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

// ==================== Skill Types (产物) ====================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SkillStatus {
    Requested,
    Generated,
    Verified,
    Failed,
}

impl std::fmt::Display for SkillStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillStatus::Requested => write!(f, "requested"),
            SkillStatus::Generated => write!(f, "generated"),
            SkillStatus::Verified => write!(f, "verified"),
            SkillStatus::Failed => write!(f, "failed"),
        }
    }
}

impl TryFrom<&str> for SkillStatus {
    type Error = String;
    fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
        match s {
            "requested" => Ok(SkillStatus::Requested),
            "generated" => Ok(SkillStatus::Generated),
            "verified" => Ok(SkillStatus::Verified),
            "failed" => Ok(SkillStatus::Failed),
            _ => Err(format!("Unknown skill status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Skill {
    pub id: String,
    pub milestone_id: String,
    pub name: String,
    pub description: String,
    pub harness_skill_id: Option<String>,
    pub parameters_schema: Option<String>,
    pub test_report: Option<String>,
    pub status: String,
    pub source_code_ref: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// ==================== AgentLog Types ====================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgentEventKind {
    Started,
    Progress,
    Decision,
    Error,
    Completed,
}

impl std::fmt::Display for AgentEventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentEventKind::Started => write!(f, "started"),
            AgentEventKind::Progress => write!(f, "progress"),
            AgentEventKind::Decision => write!(f, "decision"),
            AgentEventKind::Error => write!(f, "error"),
            AgentEventKind::Completed => write!(f, "completed"),
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
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkNode {
    pub id: String,
    pub workspace_id: String,
    pub parent_node_id: Option<String>,
    pub node_order: i32,
    pub status: String,
    pub bug_md: Option<String>,
    pub user_manual_md: Option<String>,
    pub conclusion_md: Option<String>,
    pub milestone_id: Option<String>,
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
