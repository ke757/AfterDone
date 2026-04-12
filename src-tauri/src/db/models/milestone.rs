use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRequirement {
    pub name: String,
    pub description: String,
    pub parameters_schema: Option<serde_json::Value>,
    pub priority: u32,
}
