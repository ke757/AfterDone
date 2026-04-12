use sqlx::SqlitePool;

use crate::db::models::Message;
use crate::error::AppResult;

pub struct MessagesRepo;

impl MessagesRepo {
    pub async fn create(
        pool: &SqlitePool,
        goal_id: &str,
        role: &str,
        content: &str,
        metadata: Option<&str>,
    ) -> AppResult<Message> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO messages (id, goal_id, role, content, metadata, created_at)
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(goal_id)
        .bind(role)
        .bind(content)
        .bind(metadata)
        .bind(&now)
        .execute(pool)
        .await?;

        Self::get_by_id(pool, &id).await
    }

    pub async fn get_by_id(pool: &SqlitePool, id: &str) -> AppResult<Message> {
        sqlx::query_as::<_, Message>("SELECT * FROM messages WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("Message not found: {}", id)))
    }

    pub async fn list_by_goal(
        pool: &SqlitePool,
        goal_id: &str,
        limit: Option<u32>,
        before: Option<&str>,
    ) -> AppResult<Vec<Message>> {
        match (limit, before) {
            (Some(lim), Some(bef)) => {
                sqlx::query_as::<_, Message>(
                    "SELECT * FROM messages WHERE goal_id = ? AND created_at < (SELECT created_at FROM messages WHERE id = ?) ORDER BY created_at DESC LIMIT ?"
                )
                .bind(goal_id)
                .bind(bef)
                .bind(lim)
                .fetch_all(pool)
                .await
            }
            (Some(lim), None) => {
                sqlx::query_as::<_, Message>(
                    "SELECT * FROM messages WHERE goal_id = ? ORDER BY created_at DESC LIMIT ?"
                )
                .bind(goal_id)
                .bind(lim)
                .fetch_all(pool)
                .await
            }
            (None, Some(bef)) => {
                sqlx::query_as::<_, Message>(
                    "SELECT * FROM messages WHERE goal_id = ? AND created_at < (SELECT created_at FROM messages WHERE id = ?) ORDER BY created_at DESC"
                )
                .bind(goal_id)
                .bind(bef)
                .fetch_all(pool)
                .await
            }
            (None, None) => {
                sqlx::query_as::<_, Message>(
                    "SELECT * FROM messages WHERE goal_id = ? ORDER BY created_at DESC"
                )
                .bind(goal_id)
                .fetch_all(pool)
                .await
            }
        }
        .map_err(crate::error::AppError::Database)
    }
}
