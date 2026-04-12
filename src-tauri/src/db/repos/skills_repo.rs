use sqlx::SqlitePool;

use crate::db::models::Skill;
use crate::error::{AppError, AppResult};

pub struct SkillsRepo;

impl SkillsRepo {
    pub async fn create(
        pool: &SqlitePool,
        milestone_id: &str,
        name: &str,
        description: &str,
    ) -> AppResult<Skill> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO skills (id, milestone_id, name, description, status, created_at, updated_at)
             VALUES (?, ?, ?, ?, 'requested', ?, ?)"
        )
        .bind(&id)
        .bind(milestone_id)
        .bind(name)
        .bind(description)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await?;

        Self::get_by_id(pool, &id).await
    }

    pub async fn get_by_id(pool: &SqlitePool, id: &str) -> AppResult<Skill> {
        sqlx::query_as::<_, Skill>("SELECT * FROM skills WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Skill not found: {}", id)))
    }

    pub async fn list_by_milestone(pool: &SqlitePool, milestone_id: &str) -> AppResult<Vec<Skill>> {
        sqlx::query_as::<_, Skill>("SELECT * FROM skills WHERE milestone_id = ? ORDER BY created_at")
            .bind(milestone_id)
            .fetch_all(pool)
            .await
            .map_err(AppError::Database)
    }

    pub async fn update_status(pool: &SqlitePool, id: &str, status: &str) -> AppResult<Skill> {
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query("UPDATE skills SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status)
            .bind(&now)
            .bind(id)
            .execute(pool)
            .await?;

        Self::get_by_id(pool, id).await
    }

    pub async fn update_test_report(pool: &SqlitePool, id: &str, test_report: &str) -> AppResult<Skill> {
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query("UPDATE skills SET test_report = ?, updated_at = ? WHERE id = ?")
            .bind(test_report)
            .bind(&now)
            .bind(id)
            .execute(pool)
            .await?;

        Self::get_by_id(pool, id).await
    }

    pub async fn update_source_ref(pool: &SqlitePool, id: &str, source_ref: &str) -> AppResult<Skill> {
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query("UPDATE skills SET source_code_ref = ?, updated_at = ? WHERE id = ?")
            .bind(source_ref)
            .bind(&now)
            .bind(id)
            .execute(pool)
            .await?;

        Self::get_by_id(pool, id).await
    }
}
