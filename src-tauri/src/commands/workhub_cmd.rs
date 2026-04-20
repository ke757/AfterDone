//! WorkSpace and WorkNode IPC commands

use tauri::State;
use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::state::AppState;
use crate::workhub::{
    WorkHub, WorkSpace, WorkNode, BugEntry,
    CreateWorkSpaceInput, CreateWorkNodeInput,
};

/// Initialize a WorkSpace for a goal
#[tauri::command]
pub async fn workspace_init(
    state: State<'_, AppState>,
    goal_id: String,
) -> AppResult<WorkSpace> {
    WorkHub::init_workspace(&state.db, &goal_id).await
}

/// Get WorkSpace by goal ID
#[tauri::command]
pub async fn workspace_get(
    state: State<'_, AppState>,
    goal_id: String,
) -> AppResult<Option<WorkSpace>> {
    WorkHub::get_workspace_by_goal(&state.db, &goal_id).await
}

/// Get or create WorkSpace for a goal
#[tauri::command]
pub async fn workspace_get_or_create(
    state: State<'_, AppState>,
    goal_id: String,
) -> AppResult<WorkSpace> {
    WorkHub::get_or_create_workspace(&state.db, &goal_id).await
}

/// List all WorkSpaces
#[tauri::command]
pub async fn workspace_list(
    state: State<'_, AppState>,
) -> AppResult<Vec<WorkSpace>> {
    WorkHub::list_workspaces(&state.db).await
}

/// Update PLAN.md
#[tauri::command]
pub async fn workspace_update_plan(
    state: State<'_, AppState>,
    workspace_id: String,
    content: String,
) -> AppResult<()> {
    WorkHub::store_plan(&state.db, &workspace_id, &content).await
}

/// Get PLAN.md
#[tauri::command]
pub async fn workspace_get_plan(
    state: State<'_, AppState>,
    workspace_id: String,
) -> AppResult<Option<String>> {
    WorkHub::get_plan(&state.db, &workspace_id).await
}

/// Create initial worknode
#[tauri::command]
pub async fn worknode_create_initial(
    state: State<'_, AppState>,
    workspace_id: String,
) -> AppResult<WorkNode> {
    WorkHub::create_initial_worknode(&state.db, &workspace_id).await
}

/// Get current worknode
#[tauri::command]
pub async fn worknode_get_current(
    state: State<'_, AppState>,
    workspace_id: String,
) -> AppResult<Option<WorkNode>> {
    WorkHub::get_current_node(&state.db, &workspace_id).await
}

/// Get worknode by ID
#[tauri::command]
pub async fn worknode_get(
    state: State<'_, AppState>,
    node_id: String,
) -> AppResult<WorkNode> {
    WorkHub::get_worknode(&state.db, &node_id).await
}

/// List all worknodes for a workspace
#[tauri::command]
pub async fn worknode_list(
    state: State<'_, AppState>,
    workspace_id: String,
) -> AppResult<Vec<WorkNode>> {
    WorkHub::list_worknodes(&state.db, &workspace_id).await
}

/// Get bugs from worknode
#[tauri::command]
pub async fn worknode_get_bugs(
    state: State<'_, AppState>,
    node_id: String,
) -> AppResult<Vec<BugEntry>> {
    WorkHub::get_node_bug(&state.db, &node_id).await
}

/// Add bug to worknode
#[tauri::command]
pub async fn worknode_add_bug(
    state: State<'_, AppState>,
    node_id: String,
    description: String,
    error_output: Option<String>,
) -> AppResult<()> {
    let bug = BugEntry::new(description, error_output);
    WorkHub::add_node_bug(&state.db, &node_id, bug).await
}

/// Get conclusion from worknode
#[tauri::command]
pub async fn worknode_get_conclusion(
    state: State<'_,
 AppState>,
    node_id: String,
) -> AppResult<Option<String>> {
    WorkHub::get_node_conclusion(&state.db, &node_id).await
}

/// Get user manual from worknode
#[tauri::command]
pub async fn worknode_get_user_manual(
    state: State<'_, AppState>,
    node_id: String,
) -> AppResult<Option<String>> {
    WorkHub::get_node_user_manual(&state.db, &node_id).await
}

/// Mark goal as achieved
#[tauri::command]
pub async fn worknode_goal_achieved(
    state: State<'_, AppState>,
    workspace_id: String,
    conclusion: String,
) -> AppResult<String> {
    WorkHub::goal_achieved(&state.db, &workspace_id, &conclusion).await
}

/// Mark node as achieved
#[tauri::command]
pub async fn worknode_achieved(
    state: State<'_, AppState>,
    parent_node_id: String,
    conclusion: String,
) -> AppResult<String> {
    WorkHub::node_achieved(&state.db, &parent_node_id, &conclusion).await
}
