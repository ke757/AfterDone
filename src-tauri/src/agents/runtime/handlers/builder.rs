use async_trait::async_trait;

use super::{HandlerContext, ResultHandler};
use crate::agents::AgentOutput;
use crate::db::DatabasePool;
use crate::error::{AppError, AppResult};
use crate::session::cell::result::{ResultStatus, StoredResult};
use crate::workhub::{
    AgentType, GoalsRepo, GoalStatus, MilestonesRepo, AgentLogsRepo,
};

pub struct BuilderResultHandler;

#[async_trait]
impl ResultHandler for BuilderResultHandler {
    fn agent_type(&self) -> AgentType {
        AgentType::Builder
    }

    async fn handle(&self, output: AgentOutput, ctx: &HandlerContext) -> AppResult<StoredResult> {
        let status = match &output {
            AgentOutput::BuilderResult { success, .. } => {
                if *success {
                    ResultStatus::Ready
                } else {
                    ResultStatus::Failed
                }
            }
            _ => {
                return Err(AppError::Agent(
                    "Builder handler received unexpected output variant".into(),
                ))
            }
        };

        Ok(StoredResult::new(
            ctx.session_id.clone(),
            ctx.workspace_id.clone(),
            ctx.goal_id.clone(),
            AgentType::Builder,
            serde_json::to_value(&output)?,
            status,
        ))
    }

    async fn persist(&self, db: &DatabasePool, stored: &StoredResult) -> AppResult<()> {
        let output: AgentOutput = serde_json::from_value(stored.output.clone())?;

        match output {
            AgentOutput::BuilderResult {
                milestone_id,
                success,
                failure_reason,
                new_node_id,
                skills_used,
                ..
            } => {
                if success {
                    GoalsRepo::update_status(db, &stored.goal_id, GoalStatus::Reached).await?;
                    if !milestone_id.is_empty() {
                        MilestonesRepo::update_status(db, &milestone_id, "completed").await?;
                    }
                    AgentLogsRepo::append(
                        db,
                        &stored.goal_id,
                        "builder",
                        "building",
                        "reached",
                        Some(&serde_json::json!({
                            "new_node_id": new_node_id,
                            "skills_used": skills_used,
                        }).to_string()),
                    ).await?;
                } else {
                    GoalsRepo::update_status(db, &stored.goal_id, GoalStatus::Failed).await?;
                    AgentLogsRepo::append(
                        db,
                        &stored.goal_id,
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
