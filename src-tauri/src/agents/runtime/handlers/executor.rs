use async_trait::async_trait;

use super::ResultHandler;
use crate::agents::AgentOutput;
use crate::db::DatabasePool;
use crate::error::{AppError, AppResult};
use crate::workhub::{AgentType, GoalsRepo, GoalStatus};

pub struct ExecutorResultHandler;

#[async_trait]
impl ResultHandler for ExecutorResultHandler {
    fn agent_type(&self) -> AgentType {
        AgentType::Executor
    }

    fn to_effect_content(&self, output: &AgentOutput) -> AppResult<serde_json::Value> {
        match output {
            AgentOutput::ExecutionResult { .. } => Ok(serde_json::to_value(output)?),
            _ => Err(AppError::Agent(
                "Executor handler received unexpected output variant".into(),
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
            AgentOutput::ExecutionResult {
                milestone_id: _,
                success,
                bugs_found,
                ..
            } => {
                if *success {
                    GoalsRepo::update_status(db, goal_id, GoalStatus::Reached).await?;
                } else {
                    GoalsRepo::update_status(db, goal_id, GoalStatus::Failed).await?;
                    if !bugs_found.is_empty() {
                        tracing::info!("Execution found {} bugs", bugs_found.len());
                    }
                }
            }
            _ => {
                return Err(AppError::Agent(
                    "Executor persist received unexpected output variant".into(),
                ))
            }
        }

        Ok(())
    }
}
