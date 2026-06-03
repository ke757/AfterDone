use std::sync::Arc;
use tauri::{AppHandle, Emitter};

use crate::events::types::*;
use crate::llm::events::StreamEvent;

/// Typed event bridge wrapping Tauri's AppHandle.
/// This is the ONLY component that knows about Tauri's event system.
/// All other modules emit events through this abstraction.
#[derive(Clone)]
pub struct EventBridge {
    app: Arc<AppHandle>,
}

impl EventBridge {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app: Arc::new(app),
        }
    }

    pub fn emit_agent_stream(&self, workspace_id: &str, delta: &str, finished: bool) {
        let event = format!("ryg:agent:stream:{}", workspace_id);
        let _ = self.app.emit(&event, AgentStreamPayload {
            delta: delta.to_string(),
            finished,
        });
    }

    /// Send a structured StreamEvent to the frontend.
    pub fn emit_stream_event(&self, workspace_id: &str, evt: &StreamEvent) {
        let event = format!("ryg:agent:stream:{}", workspace_id);
        let _ = self.app.emit(&event, evt);
    }

    pub fn emit_agent_status(&self, workspace_id: &str, agent_type: &str, status: &str, phase: &str) {
        let event = format!("ryg:agent:status:{}", workspace_id);
        let _ = self.app.emit(&event, AgentStatusPayload {
            agent_type: agent_type.to_string(),
            status: status.to_string(),
            phase: phase.to_string(),
        });
    }

    pub fn emit_agent_decision(
        &self,
        workspace_id: &str,
        decision: &str,
        reasoning: &str,
        data: Option<serde_json::Value>,
    ) {
        let event = format!("ryg:agent:decision:{}", workspace_id);
        let _ = self.app.emit(&event, AgentDecisionPayload {
            decision: decision.to_string(),
            reasoning: reasoning.to_string(),
            data,
        });
    }

    pub fn emit_goal_status(&self, goal_id: &str, status: &str, previous: &str) {
        let event = format!("ryg:goal:status:{}", goal_id);
        let _ = self.app.emit(&event, GoalStatusPayload {
            status: status.to_string(),
            previous: previous.to_string(),
        });
    }

    pub fn emit_openclaw_connection(&self, connected: bool, url: &str) {
        let _ = self.app.emit("ryg:openclaw:connection", OpenClawConnectionPayload {
            connected,
            url: url.to_string(),
        });
    }
}
