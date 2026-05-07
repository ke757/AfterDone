use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStreamPayload {
    pub delta: String,
    pub finished: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatusPayload {
    pub agent_type: String,
    pub status: String,
    pub phase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDecisionPayload {
    pub decision: String,
    pub reasoning: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalStatusPayload {
    pub status: String,
    pub previous: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenClawConnectionPayload {
    pub connected: bool,
    pub url: String,
}
