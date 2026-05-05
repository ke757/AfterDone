//! Summarizer IPC commands
//!
//! 对话式 Summarizer 的 Tauri 命令。
//! 前端通过这两个命令驱动 Summarizer 的多轮探需对话流程。

use serde::Serialize;
use tauri::State;

use crate::workhub::Goal;
use crate::error::AppResult;
use crate::memory::{StoredResult, ResultStatus};
use crate::state::AppState;
use crate::services::SummarizerService;

/// 投递一条用户消息给 Summarizer Agent 并获取响应
///
/// 每次调用处理一轮对话。后端加载 session 历史，调用 LLM，返回流式响应。
/// 如果 LLM 已生成完整的 GoalSummary JSON，响应中会包含 summary_draft。
///
/// 前端流程：
///   1. 用户输入消息，调用此命令
///   2. 展示流式响应（通过 ryg:agent:stream:{workspace_id} 事件）
///   3. 检查返回的 summary_ready：
///      - true → 显示"确认"按钮
///      - false → 继续对话
#[tauri::command]
pub async fn summarizer_send_message(
    state: State<'_, AppState>,
    workspace_id: String,
    message: String,
) -> AppResult<SummarizerTurnResponse> {
    let stored = state.supervisor.deliver_message(&workspace_id, &message).await?;

    let summary_ready = stored.status == ResultStatus::Ready;

    // 从 StoredResult.output 提取消息文本
    let message = extract_message_text(&stored);

    Ok(SummarizerTurnResponse {
        workspace_id,
        message,
        summary_ready,
    })
}

/// 确认 Summarizer 生成的 GoalSummary，持久化到数据库
///
/// 从 session 历史中提取最后一条 assistant 消息，解析为 GoalSummary JSON，
/// 写入 goals 表的 summary 字段，并将状态设为 Draft（等待用户 Pin）。
///
/// 直接调用 SummarizerService::confirm()，不经过 Supervisor/Runtime。
#[tauri::command]
pub async fn summarizer_confirm(
    state: State<'_, AppState>,
    workspace_id: String,
) -> AppResult<Goal> {
    SummarizerService::confirm(
        &state.db,
        &workspace_id,
        &state.sessions,
        &state.emitter,
    ).await
}

/// 查询 workspace 的当前对话状态
#[tauri::command]
pub async fn summarizer_status(
    state: State<'_, AppState>,
    workspace_id: String,
) -> AppResult<SummarizerStatusResponse> {
    let status = state.supervisor.get_conversation_status(&workspace_id).await;

    Ok(SummarizerStatusResponse {
        workspace_id,
        status: status.map(|s| format!("{:?}", s)),
    })
}

/// 从 StoredResult 中提取消息文本
fn extract_message_text(stored: &StoredResult) -> String {
    // 尝试从 output JSON 中提取 message 字段
    stored.output
        .get("message")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| format!("{:?}", stored.output))
}

/// 一轮 Summarizer 对话的响应
#[derive(Debug, Clone, Serialize)]
pub struct SummarizerTurnResponse {
    pub workspace_id: String,
    pub message: String,
    pub summary_ready: bool,
}

/// Summarizer 对话状态响应
#[derive(Debug, Clone, Serialize)]
pub struct SummarizerStatusResponse {
    pub workspace_id: String,
    pub status: Option<String>,
}
