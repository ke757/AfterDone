use tauri::State;

use crate::db::models::Message;
use crate::db::repos::MessagesRepo;
use crate::error::AppResult;
use crate::state::AppState;

#[tauri::command]
pub async fn chat_send(
    state: State<'_, AppState>,
    goal_id: String,
    content: String,
) -> AppResult<Message> {
    MessagesRepo::create(&state.db, &goal_id, "user", &content, None).await
}

#[tauri::command]
pub async fn chat_history(
    state: State<'_, AppState>,
    goal_id: String,
    limit: Option<u32>,
    before: Option<String>,
) -> AppResult<Vec<Message>> {
    MessagesRepo::list_by_goal(&state.db, &goal_id, limit, before.as_deref()).await
}
