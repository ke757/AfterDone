use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use super::super::types::*;

/// Repository for GeneratorTask operations
pub struct GeneratorTaskRepo;

impl GeneratorTaskRepo {
    /// Create a new GeneratorTask
    pub async fn create(db: &SqlitePool, input: CreateGeneratorTaskInput) -> AppResult<GeneratorTask> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let status = GeneratorTaskStatus::Pending.to_string();

        sqlx::query(
            "INSERT INTO generator_tasks (id, workspace_id, worknode_id, specification, status, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&input.workspace_id)
        .bind(&input.worknode_id)
        .bind(&input.specification)
        .bind(&status)
        .bind(&now)
        .bind(&now)
        .execute(db)
        .await?;

        Self::get_by_id(db, &id).await
    }

    /// Get GeneratorTask by id
    pub async fn get_by_id(db: &SqlitePool, id: &str) -> AppResult<GeneratorTask> {
        sqlx::query_as::<_, GeneratorTask>(
            "SELECT id, workspace_id, worknode_id, specification, status,
                    result, user_manual, skills_json, created_at, updated_at
             FROM generator_tasks WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("GeneratorTask not found: {}", id)))
    }

    /// Get pending tasks for a WorkSpace
    pub async fn get_pending_by_workspace(db: &SqlitePool, workspace_id: &str) -> AppResult<Option<GeneratorTask>> {
        sqlx::query_as::<_, GeneratorTask>(
            "SELECT id, workspace_id, worknode_id, specification, status,
                    result, user_manual, skills_json, created_at, updated_at
             FROM generator_tasks
             WHERE workspace_id = ? AND status IN ('pending', 'running')
             ORDER BY created_at DESC LIMIT 1"
        )
        .bind(workspace_id)
        .fetch_optional(db)
        .await
        .map_err(AppError::Database)
    }

    /// Update GeneratorTask
    pub async fn update(db: &SqlitePool, id: &str, input: UpdateGeneratorTaskInput) -> AppResult<()> {
        let now = chrono::Utc::now().to_rfc3339();

        if let Some(status) = &input.status {
            sqlx::query("UPDATE generator_tasks SET status = ?, updated_at = ? WHERE id = ?")
                .bind(status)
                .bind(&now)
                .bind(id)
                .execute(db)
                .await?;
        }

        if let Some(result) = &input.result {
            sqlx::query("UPDATE generator_tasks SET result = ?, updated_at = ? WHERE id = ?")
                .bind(result)
                .bind(&now)
                .bind(id)
                .execute(db)
                .await?;
        }

        if let Some(user_manual) = &input.user_manual {
            sqlx::query("UPDATE generator_tasks SET user_manual = ?, updated_at = ? WHERE id = ?")
                .bind(user_manual)
                .bind(&now)
                .bind(id)
                .execute(db)
                .await?;
        }

        if let Some(skills_json) = &input.skills_json {
            sqlx::query("UPDATE generator_tasks SET skills_json = ?, updated_at = ? WHERE id = ?")
                .bind(skills_json)
                .bind(&now)
                .bind(id)
                .execute(db)
                .await?;
        }

        Ok(())
    }

    /// Set task status to running
    pub async fn set_running(db: &SqlitePool, id: &str) -> AppResult<()> {
        Self::update(db, id, UpdateGeneratorTaskInput {
            status: Some(GeneratorTaskStatus::Running.to_string()),
            result: None,
            user_manual: None,
            skills_json: None,
        }).await
    }

    /// Complete task with result
    pub async fn complete(
        db: &SqlitePool,
        id: &str,
        result: &str,
        user_manual: Option<&str>,
        skills_json: Option<&str>,
    ) -> AppResult<()> {
        Self::update(db, id, UpdateGeneratorTaskInput {
            status: Some(GeneratorTaskStatus::Completed.to_string()),
            result: Some(result.to_string()),
            user_manual: user_manual.map(|s| s.to_string()),
            skills_json: skills_json.map(|s| s.to_string()),
        }).await
    }

    /// Fail task
    pub async fn fail(db: &SqlitePool, id: &str, error: &str) -> AppResult<()> {
        Self::update(db, id, UpdateGeneratorTaskInput {
            status: Some(GeneratorTaskStatus::Failed.to_string()),
            result: Some(error.to_string()),
            user_manual: None,
            skills_json: None,
        }).await
    }

    /// Delete GeneratorTask
    pub async fn delete(db: &SqlitePool, id: &str) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM generator_tasks WHERE id = ?")
            .bind(id)
            .execute(db)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("GeneratorTask not found: {}", id)));
        }

        Ok(())
    }
}
