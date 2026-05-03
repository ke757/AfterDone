use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::agents::GoalSummarizerAgent;
use crate::agents::BuilderAgent;
use crate::agents::ExecutorAgent;
use crate::agents::OptimizerAgent;
use crate::agents::traits::SideCarAgent;
use crate::agents::types::{AgentOutput, AgentStatus, AgentTaskHandle, GoalContext};
use crate::adapter::Transport;
use crate::workhub::{AgentType, GoalsRepo, MilestonesRepo, SkillsRepo, AgentLogsRepo, GoalStatus};
use crate::workhub::{WorkSpaceRepo};
use crate::db::DatabasePool;
use crate::error::{AppError, AppResult};
use crate::events::EventBridge;
use crate::llm::LlmProvider;
use crate::session::{ChatMemory, InMemoryChatMemory};
use crate::workhub::WorkHub;

/// The AgentSupervisor is the central coordinator(调度器).
/// It owns the lifecycle of all agent tasks and manages phase transitions(阶段转换).
pub struct AgentSupervisor {
    db: DatabasePool,
    transport: Arc<dyn Transport>,
    llm: Arc<dyn LlmProvider>,
    emitter: EventBridge,
    tasks: Arc<RwLock<HashMap<String, AgentTaskHandle>>>,
    memory: Arc<dyn ChatMemory>,
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
            memory: Arc::new(InMemoryChatMemory::new()),
        }
    }

    /// 创建一个带有自定义内存的AgentSupervisor.
    pub fn with_memory(
        db: DatabasePool,
        transport: Arc<dyn Transport>,
        llm: Arc<dyn LlmProvider>,
        emitter: EventBridge,
        memory: Arc<dyn ChatMemory>,
    ) -> Self {
        Self {
            db,
            transport,
            llm,
            emitter,
            tasks: Arc::new(RwLock::new(HashMap::new())),
            memory,
        }
    }

    /// 为给定的工作空间启动一个代理.
    /// - 根据对应目标状态确定代理类型.
    /// - 维护 agent 运行时的状态变更
    pub async fn start_agent(&self, workspace_id: &str) -> AppResult<AgentStatus> {
        let workspace = WorkSpaceRepo::get_by_id(&self.db, workspace_id).await?;
        let goal = GoalsRepo::get_by_id(&self.db, &workspace.goal_id).await?;
        let agent_type = determine_agent_type(goal.status.clone())?;

        // 检查是否已经为该工作空间运行了代理
        {
            let tasks = self.tasks.read().await;
            if let Some(handle) = tasks.get(workspace_id) {
                if handle.status == AgentStatus::Running || handle.status == AgentStatus::Starting {
                    return Ok(handle.status.clone());
                }
            }
        }

        // 创建取消令牌
        let cancel_token = tokio_util::sync::CancellationToken::new();
        let workspace_id_owned = workspace_id.to_string();
        let goal_id_owned = goal.id.clone();

        // 从DB构建目标上下文（包含工作空间信息）
        let ctx = self.build_goal_context(&goal).await?;

        // 创建适当的代理
        let agent: Box<dyn SideCarAgent> = match agent_type {
            AgentType::Summarizer => Box::new(GoalSummarizerAgent::new(cancel_token.clone(), self.memory.clone())),
            AgentType::Builder => Box::new(BuilderAgent::new(cancel_token.clone())),
            AgentType::Executor => Box::new(ExecutorAgent::new(cancel_token.clone())),
            AgentType::Optimizer => Box::new(OptimizerAgent::new(cancel_token.clone())),
        };

        // 注册任务 Handle 到 tasks
        let handle = AgentTaskHandle {
            agent_type: agent_type.clone(),
            cancel_token: cancel_token.clone(),
            status: AgentStatus::Starting,
            started_at: chrono::Utc::now(),
        };
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(workspace_id.to_string(), handle);
        }

        // 更新目标状态
        let new_status = match agent_type {
            AgentType::Summarizer => GoalStatus::Draft,
            AgentType::Builder => GoalStatus::Building,
            AgentType::Executor => GoalStatus::Building,
            AgentType::Optimizer => GoalStatus::Optimizing,
        };
        let previous_status = goal.status.clone();
        GoalsRepo::update_status(&self.db, &goal.id, new_status.clone()).await?;
        self.emitter.emit_goal_status(&goal.id, &new_status.to_string(), &previous_status.to_string());

        // 启动代理任务
        let db = self.db.clone();
        let transport = self.transport.clone();
        let llm = self.llm.clone();
        let emitter = self.emitter.clone();
        let tasks = self.tasks.clone();
        let workspace_id_tasks = workspace_id_owned.clone();

        tokio::spawn(async move {
            // 更新状态为 Running
            {
                let mut tasks_guard = tasks.write().await;
                if let Some(h) = tasks_guard.get_mut(&workspace_id_tasks) {
                    h.status = AgentStatus::Running;
                }
            }

            // 运行 Agent
            let result = agent.run(ctx, transport, llm, emitter.clone()).await;

            // 处理结果
            match result {
                Ok(output) => {
                    if let Err(e) = handle_agent_output(&db, &goal_id_owned, &output, &emitter).await {
                        tracing::error!("Failed to handle agent output: {}", e);
                        let mut tasks_guard = tasks.write().await;
                        if let Some(h) = tasks_guard.get_mut(&workspace_id_tasks) {
                            h.status = AgentStatus::Failed;
                        }
                    } else {
                        let mut tasks_guard = tasks.write().await;
                        if let Some(h) = tasks_guard.get_mut(&workspace_id_tasks) {
                            h.status = AgentStatus::Completed;
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Agent failed for workspace {} (goal {}): {}", workspace_id_tasks, goal_id_owned, e);
                    let mut tasks_guard = tasks.write().await;
                    if let Some(h) = tasks_guard.get_mut(&workspace_id_tasks) {
                        h.status = AgentStatus::Failed;
                    }
                    // Revert goal status on failure
                    let _ = GoalsRepo::update_status(&db, &goal_id_owned, GoalStatus::Failed).await;
                    emitter.emit_goal_status(&goal_id_owned, &GoalStatus::Failed.to_string(), &new_status.to_string());
                }
            }
        });

        Ok(AgentStatus::Starting)
    }

    /// Stop the agent running for a workspace
    pub async fn stop_agent(&self, workspace_id: &str) -> AppResult<AgentStatus> {
        let mut tasks = self.tasks.write().await;
        if let Some(handle) = tasks.get_mut(workspace_id) {
            handle.cancel_token.cancel();
            handle.status = AgentStatus::Canceling;
            Ok(handle.status.clone())
        } else {
            Err(AppError::NotFound(format!("No running agent for workspace: {}", workspace_id)))
        }
    }

    /// Get the status of the agent for a workspace
    pub async fn get_status(&self, workspace_id: &str) -> Option<AgentStatus> {
        let tasks = self.tasks.read().await;
        tasks.get(workspace_id).map(|h| h.status.clone())
    }

    /// Get all running agents
    pub async fn list_running(&self) -> Vec<(String, AgentType, AgentStatus)> {
        let tasks = self.tasks.read().await;
        tasks.iter()
            .filter(|(_, h)| h.status == AgentStatus::Running || h.status == AgentStatus::Starting)
            .map(|(id, h)| (id.clone(), h.agent_type.clone(), h.status.clone()))
            .collect()
    }

    /// Build the GoalContext from the database
    async fn build_goal_context(&self, goal: &crate::workhub::Goal) -> AppResult<GoalContext> {
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

        // Get WorkSpace info
        let workspace = WorkHub::get_workspace_by_goal(&self.db, &goal.id).await?;
        let (workspace_id, current_node_id) = match workspace {
            Some(ws) => (Some(ws.id), ws.current_node_id),
            None => (None, None),
        };

        Ok(GoalContext {
            goal: goal.clone(),
            current_milestone,
            conversation_history,
            available_skills,
            agent_logs,
            metadata: HashMap::new(),
            workspace_id,
            current_node_id,
            memory: self.memory.clone(),
        })
    }
}

/// Determine which agent type should run based on goal status
fn determine_agent_type(goal_status: GoalStatus) -> AppResult<AgentType> {
    match goal_status {
        GoalStatus::Draft => Ok(AgentType::Summarizer),
        GoalStatus::Pinned => Ok(AgentType::Builder),      // Builder for initial achievement
        GoalStatus::Building => Ok(AgentType::Builder),    // Resume building
        GoalStatus::Reached => Ok(AgentType::Optimizer),   // Optimize after reached
        GoalStatus::Optimizing => Ok(AgentType::Optimizer),// Resume optimization
        GoalStatus::Failed => Ok(AgentType::Builder),      // Retry with Builder
        GoalStatus::Archived => Err(AppError::Agent(format!(
            "No agent transition defined for goal status: archived"
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

            let _goal = GoalsRepo::update_summary(db, goal_id, &summary_str).await?;
            let _ = GoalsRepo::update(db, goal_id, crate::workhub::UpdateGoalInput {
                title: Some(title.clone()),
                summary: None,
                raw_input: None,
            }).await;

            // Transition back to draft so user can review and pin
            GoalsRepo::update_status(db, goal_id, GoalStatus::Draft).await?;
            emitter.emit_goal_status(goal_id, &GoalStatus::Draft.to_string(), &GoalStatus::Draft.to_string());

            // Log the agent decision
            AgentLogsRepo::append(
                db, goal_id, "summarizer", "summarizing", "completed",
                Some(&serde_json::to_string(&summary_json)?),
            ).await?;
        }

        AgentOutput::BuilderResult {
            milestone_id,
            success,
            failure_reason,
            new_node_id,
            skills_used,
            ..
        } => {
            if *success {
                // Goal reached
                GoalsRepo::update_status(db, goal_id, GoalStatus::Reached).await?;
                emitter.emit_goal_status(goal_id, &GoalStatus::Reached.to_string(), &GoalStatus::Building.to_string());

                // Update milestone status
                if !milestone_id.is_empty() {
                    MilestonesRepo::update_status(db, milestone_id, "completed").await?;
                    emitter.emit_milestone_status(milestone_id, "completed", "in_progress");
                }

                // Log the achievement
                AgentLogsRepo::append(
                    db, goal_id, "builder", "building", "reached",
                    Some(&serde_json::json!({
                        "new_node_id": new_node_id,
                        "skills_used": skills_used,
                    }).to_string()),
                ).await?;
            } else {
                GoalsRepo::update_status(db, goal_id, GoalStatus::Failed).await?;
                emitter.emit_goal_status(goal_id, &GoalStatus::Failed.to_string(), &GoalStatus::Building.to_string());

                // Log the failure
                AgentLogsRepo::append(
                    db, goal_id, "builder", "building", "failed",
                    failure_reason.as_ref().map(|s| s.as_str()),
                ).await?;
            }
        }

        AgentOutput::ExecutionResult {
            milestone_id,
            success,
            failure_reason,
            bugs_found,
            ..
        } => {
            if *success {
                GoalsRepo::update_status(db, goal_id, GoalStatus::Reached).await?;
                emitter.emit_goal_status(goal_id, &GoalStatus::Reached.to_string(), &GoalStatus::Building.to_string());
            } else {
                GoalsRepo::update_status(db, goal_id, GoalStatus::Failed).await?;
                emitter.emit_goal_status(goal_id, &GoalStatus::Failed.to_string(), &GoalStatus::Building.to_string());

                // If we have a current node, persist bugs
                if !bugs_found.is_empty() {
                    // In real implementation: use WorkHub::add_node_bug for each bug
                    tracing::info!("Execution found {} bugs", bugs_found.len());
                }
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
            optimizations,
            new_node_id,
        } => {
            if *solidified {
                GoalsRepo::update_status(db, goal_id, GoalStatus::Archived).await?;
                emitter.emit_goal_status(goal_id, &GoalStatus::Archived.to_string(), &GoalStatus::Optimizing.to_string());
            } else {
                GoalsRepo::update_status(db, goal_id, GoalStatus::Reached).await?;
                emitter.emit_goal_status(goal_id, &GoalStatus::Reached.to_string(), &GoalStatus::Optimizing.to_string());
            }

            if !milestone_id.is_empty() {
                MilestonesRepo::update_status(db, milestone_id, "completed").await?;
                emitter.emit_milestone_status(milestone_id, "completed", "in_progress");
            }

            // Log the optimization
            AgentLogsRepo::append(
                db, goal_id, "optimizer", "optimizing", "completed",
                Some(&serde_json::json!({
                    "solidified": solidified,
                    "optimizations": optimizations.len(),
                    "new_node_id": new_node_id,
                }).to_string()),
            ).await?;
        }
    }

    Ok(())
}
