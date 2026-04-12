use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::agents::summarizer::GoalSummarizerAgent;
use crate::agents::traits::SideCarAgent;
use crate::agents::types::{AgentOutput, AgentStatus, AgentTaskHandle, GoalContext};
use crate::comm::Transport;
use crate::db::models::AgentType;
use crate::db::{DatabasePool, GoalsRepo, MessagesRepo, SkillsRepo, MilestonesRepo, AgentLogsRepo};
use crate::error::{AppError, AppResult};
use crate::events::EventBridge;
use crate::llm::LlmProvider;

/// The AgentSupervisor is the central coordinator.
/// It owns the lifecycle of all agent tasks and manages phase transitions.
pub struct AgentSupervisor {
    db: DatabasePool,
    transport: Arc<dyn Transport>,
    llm: Arc<dyn LlmProvider>,
    emitter: EventBridge,
    tasks: Arc<RwLock<HashMap<String, AgentTaskHandle>>>,
}

impl AgentSupervisor {
    pub fn new(
        db: DatabasePool,
        transport: Arc<dyn Transport>,
        llm: Arc<dyn LlmProvider>,
        emitter: EventBridge,
    ) -> Self {
        Self {
            db,
            transport,
            llm,
            emitter,
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start an agent for the given goal.
    /// The agent type is determined by the current goal status.
    pub async fn start_agent(&self, goal_id: &str) -> AppResult<AgentStatus> {
        let goal = GoalsRepo::get_by_id(&self.db, goal_id).await?;
        let agent_type = determine_agent_type(&goal.status)?;

        // Check if an agent is already running for this goal
        {
            let tasks = self.tasks.read().await;
            if let Some(handle) = tasks.get(goal_id) {
                if handle.status == AgentStatus::Running || handle.status == AgentStatus::Starting {
                    return Ok(handle.status.clone());
                }
            }
        }

        let cancel_token = tokio_util::sync::CancellationToken::new();
        let goal_id_owned = goal_id.to_string();

        // Build goal context from DB
        let ctx = self.build_goal_context(&goal).await?;

        // Create the appropriate agent
        let agent: Box<dyn SideCarAgent> = match agent_type {
            AgentType::Summarizer => Box::new(GoalSummarizerAgent::new(cancel_token.clone())),
            _ => return Err(AppError::Agent(format!(
                "Agent type {} not yet implemented", agent_type
            ))),
        };

        // Register the task handle
        let handle = AgentTaskHandle {
            agent_type: agent_type.clone(),
            cancel_token: cancel_token.clone(),
            status: AgentStatus::Starting,
            started_at: chrono::Utc::now(),
        };
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(goal_id.to_string(), handle);
        }

        // Update goal status
        let new_status = match agent_type {
            AgentType::Summarizer => "summarizing",
            AgentType::Executor => "executing",
            AgentType::Optimizer => "optimizing",
        };
        let previous_status = goal.status.clone();
        GoalsRepo::update_status(&self.db, goal_id, new_status).await?;
        self.emitter.emit_goal_status(goal_id, new_status, &previous_status);

        // Spawn the agent task
        let db = self.db.clone();
        let transport = self.transport.clone();
        let llm = self.llm.clone();
        let emitter = self.emitter.clone();
        let tasks = self.tasks.clone();

        tokio::spawn(async move {
            // Update status to Running
            {
                let mut tasks_guard = tasks.write().await;
                if let Some(h) = tasks_guard.get_mut(&goal_id_owned) {
                    h.status = AgentStatus::Running;
                }
            }

            let result = agent.run(ctx, transport, llm, emitter.clone()).await;

            // Handle result
            match result {
                Ok(output) => {
                    if let Err(e) = handle_agent_output(&db, &goal_id_owned, &output, &emitter).await {
                        tracing::error!("Failed to handle agent output: {}", e);
                    }
                    let mut tasks_guard = tasks.write().await;
                    if let Some(h) = tasks_guard.get_mut(&goal_id_owned) {
                        h.status = AgentStatus::Completed;
                    }
                }
                Err(e) => {
                    tracing::error!("Agent failed for goal {}: {}", goal_id_owned, e);
                    let mut tasks_guard = tasks.write().await;
                    if let Some(h) = tasks_guard.get_mut(&goal_id_owned) {
                        h.status = AgentStatus::Failed;
                    }
                    // Revert goal status on failure
                    let _ = GoalsRepo::update_status(&db, &goal_id_owned, "failed").await;
                    emitter.emit_goal_status(&goal_id_owned, "failed", new_status);
                }
            }
        });

        Ok(AgentStatus::Starting)
    }

    /// Stop the agent running for a goal
    pub async fn stop_agent(&self, goal_id: &str) -> AppResult<AgentStatus> {
        let mut tasks = self.tasks.write().await;
        if let Some(handle) = tasks.get_mut(goal_id) {
            handle.cancel_token.cancel();
            handle.status = AgentStatus::Canceling;
            Ok(handle.status.clone())
        } else {
            Err(AppError::NotFound(format!("No running agent for goal: {}", goal_id)))
        }
    }

    /// Get the status of the agent for a goal
    pub async fn get_status(&self, goal_id: &str) -> Option<AgentStatus> {
        let tasks = self.tasks.read().await;
        tasks.get(goal_id).map(|h| h.status.clone())
    }

    /// Build the GoalContext from the database
    async fn build_goal_context(&self, goal: &crate::db::models::Goal) -> AppResult<GoalContext> {
        let current_milestone = match &goal.current_milestone_id {
            Some(mid) => Some(MilestonesRepo::get_by_id(&self.db, mid).await?),
            None => None,
        };

        let conversation_history = MessagesRepo::list_by_goal(&self.db, &goal.id, None, None)
            .await?
            .into_iter()
            .rev() // Reverse to chronological order
            .collect();

        let available_skills = match &current_milestone {
            Some(m) => SkillsRepo::list_by_milestone(&self.db, &m.id).await?,
            None => vec![],
        };

        let agent_logs = AgentLogsRepo::list_by_goal(&self.db, &goal.id).await?;

        Ok(GoalContext {
            goal: goal.clone(),
            current_milestone,
            conversation_history,
            available_skills,
            agent_logs,
            metadata: HashMap::new(),
        })
    }
}

/// Determine which agent type should run based on goal status
fn determine_agent_type(goal_status: &str) -> AppResult<AgentType> {
    match goal_status {
        "draft" => Ok(AgentType::Summarizer),
        "pinned" => Ok(AgentType::Executor),
        "achieved" => Ok(AgentType::Optimizer),
        "failed" => Ok(AgentType::Executor), // Retry execution
        s => Err(AppError::Agent(format!(
            "No agent transition defined for goal status: {}", s
        ))),
    }
}

/// Handle the output from an agent run — persist results to DB
async fn handle_agent_output(
    db: &DatabasePool,
    goal_id: &str,
    output: &AgentOutput,
    emitter: &EventBridge,
) -> AppResult<()> {
    match output {
        AgentOutput::GoalSummary {
            title,
            description,
            acceptance_criteria,
            constraints,
            refinement_questions,
        } => {
            // Save the summary as JSON in the goal's summary field
            let summary_json = serde_json::json!({
                "title": title,
                "description": description,
                "acceptance_criteria": acceptance_criteria,
                "constraints": constraints,
                "refinement_questions": refinement_questions,
            });
            let summary_str = serde_json::to_string_pretty(&summary_json)?;

            let goal = GoalsRepo::update_summary(db, goal_id, &summary_str).await?;
            let _ = GoalsRepo::update(db, goal_id, crate::db::models::UpdateGoalInput {
                title: Some(title.clone()),
                summary: None,
                raw_input: None,
            }).await;

            // Transition to summarizing -> draft (waiting for user to pin)
            // Actually after summarizing, go back to "draft" so user can review and pin
            // Wait — the flow is: draft → summarizing → (summary ready, user reviews) → pinned
            // After summarizer completes, the goal status should transition to indicate "summary ready"
            // We use "draft" with a summary now available for the user to pin
            GoalsRepo::update_status(db, goal_id, "draft").await?;
            emitter.emit_goal_status(goal_id, "draft", "summarizing");

            // Log the agent decision
            AgentLogsRepo::append(
                db, goal_id, "summarizer", "summarizing", "completed",
                Some(&serde_json::to_string(&summary_json)?),
            ).await?;
        }

        AgentOutput::ExecutionResult {
            milestone_id,
            success,
            failure_reason,
            ..
        } => {
            if *success {
                GoalsRepo::update_status(db, goal_id, "achieved").await?;
                emitter.emit_goal_status(goal_id, "achieved", "executing");
            } else {
                GoalsRepo::update_status(db, goal_id, "failed").await?;
                emitter.emit_goal_status(goal_id, "failed", "executing");
            }

            if !milestone_id.is_empty() {
                let status = if *success { "completed" } else { "failed" };
                MilestonesRepo::update_status(db, milestone_id, status).await?;
                emitter.emit_milestone_status(milestone_id, status, "in_progress");
            }
        }

        AgentOutput::OptimizationResult {
            milestone_id,
            solidified,
            ..
        } => {
            if *solidified {
                GoalsRepo::update_status(db, goal_id, "solidified").await?;
                emitter.emit_goal_status(goal_id, "solidified", "optimizing");
            } else {
                GoalsRepo::update_status(db, goal_id, "achieved").await?;
                emitter.emit_goal_status(goal_id, "achieved", "optimizing");
            }

            if !milestone_id.is_empty() {
                MilestonesRepo::update_status(db, milestone_id, "completed").await?;
                emitter.emit_milestone_status(milestone_id, "completed", "in_progress");
            }
        }
    }

    Ok(())
}
