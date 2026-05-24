use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use crate::agents::runtime::AgentRuntime;
use crate::agents::types::{AgentStatus, AgentTaskHandle};
use crate::error::{AppError, AppResult};
use crate::session::EffectType;
use crate::workhub::{AgentType, GoalsRepo, GoalStatus, WorkSpaceRepo};

/// AgentSupervisor — 任务分发 + 生命周期管理
///
/// 职责：
/// - `deliver_message(cell_id, msg)` — 委托 AgentRuntime 执行对话 turn
/// - `start_agent(cell_id)` — 启动后台 Agent 任务
/// - `stop_agent(workspace_id)` / `get_status(workspace_id)` — 任务状态管理
pub struct AgentSupervisor {
    runtime: Arc<AgentRuntime>,
    tasks: Arc<RwLock<HashMap<String, AgentTaskHandle>>>,
}

impl AgentSupervisor {
    pub fn new(runtime: Arc<AgentRuntime>) -> Self {
        Self {
            runtime,
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 提供 Runtime 访问（供 app_state 使用）
    pub fn runtime(&self) -> &Arc<AgentRuntime> {
        &self.runtime
    }

    // ======================================================================
    // deliver_message — 对话式入口
    // ======================================================================

    /// 投递一条用户消息到指定 Cell 的 Agent
    pub async fn deliver_message(
        &self,
        cell_id: &str,
        message: &str,
    ) -> AppResult<()> {
        let cell = self.runtime.cell_manager().get_cell(cell_id).await.ok_or_else(|| {
            AppError::NotFound(format!("Cell not found: {}", cell_id))
        })?;

        let workspace = WorkSpaceRepo::get_by_id(self.runtime.db(), cell.workspace_id()).await?;
        let goal = GoalsRepo::get_by_id(self.runtime.db(), &workspace.goal_id).await?;
        let agent_type = determine_agent_type(goal.status.clone())?;

        match agent_type {
            AgentType::Summarizer => {
                self.runtime.run_turn(cell_id, message).await
            }
            _ => {
                self.start_agent(cell_id).await?;
                // 记录一个简单的 started effect
                cell.add_effect(
                    EffectType::Result,
                    serde_json::json!({ "message": format!("{} agent started", agent_type) }),
                ).await;
                Ok(())
            }
        }
    }

    /// 查询 Cell 的当前结果状态
    pub async fn get_cell_result_status(&self, cell_id: &str) -> Option<String> {
        self.runtime.cell_manager()
            .get_cell(cell_id).await
            .and_then(|c| {
                c.latest_effect(EffectType::Result)
                    .and_then(|l| {
                        if let crate::session::SessionLine::Effect { content, .. } = l {
                            content.get("type").and_then(|v| v.as_str()).map(String::from)
                        } else {
                            None
                        }
                    })
            })
    }

    // ======================================================================
    // start_agent / stop_agent / get_status
    // ======================================================================

    /// 为指定 Cell 启动后台 Agent 任务
    pub async fn start_agent(&self, cell_id: &str) -> AppResult<AgentStatus> {
        let cell = self.runtime.cell_manager().get_cell(cell_id).await.ok_or_else(|| {
            AppError::NotFound(format!("Cell not found: {}", cell_id))
        })?;

        let workspace = WorkSpaceRepo::get_by_id(self.runtime.db(), cell.workspace_id()).await?;
        let goal = GoalsRepo::get_by_id(self.runtime.db(), &workspace.goal_id).await?;
        let agent_type = determine_agent_type(goal.status.clone())?;

        // 检查是否已有运行中的 Agent（按 workspace 检查）
        {
            let tasks = self.tasks.read().await;
            if let Some(handle) = tasks.get(cell.workspace_id()) {
                if handle.status == AgentStatus::Running || handle.status == AgentStatus::Starting {
                    return Ok(handle.status.clone());
                }
            }
        }

        let cancel_token = CancellationToken::new();
        let handle = AgentTaskHandle {
            agent_type: agent_type.clone(),
            cancel_token: cancel_token.clone(),
            status: AgentStatus::Starting,
            started_at: chrono::Utc::now(),
        };
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(cell.workspace_id().to_string(), handle);
        }

        // 更新 goal 状态
        let new_status = match agent_type {
            AgentType::Summarizer => GoalStatus::Draft,
            AgentType::Builder => GoalStatus::Building,
            AgentType::Executor => GoalStatus::Building,
            AgentType::Optimizer => GoalStatus::Optimizing,
        };
        GoalsRepo::update_status(self.runtime.db(), &goal.id, new_status.clone()).await?;
        
        // 启动后台任务
        let runtime = self.runtime.clone();
        let tasks = self.tasks.clone();
        let workspace_id_owned = cell.workspace_id().to_string();
        let cell_id_owned = cell_id.to_string();
        let goal_id_owned = goal.id.clone();

        tokio::spawn(async move {
            {
                let mut tasks_guard = tasks.write().await;
                if let Some(h) = tasks_guard.get_mut(&workspace_id_owned) {
                    h.status = AgentStatus::Running;
                }
            }

            let result = runtime.run_task(&cell_id_owned).await;

            match result {
                Ok(()) => {
                    let mut tasks_guard = tasks.write().await;
                    if let Some(h) = tasks_guard.get_mut(&workspace_id_owned) {
                        h.status = AgentStatus::Completed;
                    }
                }
                Err(e) => {
                    tracing::error!(
                        "Agent failed for workspace {} (goal {}): {}",
                        workspace_id_owned, goal_id_owned, e
                    );
                    let mut tasks_guard = tasks.write().await;
                    if let Some(h) = tasks_guard.get_mut(&workspace_id_owned) {
                        h.status = AgentStatus::Failed;
                    }
                    let _ = GoalsRepo::update_status(
                        runtime.db(),
                        &goal_id_owned,
                        GoalStatus::Failed,
                    ).await;
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
            Err(AppError::NotFound(format!(
                "No running agent for workspace: {}",
                workspace_id
            )))
        }
    }

    /// 查询 Agent 运行状态
    pub async fn get_status(&self, workspace_id: &str) -> Option<AgentStatus> {
        let tasks = self.tasks.read().await;
        tasks.get(workspace_id).map(|h| h.status.clone())
    }

    /// 列出所有运行中的 Agent
    pub async fn list_running(&self) -> Vec<(String, AgentType, AgentStatus)> {
        let tasks = self.tasks.read().await;
        tasks
            .iter()
            .filter(|(_, h)| {
                h.status == AgentStatus::Running || h.status == AgentStatus::Starting
            })
            .map(|(id, h)| (id.clone(), h.agent_type.clone(), h.status.clone()))
            .collect()
    }
}

/// 确定 goal 状态对应的 agent 类型
fn determine_agent_type(goal_status: GoalStatus) -> AppResult<AgentType> {
    match goal_status {
        GoalStatus::Draft => Ok(AgentType::Summarizer),
        GoalStatus::Pinned => Ok(AgentType::Builder),
        GoalStatus::Building => Ok(AgentType::Builder),
        GoalStatus::Reached => Ok(AgentType::Optimizer),
        GoalStatus::Optimizing => Ok(AgentType::Optimizer),
        GoalStatus::Failed => Ok(AgentType::Builder),
        GoalStatus::Archived => Err(AppError::Agent(format!(
            "No agent transition defined for goal status: archived"
        ))),
    }
}
