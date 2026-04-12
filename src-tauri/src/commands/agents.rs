use serde::Serialize;
use tauri::State;

use crate::agents::AgentStatus;
use crate::error::AppResult;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct AgentStatusResponse {
    pub goal_id: String,
    pub status: Option<String>,
}

#[tauri::command]
pub async fn agents_start(
    state: State<'_, AppState>,
    goal_id: String,
) -> AppResult<AgentStatusResponse> {
    // TODO: Wire to AgentSupervisor
    Ok(AgentStatusResponse {
        goal_id,
        status: Some("not_implemented".to_string()),
    })
}

#[tauri::command]
pub async fn agents_stop(
    state: State<'_, AppState>,
    goal_id: String,
) -> AppResult<AgentStatusResponse> {
    Ok(AgentStatusResponse {
        goal_id,
        status: Some("not_implemented".to_string()),
    })
}

#[tauri::command]
pub async fn agents_status(
    state: State<'_, AppState>,
    goal_id: String,
) -> AppResult<AgentStatusResponse> {
    Ok(AgentStatusResponse {
        goal_id,
        status: None,
    })
}
