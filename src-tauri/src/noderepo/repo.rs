use sqlx::SqlitePool;
use serde_json;

use crate::error::{AppError, AppResult};
use super::types::*;

/// Repository for NodeSpace operations
pub struct NodeSpaceRepo;

impl NodeSpaceRepo {
    /// Create a new NodeSpace for a goal
    pub async fn create(db: &SqlitePool, input: CreateNodeSpaceInput) -> AppResult<NodeSpace> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO nodespaces (id, goal_id, goal_md, plan_md, created_at, updated_at)
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

    /// Get NodeSpace by id
    pub async fn get_by_id(db: &SqlitePool, id: &str) -> AppResult<NodeSpace> {
        sqlx::query_as::<_, NodeSpace>(
            "SELECT id, goal_id, current_node_id, goal_md, plan_md, created_at, updated_at
             FROM nodespaces WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("NodeSpace not found: {}", id)))
    }

    /// Get NodeSpace by goal_id
    pub async fn get_by_goal_id(db: &SqlitePool, goal_id: &str) -> AppResult<Option<NodeSpace>> {
        sqlx::query_as::<_, NodeSpace>(
            "SELECT id, goal_id, current_node_id, goal_md, plan_md, created_at, updated_at
             FROM nodespaces WHERE goal_id = ?"
        )
        .bind(goal_id)
        .fetch_optional(db)
        .await
        .map_err(AppError::Database)
    }

    /// Update GOAL.md
    pub async fn update_goal_md(db: &SqlitePool, id: &str, content: &str) -> AppResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE nodespaces SET goal_md = ?, updated_at = ? WHERE id = ?")
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
        sqlx::query("UPDATE nodespaces SET plan_md = ?, updated_at = ? WHERE id = ?")
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
        sqlx::query("UPDATE nodespaces SET current_node_id = ?, updated_at = ? WHERE id = ?")
            .bind(node_id)
            .bind(&now)
            .bind(id)
            .execute(db)
            .await?;
        Ok(())
    }

    /// Delete NodeSpace
    pub async fn delete(db: &SqlitePool, id: &str) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM nodespaces WHERE id = ?")
            .bind(id)
            .execute(db)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("NodeSpace not found: {}", id)));
        }

        Ok(())
    }
}

/// Repository for WorkNode operations
pub struct WorkNodeRepo;

impl WorkNodeRepo {
    /// Create a new WorkNode
    pub async fn create(db: &SqlitePool, input: CreateWorkNodeInput) -> AppResult<WorkNode> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let status = WorkNodeStatus::Planned.to_string();

        sqlx::query(
            "INSERT INTO worknodes (id, nodespace_id, parent_node_id, node_order, status, milestone_id, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&input.nodespace_id)
        .bind(&input.parent_node_id)
        .bind(input.node_order)
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
            "SELECT id, nodespace_id, parent_node_id, node_order, status,
                    bug_md, user_manual_md, conclusion_md, milestone_id,
                    plan_summary, result_summary, created_at, updated_at
             FROM worknodes WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("WorkNode not found: {}", id)))
    }

    /// List all WorkNodes for a NodeSpace
    pub async fn list_by_nodespace(db: &SqlitePool, nodespace_id: &str) -> AppResult<Vec<WorkNode>> {
        sqlx::query_as::<_, WorkNode>(
            "SELECT id, nodespace_id, parent_node_id, node_order, status,
                    bug_md, user_manual_md, conclusion_md, milestone_id,
                    plan_summary, result_summary, created_at, updated_at
             FROM worknodes WHERE nodespace_id = ?
             ORDER BY node_order ASC"
        )
        .bind(nodespace_id)
        .fetch_all(db)
        .await
        .map_err(AppError::Database)
    }

    /// Get children of a WorkNode
    pub async fn list_children(db: &SqlitePool, parent_id: &str) -> AppResult<Vec<WorkNode>> {
        sqlx::query_as::<_, WorkNode>(
            "SELECT id, nodespace_id, parent_node_id, node_order, status,
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

/// Repository for GeneratorTask operations
pub struct GeneratorTaskRepo;

impl GeneratorTaskRepo {
    /// Create a new GeneratorTask
    pub async fn create(db: &SqlitePool, input: CreateGeneratorTaskInput) -> AppResult<GeneratorTask> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let status = GeneratorTaskStatus::Pending.to_string();

        sqlx::query(
            "INSERT INTO generator_tasks (id, nodespace_id, worknode_id, specification, status, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&input.nodespace_id)
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
            "SELECT id, nodespace_id, worknode_id, specification, status,
                    result, user_manual, skills_json, created_at, updated_at
             FROM generator_tasks WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("GeneratorTask not found: {}", id)))
    }

    /// Get pending tasks for a NodeSpace
    pub async fn get_pending_by_nodespace(db: &SqlitePool, nodespace_id: &str) -> AppResult<Option<GeneratorTask>> {
        sqlx::query_as::<_, GeneratorTask>(
            "SELECT id, nodespace_id, worknode_id, specification, status,
                    result, user_manual, skills_json, created_at, updated_at
             FROM generator_tasks
             WHERE nodespace_id = ? AND status IN ('pending', 'running')
             ORDER BY created_at DESC LIMIT 1"
        )
        .bind(nodespace_id)
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
