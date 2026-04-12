use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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
