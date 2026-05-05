use async_trait::async_trait;

use super::{HandlerContext, ResultHandler};
use crate::agents::AgentOutput;
use crate::db::DatabasePool;
use crate::error::{AppError, AppResult};
use crate::memory::{ResultStatus, StoredResult};
use crate::workhub::{
    AgentType, GoalsRepo, GoalStatus, MilestonesRepo,
};

pub struct ExecutorResultHandler;

#[async_trait]
impl ResultHandler for ExecutorResultHandler {
    fn agent_type(&self) -> AgentType {
        AgentType::Executor
    }

    async fn handle(&self, output: AgentOutput, ctx: &HandlerContext) -> AppResult<StoredResult> {
        let status = match &output {
            AgentOutput::ExecutionResult { success, .. } => {
                if *success {
                    ResultStatus::Ready
                } else {
                    ResultStatus::Failed
                }
            }
            _ => {
                return Err(AppError::Agent(
                    "Executor handler received unexpected output variant".into(),
                ))
            }
        };

        Ok(StoredResult::new(
            ctx.session_id.clone(),
            ctx.workspace_id.clone(),
            ctx.goal_id.clone(),
            AgentType::Executor,
            serde_json::to_value(&output)?,
            status,
        ))
    }

    async fn persist(&self, db: &DatabasePool, stored: &StoredResult) -> AppResult<()> {
        let output: AgentOutput = serde_json::from_value(stored.output.clone())?;

        match output {
            AgentOutput::ExecutionResult {
                milestone_id,
                success,
                bugs_found,
                ..
            } => {
                if success {
                    GoalsRepo::update_status(db, &stored.goal_id, GoalStatus::Reached).await?;
                } else {
                    GoalsRepo::update_status(db, &stored.goal_id, GoalStatus::Failed).await?;
                    if !bugs_found.is_empty() {
                        tracing::info!("Execution found {} bugs", bugs_found.len());
                    }
                }
                if !milestone_id.is_empty() {
                    let status = if success { "completed" } else { "failed" };
                    MilestonesRepo::update_status(db, &milestone_id, status).await?;
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
