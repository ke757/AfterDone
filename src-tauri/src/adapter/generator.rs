//! Generator Communication Protocol
//!
//! Defines the JSON-RPC methods and types for communicating with
//! HarnessAgent (Generator) via the Transport trait.
//!
//! Protocol Methods:
//! - generator.execute: Execute a generation task with a specification
//! - generator.status: Query task status
//! - generator.cancel: Cancel a running task
//! - generator.list: List available skills

use serde::{Deserialize, Serialize};

use crate::workhub::Specification;

/// Generator method names
pub const METHOD_EXECUTE: &str = "generator.execute";
pub const METHOD_STATUS: &str = "generator.status";
pub const METHOD_CANCEL: &str = "generator.cancel";
pub const METHOD_LIST_SKILLS: &str = "generator.listSkills";

/// Execute request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteParams {
    /// Task ID (for tracking)
    pub task_id: String,
    /// WorkSpace ID
    pub workspace_id: String,
    /// WorkNode ID (optional)
    pub worknode_id: Option<String>,
    /// The specification to execute
    pub specification: Specification,
    /// Additional context
    #[serde(default)]
    pub context: serde_json::Value,
}

/// Execute response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteResponse {
    /// Task ID
    pub task_id: String,
    /// Whether the task was accepted
    pub accepted: bool,
    /// Message from generator
    pub message: Option<String>,
}

/// Status request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusParams {
    /// Task ID to query
    pub task_id: String,
}

/// Status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    /// Task ID
    pub task_id: String,
    /// Current status
    pub status: GeneratorTaskStatus,
    /// Progress percentage (0-100)
    pub progress: u8,
    /// Current step description
    pub current_step: Option<String>,
    /// Result (if completed)
    pub result: Option<TaskResult>,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Generator task status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GeneratorTaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for GeneratorTaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GeneratorTaskStatus::Pending => write!(f, "pending"),
            GeneratorTaskStatus::Running => write!(f, "running"),
            GeneratorTaskStatus::Completed => write!(f, "completed"),
            GeneratorTaskStatus::Failed => write!(f, "failed"),
            GeneratorTaskStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Task result from generator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Summary of what was generated
    pub summary: String,
    /// User manual content (USER_MANUAL.md)
    pub user_manual: Option<String>,
    /// Skills that were created/used
    pub skills: Vec<GeneratedSkill>,
    /// Files that were modified
    #[serde(default)]
    pub files_modified: Vec<String>,
}

/// Generated skill info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedSkill {
    /// Skill name
    pub name: String,
    /// Skill description
    pub description: String,
    /// Skill path (if saved)
    pub path: Option<String>,
}

/// Cancel request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelParams {
    /// Task ID to cancel
    pub task_id: String,
}

/// Cancel response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelResponse {
    /// Whether the task was cancelled
    pub cancelled: bool,
    /// Message
    pub message: Option<String>,
}

/// List skills response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSkillsResponse {
    /// Available skills
    pub skills: Vec<SkillInfo>,
}

/// Skill info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    /// Skill name
    pub name: String,
    /// Skill description
    pub description: String,
    /// Parameters schema
    pub parameters_schema: Option<serde_json::Value>,
}

impl ExecuteParams {
    /// Create new execute params
    pub fn new(
        task_id: String,
        workspace_id: String,
        worknode_id: Option<String>,
        specification: Specification,
    ) -> Self {
        Self {
            task_id,
            workspace_id,
            worknode_id,
            specification,
            context: serde_json::Value::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_params_serialization() {
        let spec = Specification {
            project_background: "Test project".to_string(),
            module_breakdown: vec![],
            acceptance_criteria: vec!["Works".to_string()],
        };
        let params = ExecuteParams::new(
            "task-1".to_string(),
            "ns-1".to_string(),
            None,
            spec,
        );
        
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("task-1"));
        assert!(json.contains("ns-1"));
    }
}

