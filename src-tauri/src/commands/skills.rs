use tauri::State;

use crate::db::models::Skill;
use crate::db::repos::SkillsRepo;
use crate::error::AppResult;
use crate::state::AppState;

#[tauri::command]
pub async fn skills_list(
    state: State<'_, AppState>,
    milestone_id: String,
) -> AppResult<Vec<Skill>> {
    SkillsRepo::list_by_milestone(&state.db, &milestone_id).await
}

#[tauri::command]
pub async fn skills_get(
    state: State<'_, AppState>,
    skill_id: String,
) -> AppResult<Skill> {
    SkillsRepo::get_by_id(&state.db, &skill_id).await
}

#[tauri::command]
pub async fn skills_invoke(
    state: State<'_, AppState>,
    skill_id: String,
    params: serde_json::Value,
) -> AppResult<serde_json::Value> {
    // TODO: Implement skill invocation via Transport
    let _ = params;
    let skill = SkillsRepo::get_by_id(&state.db, &skill_id).await?;

    Ok(serde_json::json!({
        "skill": skill.name,
        "status": "invoked",
        "message": "Skill invocation not yet implemented"
    }))
}
