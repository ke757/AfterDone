use async_trait::async_trait;

use super::{HandlerContext, ResultHandler};
use crate::agents::AgentOutput;
use crate::db::DatabasePool;
use crate::error::{AppError, AppResult};
use crate::session::cell::result::{ResultStatus, StoredResult};
use crate::workhub::{AgentType, GoalsRepo, GoalStatus};

pub struct SummarizerResultHandler;

#[async_trait]
impl ResultHandler for SummarizerResultHandler {
    fn agent_type(&self) -> AgentType {
        AgentType::Summarizer
    }

    async fn handle(&self, output: AgentOutput, ctx: &HandlerContext) -> AppResult<StoredResult> {
        let status = match &output {
            AgentOutput::ConversationTurn { summary_draft, .. } => {
                if summary_draft.is_some() {
                    ResultStatus::Ready
                } else {
                    ResultStatus::Active
                }
            }
            AgentOutput::GoalSummary { .. } => ResultStatus::Ready,
            _ => {
                return Err(AppError::Agent(
                    "Summarizer handler received unexpected output variant".into(),
                ))
            }
        };

        Ok(StoredResult::new(
            ctx.session_id.clone(),
            ctx.workspace_id.clone(),
            ctx.goal_id.clone(),
            AgentType::Summarizer,
            serde_json::to_value(&output)?,
            status,
        ))
    }

    async fn persist(&self, db: &DatabasePool, stored: &StoredResult) -> AppResult<()> {
        let output: AgentOutput = serde_json::from_value(stored.output.clone())?;

        match output {
            AgentOutput::GoalSummary {
                title,
                description,
                acceptance_criteria,
                constraints,
                refinement_questions,
            } => {
                crate::services::SummarizerService::persist_summary(
                    db,
                    &stored.goal_id,
                    &crate::workhub::GoalSummary {
                        title: title.clone(),
                        description: description.clone(),
                        acceptance_criteria: acceptance_criteria.clone(),
                        constraints: constraints.clone(),
                        refinement_questions: refinement_questions.clone(),
                    },
                ).await?;

                GoalsRepo::update_status(db, &stored.goal_id, GoalStatus::Draft).await?;
                crate::workhub::AgentLogsRepo::append(
                    db,
                    &stored.goal_id,
                    "summarizer",
                    "summarizing",
                    "completed",
                    Some(&serde_json::to_string(&stored.output)?),
                ).await?;
            }
            AgentOutput::ConversationTurn { .. } => {
                // 对话 turn 不自动持久化，等待用户确认
            }
            _ => {
                return Err(AppError::Agent(
                    "Summarizer persist received unexpected output variant".into(),
                ))
            }
        }

        Ok(())
    }
}
