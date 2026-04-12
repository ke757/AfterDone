//! NodeSpace and WorkNode IPC commands

use tauri::State;
use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::state::AppState;
use crate::noderepo::{
    NodeRepo, NodeSpace, WorkNode, BugEntry,
    CreateNodeSpaceInput, CreateWorkNodeInput,
};

/// Initialize a NodeSpace for a goal
#[tauri::command]
pub async fn nodespace_init(
    state: State<'_, AppState>,
    goal_id: String,
) -> AppResult<NodeSpace> {
    NodeRepo::init_nodespace(&state.db, &goal_id).await
}

/// Get NodeSpace by goal ID
#[tauri::command]
pub async fn nodespace_get(
    state: State<'_, AppState>,
    goal_id: String,
) -> AppResult<Option<NodeSpace>> {
    NodeRepo::get_nodespace_by_goal(&state.db, &goal_id).await
}

/// Get or create NodeSpace for a goal
#[tauri::command]
pub async fn nodespace_get_or_create(
    state: State<'_, AppState>,
    goal_id: String,
) -> AppResult<NodeSpace> {
    NodeRepo::get_or_create_nodespace(&state.db, &goal_id).await
}

/// Update PLAN.md
#[tauri::command]
pub async fn nodespace_update_plan(
    state: State<'_, AppState>,
    nodespace_id: String,
    content: String,
) -> AppResult<()> {
    NodeRepo::store_plan(&state.db, &nodespace_id, &content).await
}

/// Get PLAN.md
#[tauri::command]
pub async fn nodespace_get_plan(
    state: State<'_, AppState>,
    nodespace_id: String,
) -> AppResult<Option<String>> {
    NodeRepo::get_plan(&state.db, &nodespace_id).await
}

/// Create initial worknode
#[tauri::command]
pub async fn worknode_create_initial(
    state: State<'_, AppState>,
    nodespace_id: String,
) -> AppResult<WorkNode> {
    NodeRepo::create_initial_worknode(&state.db, &nodespace_id).await
}

/// Get current worknode
#[tauri::command]
pub async fn worknode_get_current(
    state: State<'_, AppState>,
    nodespace_id: String,
) -> AppResult<Option<WorkNode>> {
    NodeRepo::get_current_node(&state.db, &nodespace_id).await
}

/// Get worknode by ID
#[tauri::command]
pub async fn worknode_get(
    state: State<'_, AppState>,
    node_id: String,
) -> AppResult<WorkNode> {
    NodeRepo::get_worknode(&state.db, &node_id).await
}

/// List all worknodes for a nodespace
#[tauri::command]
pub async fn worknode_list(
    state: State<'_, AppState>,
    nodespace_id: String,
) -> AppResult<Vec<WorkNode>> {
    NodeRepo::list_worknodes(&state.db, &nodespace_id).await
}

/// Get bugs from worknode
#[tauri::command]
pub async fn worknode_get_bugs(
    state: State<'_, AppState>,
    node_id: String,
) -> AppResult<Vec<BugEntry>> {
    NodeRepo::get_node_bug(&state.db, &node_id).await
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
    NodeRepo::add_node_bug(&state.db, &node_id, bug).await
}

/// Get conclusion from worknode
#[tauri::command]
pub async fn worknode_get_conclusion(
    state: State<'_, AppState>,
    node_id: String,
) -> AppResult<Option<String>> {
    NodeRepo::get_node_conclusion(&state.db, &node_id).await
}

/// Get user manual from worknode
#[tauri::command]
pub async fn worknode_get_user_manual(
    state: State<'_, AppState>,
    node_id: String,
) -> AppResult<Option<String>> {
    NodeRepo::get_node_user_manual(&state.db, &node_id).await
}

/// Mark goal as achieved
#[tauri::command]
pub async fn worknode_goal_achieved(
    state: State<'_, AppState>,
    nodespace_id: String,
    conclusion: String,
) -> AppResult<String> {
    NodeRepo::goal_achieved(&state.db, &nodespace_id, &conclusion).await
}

/// Mark node as achieved
#[tauri::command]
pub async fn worknode_achieved(
    state: State<'_, AppState>,
    parent_node_id: String,
    conclusion: String,
) -> AppResult<String> {
    NodeRepo::node_achieved(&state.db, &parent_node_id, &conclusion).await
}
