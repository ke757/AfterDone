use serde::Serialize;
use tauri::State;
use std::sync::Arc;

use crate::session::Message;
use crate::session::cell::{CellInfo, CellType, SessionCell};
use crate::state::AppState;
use crate::error::AppResult;

// ============================================================================
// Cell Commands
// ============================================================================

/// 创建新的 Cell
#[tauri::command]
pub async fn cell_create(
    state: State<'_, AppState>,
    workspace_id: String,
    node_id: String,
    cell_type: String,
) -> AppResult<CellInfo> {
    let ct: CellType = cell_type.as_str().try_into()
        .map_err(|e: String| crate::error::AppError::Agent(e))?;
    let cell: Arc<dyn SessionCell> = state.cell_manager.create_cell(&workspace_id, &node_id, ct).await?;
    Ok(cell.to_info())
}

/// 列出某 WorkNode 下所有 cells
#[tauri::command]
pub async fn cell_list_by_node(
    state: State<'_, AppState>,
    node_id: String,
) -> AppResult<Vec<CellInfo>> {
    Ok(state.cell_manager.list_cells_by_node(&node_id).await)
}

/// 获取 Cell 的完整会话历史
#[tauri::command]
pub async fn cell_get_history(
    state: State<'_, AppState>,
    cell_id: String,
) -> AppResult<CellHistoryResponse> {
    let cell = state.cell_manager.get_cell(&cell_id).await
        .ok_or_else(|| crate::error::AppError::NotFound(format!("Cell not found: {}", cell_id)))?;
    let cell: Arc<dyn SessionCell> = cell;

    Ok(CellHistoryResponse {
        cell_id,
        messages: cell.get_history(),
    })
}

// ============================================================================
// Response types
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct CellHistoryResponse {
    pub cell_id: String,
    pub messages: Vec<Message>,
}
