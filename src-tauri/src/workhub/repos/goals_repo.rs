use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use super::super::types::*;
use super::super::types::GoalStatus;

/// Repository for Goal operations
pub struct GoalsRepo;

impl GoalsRepo {
    pub async fn create(pool: &SqlitePool, input: CreateGoalInput) -> AppResult<Goal> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO goals (id, title, content, status, created_at, updated_at)
             VALUES (?, ?, ?, 'draft', ?, ?)"
        )
        .bind(&id)
        .bind(&input.title)
        .bind(&input.content)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await?;

        Self::get_by_id(pool, &id).await
    }

    pub async fn get_by_id(pool: &SqlitePool, id: &str) -> AppResult<Goal> {
        sqlx::query_as::<_, Goal>("SELECT * FROM goals WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Goal not found: {}", id)))
    }

    pub async fn list(pool: &SqlitePool, status: Option<&str>) -> AppResult<Vec<Goal>> {
        match status {
            Some(s) => {
                sqlx::query_as::<_, Goal>("SELECT * FROM goals WHERE status = ? ORDER BY updated_at DESC")
                    .bind(s)
                    .fetch_all(pool)
                    .await
            }
            None => {
                sqlx::query_as::<_, Goal>("SELECT * FROM goals ORDER BY updated_at DESC")
                    .fetch_all(pool)
                    .await
            }
        }
        .map_err(AppError::Database)
    }

    pub async fn update(pool: &SqlitePool, id: &str, input: UpdateGoalInput) -> AppResult<Goal> {
        let now = chrono::Utc::now().to_rfc3339();
        let goal = Self::get_by_id(pool, id).await?;

        let title = input.title.unwrap_or(goal.title);
        let summary = input.summary.or(goal.summary);
        let content = input.content.unwrap_or(goal.content);

        sqlx::query(
            "UPDATE goals SET title = ?, summary = ?, content = ?, updated_at = ? WHERE id = ?"
        )
        .bind(&title)
        .bind(&summary)
        .bind(&content)
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await?;

        Self::get_by_id(pool, id).await
    }

    pub async fn update_status(pool: &SqlitePool, id: &str, status: GoalStatus) -> AppResult<Goal> {
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query("UPDATE goals SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status.to_string())
            .bind(&now)
            .bind(id)
            .execute(pool)
            .await?;

        Self::get_by_id(pool, id).await
    }

    pub async fn update_summary(pool: &SqlitePool, id: &str, summary: &str) -> AppResult<Goal> {
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query("UPDATE goals SET summary = ?, updated_at = ? WHERE id = ?")
            .bind(summary)
            .bind(&now)
            .bind(id)
            .execute(pool)
            .await?;

        Self::get_by_id(pool, id).await
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM goals WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Goal not found: {}", id)));
        }

        Ok(())
    }
}
