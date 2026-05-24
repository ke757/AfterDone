use async_trait::async_trait;

use super::ResultHandler;
use crate::agents::AgentOutput;
use crate::db::DatabasePool;
use crate::error::{AppError, AppResult};
use crate::workhub::{AgentType, GoalsRepo, GoalStatus};

pub struct SummarizerResultHandler;

#[async_trait]
impl ResultHandler for SummarizerResultHandler {
    fn agent_type(&self) -> AgentType {
        AgentType::Summarizer
    }

    fn to_effect_content(&self, output: &AgentOutput) -> AppResult<serde_json::Value> {
        match output {
            AgentOutput::ConversationTurn { .. } | AgentOutput::GoalSummary { .. } => {
                Ok(serde_json::to_value(output)?)
            }
            _ => Err(AppError::Agent(
                "Summarizer handler received unexpected output variant".into(),
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
            AgentOutput::GoalSummary {
                title,
                description,
                acceptance_criteria,
                constraints,
                refinement_questions,
            } => {
                crate::services::SummarizerService::persist_summary(
                    db,
                    goal_id,
                    &crate::workhub::GoalSummary {
                        title: title.clone(),
                        description: description.clone(),
                        acceptance_criteria: acceptance_criteria.clone(),
                        constraints: constraints.clone(),
                        refinement_questions: refinement_questions.clone(),
                    },
                ).await?;

                GoalsRepo::update_status(db, goal_id, GoalStatus::Draft).await?;
                crate::workhub::AgentLogsRepo::append(
                    db,
                    goal_id,
                    "summarizer",
                    "summarizing",
                    "completed",
                    Some(&serde_json::to_string(output)?),
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
