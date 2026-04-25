//! Agent IPC commands
//!
//! Commands for controlling agent lifecycle (start, stop, status)
//! Agent 代理生命周期控制命令

use serde::Serialize;
use tauri::State;

use crate::agents::AgentStatus;
use crate::workhub::AgentType;
use crate::error::AppResult;
use crate::state::AppState;

/// Agent status response
#[derive(Debug, Clone, Serialize)]
pub struct AgentStatusResponse {
    pub goal_id: String,
    pub agent_type: Option<String>,
    pub status: Option<String>,
}

/// Start an agent for a goal
/// 
/// The agent type is determined by the goal's current status:
/// - "draft" -> Summarizer (to create summary)
/// - "pinned" -> Builder (to build initial implementation)
/// - "building" -> Builder (to resume building)
/// - "reached" -> Optimizer (to optimize and archive)
/// - "optimizing" -> Optimizer (to resume optimization)
/// - "failed" -> Builder (to retry building)
#[tauri::command]
pub async fn agents_start(
    state: State<'_, AppState>,
    goal_id: String,
) -> AppResult<AgentStatusResponse> {
    let status = state.supervisor.start_agent(&goal_id).await?;
    
    Ok(AgentStatusResponse {
        goal_id,
        agent_type: None, // Could be added to AgentStatus
        status: Some(format!("{:?}", status)),
    })
}

/// Stop a running agent for a goal
#[tauri::command]
pub async fn agents_stop(
    state: State<'_, AppState>,
    goal_id: String,
) -> AppResult<AgentStatusResponse> {
    let status = state.supervisor.stop_agent(&goal_id).await?;
    
    Ok(AgentStatusResponse {
        goal_id,
        agent_type: None,
        status: Some(format!("{:?}", status)),
    })
}

/// Get the current status of an agent for a goal
#[tauri::command]
pub async fn agents_status(
    state: State<'_, AppState>,
    goal_id: String,
) -> AppResult<AgentStatusResponse> {
    let status = state.supervisor.get_status(&goal_id).await;
    
    Ok(AgentStatusResponse {
        goal_id,
        agent_type: None,
        status: status.map(|s| format!("{:?}", s)),
    })
}

/// List all running agents
#[tauri::command]
pub async fn agents_list_running(
    state: State<'_, AppState>,
) -> AppResult<Vec<(String, String, String)>> {
    let running = state.supervisor.list_running().await;
    Ok(running
        .into_iter()
        .map(|(goal_id, agent_type, status)| {
            (goal_id, format!("{:?}", agent_type), format!("{:?}", status))
        })
        .collect())
}
