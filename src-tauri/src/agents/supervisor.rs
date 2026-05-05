use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::agents::runtime::AgentRuntime;
use crate::agents::types::{AgentStatus, AgentTaskHandle};
use crate::error::{AppError, AppResult};
use crate::memory::StoredResult;
use crate::workhub::{AgentType, GoalsRepo, GoalStatus, WorkSpaceRepo};

/// AgentSupervisor — 任务分发 + 生命周期管理
///
/// 职责：
/// - `deliver_message()` — 委托 AgentRuntime 执行对话 turn
/// - `start_agent()`  — 启动后台 Agent 任务（Builder/Executor/Optimizer）
/// - `stop_agent()`   — 取消运行中的任务
/// - `get_status()`   — 查询 Agent 运行状态
/// - `list_running()` — 列出所有运行中的 Agent
///
/// 不再持有 DB、Transport、LLM、Sessions、Emitter。
/// 上下文读取和 Agent 执行由 AgentRuntime 负责。
/// 持久化编排由 Service 层负责。
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

    /// 投递一条用户消息给 Agent
    ///
    /// 根据 goal 状态确定 agent_type，委托 AgentRuntime 执行一轮 turn。
    /// 当前仅 Summarizer 使用 deliver_message 模式，
    /// 未来所有 Agent 将统一到此入口。
    pub async fn deliver_message(
        &self,
        workspace_id: &str,
        message: &str,
    ) -> AppResult<StoredResult> {
        let workspace = WorkSpaceRepo::get_by_id(self.runtime.db(), workspace_id).await?;
        let goal = GoalsRepo::get_by_id(self.runtime.db(), &workspace.goal_id).await?;
        let agent_type = determine_agent_type(goal.status.clone())?;

        match agent_type {
            AgentType::Summarizer => {
                self.runtime.run_turn(workspace_id, message).await
            }
            _ => {
                // 其他 Agent 暂走 start_agent 模式
                self.start_agent(workspace_id).await?;
                let msg = format!("{} agent started for workspace {}", agent_type, workspace_id);
                Ok(StoredResult::new(
                    workspace_id.to_string(),
                    workspace_id.to_string(),
                    goal.id,
                    agent_type,
                    serde_json::json!({ "message": msg }),
                    crate::memory::ResultStatus::Active,
                ))
            }
        }
    }

    /// 查询 workspace 的当前对话状态（从 ResultMemory 获取）
    pub async fn get_conversation_status(
        &self,
        workspace_id: &str,
    ) -> Option<crate::memory::ResultStatus> {
        self.runtime.get_result(workspace_id).await.map(|r| r.status)
    }

    // ======================================================================
    // start_agent / stop_agent / get_status — 任务式 Agent 生命周期
    // ======================================================================

    /// 为指定 workspace 启动后台 Agent 任务（Builder/Executor/Optimizer）
    pub async fn start_agent(&self, workspace_id: &str) -> AppResult<AgentStatus> {
        let workspace = WorkSpaceRepo::get_by_id(self.runtime.db(), workspace_id).await?;
        let goal = GoalsRepo::get_by_id(self.runtime.db(), &workspace.goal_id).await?;
        let agent_type = determine_agent_type(goal.status.clone())?;

        // 检查是否已有运行中的 Agent
        {
            let tasks = self.tasks.read().await;
            if let Some(handle) = tasks.get(workspace_id) {
                if handle.status == AgentStatus::Running || handle.status == AgentStatus::Starting {
                    return Ok(handle.status.clone());
                }
            }
        }

        // 注册任务 Handle
        let cancel_token = tokio_util::sync::CancellationToken::new();
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

        // 更新 goal 状态
        let new_status = match agent_type {
            AgentType::Summarizer => GoalStatus::Draft,
            AgentType::Builder => GoalStatus::Building,
            AgentType::Executor => GoalStatus::Building,
            AgentType::Optimizer => GoalStatus::Optimizing,
        };
        let _previous_status = goal.status.clone();
        GoalsRepo::update_status(self.runtime.db(), &goal.id, new_status.clone()).await?;
        // Emit via runtime's emitter (accessible via runtime's internal)
        // We'll handle event emission in the spawned task

        // 启动后台任务
        let runtime = self.runtime.clone();
        let tasks = self.tasks.clone();
        let workspace_id_owned = workspace_id.to_string();
        let goal_id_owned = goal.id.clone();

        tokio::spawn(async move {
            // 更新状态为 Running
            {
                let mut tasks_guard = tasks.write().await;
                if let Some(h) = tasks_guard.get_mut(&workspace_id_owned) {
                    h.status = AgentStatus::Running;
                }
            }

            // 通过 AgentRuntime 执行（内部已完成 handler.handle() + handler.persist()）
            let result = runtime.run_task(&workspace_id_owned).await;

            match result {
                Ok(_stored) => {
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
