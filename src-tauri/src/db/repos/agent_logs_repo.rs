use sqlx::SqlitePool;

use crate::db::models::AgentLog;
use crate::error::AppResult;

pub struct AgentLogsRepo;

impl AgentLogsRepo {
    pub async fn append(
        pool: &SqlitePool,
        goal_id: &str,
        agent_type: &str,
        phase: &str,
        event_type: &str,
        payload: Option<&str>,
    ) -> AppResult<AgentLog> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO agent_logs (id, goal_id, agent_type, phase, event_type, payload, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(goal_id)
        .bind(agent_type)
        .bind(phase)
        .bind(event_type)
        .bind(payload)
        .bind(&now)
        .execute(pool)
        .await?;

        Ok(AgentLog {
            id,
            goal_id: goal_id.to_string(),
            agent_type: agent_type.to_string(),
            phase: phase.to_string(),
            event_type: event_type.to_string(),
            payload: payload.map(|s| s.to_string()),
            created_at: now,
        })
    }

    pub async fn list_by_goal(pool: &SqlitePool, goal_id: &str) -> AppResult<Vec<AgentLog>> {
        sqlx::query_as::<_, AgentLog>(
            "SELECT * FROM agent_logs WHERE goal_id = ? ORDER BY created_at"
        )
        .bind(goal_id)
        .fetch_all(pool)
        .await
        .map_err(crate::error::AppError::Database)
    }
}
