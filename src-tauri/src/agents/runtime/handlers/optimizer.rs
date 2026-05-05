use async_trait::async_trait;

use super::{HandlerContext, ResultHandler};
use crate::agents::AgentOutput;
use crate::db::DatabasePool;
use crate::error::{AppError, AppResult};
use crate::session::cell::result::{ResultStatus, StoredResult};
use crate::workhub::{
    AgentType, GoalsRepo, GoalStatus, MilestonesRepo, AgentLogsRepo,
};

pub struct OptimizerResultHandler;

#[async_trait]
impl ResultHandler for OptimizerResultHandler {
    fn agent_type(&self) -> AgentType {
        AgentType::Optimizer
    }

    async fn handle(&self, output: AgentOutput, ctx: &HandlerContext) -> AppResult<StoredResult> {
        let status = match &output {
            AgentOutput::OptimizationResult { solidified, .. } => {
                if *solidified {
                    ResultStatus::Confirmed
                } else {
                    ResultStatus::Ready
                }
            }
            _ => {
                return Err(AppError::Agent(
                    "Optimizer handler received unexpected output variant".into(),
                ))
            }
        };

        Ok(StoredResult::new(
            ctx.session_id.clone(),
            ctx.workspace_id.clone(),
            ctx.goal_id.clone(),
            AgentType::Optimizer,
            serde_json::to_value(&output)?,
            status,
        ))
    }

    async fn persist(&self, db: &DatabasePool, stored: &StoredResult) -> AppResult<()> {
        let output: AgentOutput = serde_json::from_value(stored.output.clone())?;

        match output {
            AgentOutput::OptimizationResult {
                milestone_id,
                solidified,
                optimizations,
                new_node_id,
            } => {
                if solidified {
                    GoalsRepo::update_status(db, &stored.goal_id, GoalStatus::Archived).await?;
                } else {
                    GoalsRepo::update_status(db, &stored.goal_id, GoalStatus::Reached).await?;
                }
                if !milestone_id.is_empty() {
                    MilestonesRepo::update_status(db, &milestone_id, "completed").await?;
                }
                AgentLogsRepo::append(
                    db,
                    &stored.goal_id,
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
