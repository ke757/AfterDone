use tauri::State;

use crate::workhub::{Milestone, MilestonesRepo};
use crate::error::AppResult;
use crate::state::AppState;

#[tauri::command]
pub async fn milestones_list(
    state: State<'_, AppState>,
    goal_id: String,
) -> AppResult<Vec<Milestone>> {
    MilestonesRepo::list_by_goal(&state.db, &goal_id).await
}

#[tauri::command]
pub async fn milestones_get(
    state: State<'_, AppState>,
    milestone_id: String,
) -> AppResult<Milestone> {
    MilestonesRepo::get_by_id(&state.db, &milestone_id).await
}

#[tauri::command]
pub async fn milestones_fork(
    state: State<'_, AppState>,
    milestone_id: String,
    new_plan: String,
) -> AppResult<Milestone> {
    let parent = MilestonesRepo::get_by_id(&state.db, &milestone_id).await?;
    let new_order = parent.milestone_order + 1;

    MilestonesRepo::create(
        &state.db,
        &parent.goal_id,
        new_order,
        Some(&milestone_id),
        Some(&new_plan),
        None,
    )
    .await
}
