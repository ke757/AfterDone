use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    AgentSummarizer,
    AgentExecutor,
    AgentOptimizer,
    System,
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageRole::User => write!(f, "user"),
            MessageRole::AgentSummarizer => write!(f, "agent_summarizer"),
            MessageRole::AgentExecutor => write!(f, "agent_executor"),
            MessageRole::AgentOptimizer => write!(f, "agent_optimizer"),
            MessageRole::System => write!(f, "system"),
        }
    }
}

impl TryFrom<&str> for MessageRole {
    type Error = String;
    fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
        match s {
            "user" => Ok(MessageRole::User),
            "agent_summarizer" => Ok(MessageRole::AgentSummarizer),
            "agent_executor" => Ok(MessageRole::AgentExecutor),
            "agent_optimizer" => Ok(MessageRole::AgentOptimizer),
            "system" => Ok(MessageRole::System),
            _ => Err(format!("Unknown message role: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: String,
    pub goal_id: String,
    pub role: String,
    pub content: String,
    pub metadata: Option<String>,
    pub created_at: String,
}
