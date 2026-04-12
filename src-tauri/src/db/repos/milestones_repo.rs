use sqlx::SqlitePool;

use crate::db::models::Milestone;
use crate::error::{AppError, AppResult};

pub struct MilestonesRepo;

impl MilestonesRepo {
    pub async fn create(
        pool: &SqlitePool,
        goal_id: &str,
        milestone_order: i32,
        parent_milestone_id: Option<&str>,
        plan: Option<&str>,
        plan_summary: Option<&str>,
    ) -> AppResult<Milestone> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO milestones (id, goal_id, milestone_order, parent_milestone_id, plan, plan_summary, status, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, 'planned', ?, ?)"
        )
        .bind(&id)
        .bind(goal_id)
        .bind(milestone_order)
        .bind(parent_milestone_id)
        .bind(plan)
        .bind(plan_summary)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await?;

        Self::get_by_id(pool, &id).await
    }

    pub async fn get_by_id(pool: &SqlitePool, id: &str) -> AppResult<Milestone> {
        sqlx::query_as::<_, Milestone>("SELECT * FROM milestones WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Milestone not found: {}", id)))
    }

    pub async fn list_by_goal(pool: &SqlitePool, goal_id: &str) -> AppResult<Vec<Milestone>> {
        sqlx::query_as::<_, Milestone>(
            "SELECT * FROM milestones WHERE goal_id = ? ORDER BY milestone_order, created_at"
        )
        .bind(goal_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn update_status(pool: &SqlitePool, id: &str, status: &str) -> AppResult<Milestone> {
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query("UPDATE milestones SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status)
            .bind(&now)
            .bind(id)
            .execute(pool)
            .await?;

        Self::get_by_id(pool, id).await
    }

    pub async fn update_plan(
        pool: &SqlitePool,
        id: &str,
        plan: &str,
        plan_summary: &str,
    ) -> AppResult<Milestone> {
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query("UPDATE milestones SET plan = ?, plan_summary = ?, updated_at = ? WHERE id = ?")
            .bind(plan)
            .bind(plan_summary)
            .bind(&now)
            .bind(id)
            .execute(pool)
            .await?;

        Self::get_by_id(pool, id).await
    }

    pub async fn update_skill_path(pool: &SqlitePool, id: &str, skill_path: &str) -> AppResult<Milestone> {
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query("UPDATE milestones SET skill_path = ?, updated_at = ? WHERE id = ?")
            .bind(skill_path)
            .bind(&now)
            .bind(id)
            .execute(pool)
            .await?;

        Self::get_by_id(pool, id).await
    }

    pub async fn update_result_summary(pool: &SqlitePool, id: &str, result_summary: &str) -> AppResult<Milestone> {
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query("UPDATE milestones SET result_summary = ?, updated_at = ? WHERE id = ?")
            .bind(result_summary)
            .bind(&now)
            .bind(id)
            .execute(pool)
            .await?;

        Self::get_by_id(pool, id).await
    }

    pub async fn get_children(pool: &SqlitePool, parent_id: &str) -> AppResult<Vec<Milestone>> {
        sqlx::query_as::<_, Milestone>("SELECT * FROM milestones WHERE parent_milestone_id = ? ORDER BY milestone_order")
            .bind(parent_id)
            .fetch_all(pool)
            .await
            .map_err(AppError::Database)
    }
}
