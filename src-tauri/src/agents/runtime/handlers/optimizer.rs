use async_trait::async_trait;

use super::ResultHandler;
use crate::agents::AgentOutput;
use crate::db::DatabasePool;
use crate::error::{AppError, AppResult};
use crate::workhub::{AgentType, GoalsRepo, GoalStatus, AgentLogsRepo};

pub struct OptimizerResultHandler;

#[async_trait]
impl ResultHandler for OptimizerResultHandler {
    fn agent_type(&self) -> AgentType {
        AgentType::Optimizer
    }

    fn to_effect_content(&self, output: &AgentOutput) -> AppResult<serde_json::Value> {
        match output {
            AgentOutput::OptimizationResult { .. } => Ok(serde_json::to_value(output)?),
            _ => Err(AppError::Agent(
                "Optimizer handler received unexpected output variant".into(),
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
            AgentOutput::OptimizationResult {
                milestone_id: _,
                solidified,
                optimizations,
                new_node_id,
            } => {
                if *solidified {
                    GoalsRepo::update_status(db, goal_id, GoalStatus::Archived).await?;
                } else {
                    GoalsRepo::update_status(db, goal_id, GoalStatus::Reached).await?;
                }
                AgentLogsRepo::append(
                    db,
                    goal_id,
                    "optimizer",
                    "optimizing",
                    "completed",
                    Some(&serde_json::json!({
                        "solidified": solidified,
                        "optimizations": optimizations.len(),
                        "new_node_id": new_node_id,
                    }).to_string()),
                ).await?;
            }
            _ => {
                return Err(AppError::Agent(
                    "Optimizer persist received unexpected output variant".into(),
                ))
            }
        }

        Ok(())
    }
}
