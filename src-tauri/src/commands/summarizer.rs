//! Summarizer IPC commands
//!
//! 对话式 Summarizer 的 Tauri 命令。
//! 前端通过 cell_id 驱动 Summarizer 的多轮探需对话流程。

use serde::Serialize;
use tauri::State;
use std::sync::Arc;

use crate::workhub::Goal;
use crate::error::AppResult;
use crate::session::{EffectType, SessionLine};
use crate::session::cell::SessionCell;
use crate::state::AppState;
use crate::services::SummarizerService;

/// 投递一条用户消息给 Summarizer Cell 并获取响应
#[tauri::command]
pub async fn summarizer_send_message(
    state: State<'_, AppState>,
    cell_id: String,
    message: String,
) -> AppResult<SummarizerTurnResponse> {
    state.supervisor.deliver_message(&cell_id, &message).await?;

    let cell = state.cell_manager.get_cell(&cell_id).await
        .ok_or_else(|| crate::error::AppError::NotFound(format!("Cell not found: {}", cell_id)))?;
    let cell: Arc<dyn SessionCell> = cell;

    let response_message = cell
        .latest_effect(EffectType::Result)
        .and_then(|l| match l {
            SessionLine::Effect { content, .. } => {
                content.get("message").and_then(|v| v.as_str()).map(String::from)
            }
            _ => None,
        })
        .unwrap_or_default();

    // Check if summary is ready via latest effect content
    let summary_ready = cell
        .latest_effect(EffectType::Result)
        .and_then(|l| match l {
            SessionLine::Effect { content, .. } => {
                content.get("summary_draft").map(|v| !v.is_null())
            }
            _ => None,
        })
        .unwrap_or(false);

    Ok(SummarizerTurnResponse {
        cell_id,
        message: response_message,
        summary_ready,
    })
}

/// 确认 Summarizer 生成的 GoalSummary，持久化到数据库
#[tauri::command]
pub async fn summarizer_confirm(
    state: State<'_, AppState>,
    cell_id: String,
) -> AppResult<Goal> {
    let cell = state.cell_manager.get_cell(&cell_id).await
        .ok_or_else(|| crate::error::AppError::NotFound(format!("Cell not found: {}", cell_id)))?;
    let cell: Arc<dyn SessionCell> = cell;

    let workspace_id = cell.workspace_id().to_string();
    let lines = cell.get_lines();

    let goal = SummarizerService::confirm(
        &state.db,
        &workspace_id,
        &lines,
        &state.emitter,
    ).await?;

    state.cell_manager.close_cell(&cell_id).await?;

    Ok(goal)
}

/// 查询 Cell 的当前结果状态
#[tauri::command]
pub async fn summarizer_status(
    state: State<'_, AppState>,
    cell_id: String,
) -> AppResult<CellStatusResponse> {
    let cell = state.cell_manager.get_cell(&cell_id).await
        .ok_or_else(|| crate::error::AppError::NotFound(format!("Cell not found: {}", cell_id)))?;
    let status = cell.latest_effect(EffectType::Result).map(|l| l.created_at_str().to_string());

    Ok(CellStatusResponse {
        cell_id,
        result_status: status,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct SummarizerTurnResponse {
    pub cell_id: String,
    pub message: String,
    pub summary_ready: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CellStatusResponse {
    pub cell_id: String,
    pub result_status: Option<String>,
}
