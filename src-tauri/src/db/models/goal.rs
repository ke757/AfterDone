use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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
