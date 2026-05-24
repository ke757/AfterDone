use async_trait::async_trait;

use super::ResultHandler;
use crate::agents::AgentOutput;
use crate::db::DatabasePool;
use crate::error::{AppError, AppResult};
use crate::workhub::{AgentType, GoalsRepo, GoalStatus, AgentLogsRepo};

pub struct BuilderResultHandler;

#[async_trait]
impl ResultHandler for BuilderResultHandler {
    fn agent_type(&self) -> AgentType {
        AgentType::Builder
    }

    fn to_effect_content(&self, output: &AgentOutput) -> AppResult<serde_json::Value> {
        match output {
            AgentOutput::BuilderResult { .. } => Ok(serde_json::to_value(output)?),
            _ => Err(AppError::Agent(
                "Builder handler received unexpected output variant".into(),
            )),
        }
    }

    async fn persist(
        &self,
        db: &DatabasePool,
        output: &AgentOutput,
        goal_id: &str,
        _workspace_id: &str,
    ) -> AppResult<()> {
        match output {
            AgentOutput::BuilderResult {
                milestone_id: _,
                success,
                failure_reason,
                new_node_id,
                skills_used,
                ..
            } => {
                if *success {
                    GoalsRepo::update_status(db, goal_id, GoalStatus::Reached).await?;
                    AgentLogsRepo::append(
                        db,
                        goal_id,
                        "builder",
                        "building",
                        "reached",
                        Some(&serde_json::json!({
                            "new_node_id": new_node_id,
                            "skills_used": skills_used,
                        }).to_string()),
                    ).await?;
                } else {
                    GoalsRepo::update_status(db, goal_id, GoalStatus::Failed).await?;
                    AgentLogsRepo::append(
                        db,
                        goal_id,
                        "builder",
                        "building",
                        "failed",
                        failure_reason.as_deref(),
                    ).await?;
                }
            }
            _ => {
                return Err(AppError::Agent(
                    "Builder persist received unexpected output variant".into(),
                ))
            }
        }

        Ok(())
    }
}
