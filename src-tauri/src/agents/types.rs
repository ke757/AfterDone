use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::comm::Transport;
use crate::db::models::{AgentType, Goal, Message, Milestone, Skill, AgentLog};
use crate::events::EventBridge;
use crate::llm::LlmProvider;
use std::sync::Arc;

/// Context passed to any agent on each run.
/// Constructed by AgentSupervisor from the database before spawning.
#[derive(Debug, Clone)]
pub struct GoalContext {
    pub goal: Goal,
    pub current_milestone: Option<Milestone>,
    pub conversation_history: Vec<Message>,
    pub available_skills: Vec<Skill>,
    pub agent_logs: Vec<AgentLog>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Output from an agent run
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentOutput {
    /// Output from the Summarizer agent
    GoalSummary {
        title: String,
        description: String,
        acceptance_criteria: Vec<String>,
        constraints: Vec<String>,
        refinement_questions: Vec<String>,
    },
    /// Output from the Executor agent
    ExecutionResult {
        milestone_id: String,
        plan: Option<serde_json::Value>,
        skills_used: Vec<String>,
        success: bool,
        failure_reason: Option<String>,
    },
    /// Output from the Optimizer agent
    OptimizationResult {
        milestone_id: String,
        optimizations: Vec<Optimization>,
        solidified: bool,
    },
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
    Starting,
    Running,
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
