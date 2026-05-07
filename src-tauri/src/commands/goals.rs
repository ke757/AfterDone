use tauri::State;

use crate::workhub::{CreateGoalInput, Goal, UpdateGoalInput, GoalsRepo, GoalStatus};
use crate::error::AppResult;
use crate::state::AppState;

#[tauri::command]
pub async fn goals_create(
    state: State<'_, AppState>,
    content: String,
) -> AppResult<Goal> {
    // Use the first line as title, rest as content
    let title = content
        .lines()
        .next()
        .unwrap_or("Untitled Goal")
        .to_string();

    let input = CreateGoalInput { content, title };
    let goal = GoalsRepo::create(&state.db, input).await?;

    // Auto-trigger Summarizer agent (will be wired in lib.rs)
    Ok(goal)
}

#[tauri::command]
pub async fn goals_list(
    state: State<'_, AppState>,
    status: Option<String>,
) -> AppResult<Vec<Goal>> {
    GoalsRepo::list(&state.db, status.as_deref()).await
}

#[tauri::command]
pub async fn goals_get(
    state: State<'_, AppState>,
    goal_id: String,
) -> AppResult<Goal> {
    GoalsRepo::get_by_id(&state.db, &goal_id).await
}

#[tauri::command]
pub async fn goals_pin(
    state: State<'_, AppState>,
    goal_id: String,
) -> AppResult<Goal> {
    let previous = GoalsRepo::get_by_id(&state.db, &goal_id).await?;
    let goal = GoalsRepo::update_status(&state.db, &goal_id, GoalStatus::Pinned).await?;

    // TODO: Trigger Executor agent via supervisor

    Ok(goal)
}

#[tauri::command]
pub async fn goals_update(
    state: State<'_, AppState>,
    goal_id: String,
    title: Option<String>,
    summary: Option<String>,
    content: Option<String>,
) -> AppResult<Goal> {
    let input = UpdateGoalInput {
        title,
        summary,
        content,
    };
    GoalsRepo::update(&state.db, &goal_id, input).await
}

#[tauri::command]
pub async fn goals_delete(
    state: State<'_, AppState>,
    goal_id: String,
) -> AppResult<()> {
    GoalsRepo::delete(&state.db, &goal_id).await
}

// #[tauri::command]
// pub async fn goals_tree(
//     state: State<'_, AppState>,
//     goal_id: String,
// ) -> AppResult<GoalTree> {
//     let goal = GoalsRepo::get_by_id(&state.db, &goal_id).await?;
//     let children = GoalsRepo::get_children(&state.db, &goal_id).await?;

//     // Simple single-level tree (recursive tree building can be added later)
//     let child_trees = children
//         .into_iter()
//         .map(|c| GoalTree {
//             goal: c,
//             children: vec![],
//         })
//         .collect();

//     Ok(GoalTree {
//         goal,
//         children: child_trees,
//     })
// }
