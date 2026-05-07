use sqlx::SqlitePool;
use serde_json;

use crate::error::{AppError, AppResult};
use super::super::types::*;

/// Repository for WorkNode operations
pub struct WorkNodeRepo;

impl WorkNodeRepo {
    /// Create a new WorkNode
    pub async fn create(db: &SqlitePool, input: CreateWorkNodeInput) -> AppResult<WorkNode> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let status = input.status.unwrap_or_else(|| WorkNodeStatus::Planned.to_string());
        let node_type = input.node_type.unwrap_or_else(|| "version".to_string());

        sqlx::query(
            "INSERT INTO worknodes (id, workspace_id, parent_node_id, node_order, node_type, status, milestone_id, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&input.workspace_id)
        .bind(&input.parent_node_id)
        .bind(input.node_order)
        .bind(&node_type)
        .bind(&status)
        .bind(&input.milestone_id)
        .bind(&now)
        .bind(&now)
        .execute(db)
        .await?;

        Self::get_by_id(db, &id).await
    }

    /// Get WorkNode by id
    pub async fn get_by_id(db: &SqlitePool, id: &str) -> AppResult<WorkNode> {
        sqlx::query_as::<_, WorkNode>(
            "SELECT id, workspace_id, parent_node_id, node_order, node_type, status,
                    bug_md, user_manual_md, conclusion_md, milestone_id,
                    plan_summary, result_summary, created_at, updated_at
             FROM worknodes WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("WorkNode not found: {}", id)))
    }

    /// List all WorkNodes for a WorkSpace
    pub async fn list_by_workspace(db: &SqlitePool, workspace_id: &str) -> AppResult<Vec<WorkNode>> {
        sqlx::query_as::<_, WorkNode>(
            "SELECT id, workspace_id, parent_node_id, node_order, node_type, status,
                    bug_md, user_manual_md, conclusion_md, milestone_id,
                    plan_summary, result_summary, created_at, updated_at
             FROM worknodes WHERE workspace_id = ?
             ORDER BY node_order ASC"
        )
        .bind(workspace_id)
        .fetch_all(db)
        .await
        .map_err(AppError::Database)
    }

    /// Get children of a WorkNode
    pub async fn list_children(db: &SqlitePool, parent_id: &str) -> AppResult<Vec<WorkNode>> {
        sqlx::query_as::<_, WorkNode>(
            "SELECT id, workspace_id, parent_node_id, node_order, node_type, status,
                    bug_md, user_manual_md, conclusion_md, milestone_id,
                    plan_summary, result_summary, created_at, updated_at
             FROM worknodes WHERE parent_node_id = ?
             ORDER BY node_order ASC"
        )
        .bind(parent_id)
        .fetch_all(db)
        .await
        .map_err(AppError::Database)
    }

    /// Update WorkNode status
    pub async fn update_status(db: &SqlitePool, id: &str, status: &str) -> AppResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE worknodes SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status)
            .bind(&now)
            .bind(id)
            .execute(db)
            .await?;
        Ok(())
    }

    /// Update BUG.md (JSON array of BugEntry)
    pub async fn update_bug_md(db: &SqlitePool, id: &str, bugs: &[BugEntry]) -> AppResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let bug_json = serde_json::to_string(bugs)?;
        sqlx::query("UPDATE worknodes SET bug_md = ?, updated_at = ? WHERE id = ?")
            .bind(&bug_json)
            .bind(&now)
            .bind(id)
            .execute(db)
            .await?;
        Ok(())
    }

    /// Get BUG.md as BugEntry array
    pub async fn get_bugs(db: &SqlitePool, id: &str) -> AppResult<Vec<BugEntry>> {
        let node = Self::get_by_id(db, id).await?;
        match node.bug_md {
            Some(json) => {
                let bugs: Vec<BugEntry> = serde_json::from_str(&json)?;
                Ok(bugs)
            }
            None => Ok(Vec::new()),
        }
    }

    /// Update USER_MANUAL.md
    pub async fn update_user_manual_md(db: &SqlitePool, id: &str, content: &str) -> AppResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE worknodes SET user_manual_md = ?, updated_at = ? WHERE id = ?")
            .bind(content)
            .bind(&now)
            .bind(id)
            .execute(db)
            .await?;
        Ok(())
    }

    /// Update CONCLUSION.md
    pub async fn update_conclusion_md(db: &SqlitePool, id: &str, content: &str) -> AppResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE worknodes SET conclusion_md = ?, updated_at = ? WHERE id = ?")
            .bind(content)
            .bind(&now)
            .bind(id)
            .execute(db)
            .await?;
        Ok(())
    }

    /// Delete WorkNode
    pub async fn delete(db: &SqlitePool, id: &str) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM worknodes WHERE id = ?")
            .bind(id)
            .execute(db)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("WorkNode not found: {}", id)));
        }

        Ok(())
    }
}
