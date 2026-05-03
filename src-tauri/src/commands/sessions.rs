use serde::Serialize;
use tauri::State;

use crate::session::Message;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct SessionHistoryResponse {
    pub node_id: String,
    pub messages: Vec<Message>,
}

/// 获取指定 node 的会话历史（从 JSONL 加载）
#[tauri::command]
pub async fn sessions_get_node(
    node_id: String,
    state: State<'_, AppState>,
) -> Result<SessionHistoryResponse, String> {
    let session = state
        .supervisor
        .sessions()
        .get_or_create_node(&node_id)
        .map_err(|e| e.to_string())?;

    let messages = session.get_history();

    Ok(SessionHistoryResponse {
        node_id,
        messages,
    })
}
