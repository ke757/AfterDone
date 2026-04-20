use sqlx::SqlitePool;
use serde_json;

use crate::error::{AppError, AppResult};
use super::super::types::*;

/// Repository for WorkSpace operations
pub struct WorkSpaceRepo;

impl WorkSpaceRepo {
    /// Create a new WorkSpace for a goal
    pub async fn create(db: &SqlitePool, input: CreateWorkSpaceInput) -> AppResult<WorkSpace> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO workspaces (id, goal_id, goal_md, plan_md, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&input.goal_id)
        .bind(&input.goal_md)
        .bind(&input.plan_md)
        .bind(&now)
        .bind(&now)
        .execute(db)
        .await?;

        Self::get_by_id(db, &id).await
    }

    /// Get WorkSpace by id
    pub async fn get_by_id(db: &SqlitePool, id: &str) -> AppResult<WorkSpace> {
        sqlx::query_as::<_, WorkSpace>(
            "SELECT id, goal_id, current_node_id, goal_md, plan_md, created_at, updated_at
             FROM workspaces WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("WorkSpace not found: {}", id)))
    }

    /// Get WorkSpace by goal_id
    pub async fn get_by_goal_id(db: &SqlitePool, goal_id: &str) -> AppResult<Option<WorkSpace>> {
        sqlx::query_as::<_, WorkSpace>(
            "SELECT id, goal_id, current_node_id, goal_md, plan_md, created_at, updated_at
             FROM workspaces WHERE goal_id = ?"
        )
        .bind(goal_id)
        .fetch_optional(db)
        .await
        .map_err(AppError::Database)
    }

    /// Update GOAL.md
    pub async fn update_goal_md(db: &SqlitePool, id: &str, content: &str) -> AppResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE workspaces SET goal_md = ?, updated_at = ? WHERE id = ?")
            .bind(content)
            .bind(&now)
            .bind(id)
            .execute(db)
            .await?;
        Ok(())
    }

    /// Update PLAN.md
    pub async fn update_plan_md(db: &SqlitePool, id: &str, content: &str) -> AppResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE workspaces SET plan_md = ?, updated_at = ? WHERE id = ?")
            .bind(content)
            .bind(&now)
            .bind(id)
            .execute(db)
            .await?;
        Ok(())
    }

    /// Update current node
    pub async fn update_current_node(db: &SqlitePool, id: &str, node_id: Option<&str>) -> AppResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE workspaces SET current_node_id = ?, updated_at = ? WHERE id = ?")
            .bind(node_id)
            .bind(&now)
            .bind(id)
            .execute(db)
            .await?;
        Ok(())
    }

    /// List all WorkSpaces
    pub async fn list_all(db: &SqlitePool) -> AppResult<Vec<WorkSpace>> {
        sqlx::query_as::<_, WorkSpace>(
            "SELECT id, goal_id, current_node_id, goal_md, plan_md, created_at, updated_at
             FROM workspaces ORDER BY created_at DESC"
        )
        .fetch_all(db)
        .await
        .map_err(AppError::Database)
    }

    /// Delete WorkSpace
    pub async fn delete(db: &SqlitePool, id: &str) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM workspaces WHERE id = ?")
            .bind(id)
            .execute(db)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("WorkSpace not found: {}", id)));
        }

        Ok(())
    }
}
