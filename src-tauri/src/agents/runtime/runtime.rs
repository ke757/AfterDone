use std::collections::HashMap;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

use crate::agents::traits::SideCarAgent;
use crate::agents::types::{AgentOutput, RuntimeContext};
use crate::agents::{
    GoalSummarizerAgent, BuilderAgent, ExecutorAgent, OptimizerAgent,
};
use crate::agents::runtime::handlers::{
    HandlerContext,
    summarizer::SummarizerResultHandler,
    builder::BuilderResultHandler,
    executor::ExecutorResultHandler,
    optimizer::OptimizerResultHandler,
};
use crate::adapter::Transport;
use crate::db::DatabasePool;
use crate::error::{AppError, AppResult};
use crate::events::EventBridge;
use crate::llm::LlmProvider;
use crate::session::cell::{
    manager::CellManager, SessionCell,
    result::{StoredResult},
};
use crate::workhub::{
    AgentType, GoalsRepo, MilestonesRepo, SkillsRepo, AgentLogsRepo,
    Goal, GoalStatus, WorkSpaceRepo, WorkHub,
};

/// AgentRuntime — Agent 执行环境
///
/// 职责：
/// - 上下文读取：从 DB + Cell 构建 RuntimeContext
/// - Agent 单次 turn 运行：创建 Agent 实例 → 调用 run()
/// - 结果收集：通过 ResultHandler 转换 → 存入 Cell
pub struct AgentRuntime {
    db: DatabasePool,
    transport: Arc<dyn Transport>,
    llm: Arc<dyn LlmProvider>,
    emitter: EventBridge,
    cell_manager: Arc<CellManager>,
    handlers: super::handlers::ResultHandlerRegistry,
}

impl AgentRuntime {
    pub fn new(
        db: DatabasePool,
        transport: Arc<dyn Transport>,
        llm: Arc<dyn LlmProvider>,
        emitter: EventBridge,
        cell_manager: Arc<CellManager>,
    ) -> Self {
        let mut handlers = super::handlers::ResultHandlerRegistry::new();
        handlers.register(Box::new(SummarizerResultHandler));
        handlers.register(Box::new(BuilderResultHandler));
        handlers.register(Box::new(ExecutorResultHandler));
        handlers.register(Box::new(OptimizerResultHandler));

        Self {
            db,
            transport,
            llm,
            emitter,
            cell_manager,
            handlers,
        }
    }

    // ======================================================================
    // 对外接口
    // ======================================================================

    /// 执行一轮对话 turn（用于 deliver_message 模式）
    ///
    /// cell_id → 获取 Cell → 追加 user 消息 → agent.run() → handler.handle() → cell.set_result()
    pub async fn run_turn(
        &self,
        cell_id: &str,
        message: &str,
    ) -> AppResult<StoredResult> {
        let cell = self.cell_manager.get_cell(cell_id).await.ok_or_else(|| {
            AppError::NotFound(format!("Cell not found: {}", cell_id))
        })?;

        let workspace_id = cell.workspace_id().to_string();
        let (_, goal) = self.load_workspace_goal(&workspace_id).await?;
        let agent_type = determine_agent_type(goal.status.clone())?;

        let ctx = self.build_context(&goal, cell.clone()).await?;

        // 追加 user 消息到 cell
        cell.add_message("user", message).await;

        let cancel_token = CancellationToken::new();
        let agent = self.create_agent(agent_type.clone(), cancel_token);

        let output = agent.run(
            ctx,
            self.transport.clone(),
            self.llm.clone(),
            self.emitter.clone(),
        ).await?;

        let stored = self.handle_and_store(output, agent_type, &cell, &goal.id).await?;
        Ok(stored)
    }

    /// 执行一次完整 Agent 任务（用于 start_agent 的 spawned task）
    ///
    /// 完整流程：context → agent.run() → handler.handle() → cell.set_result() → handler.persist()
    pub async fn run_task(&self, cell_id: &str) -> AppResult<StoredResult> {
        let cell = self.cell_manager.get_cell(cell_id).await.ok_or_else(|| {
            AppError::NotFound(format!("Cell not found: {}", cell_id))
        })?;

        let workspace_id = cell.workspace_id().to_string();
        let (_, goal) = self.load_workspace_goal(&workspace_id).await?;
        let agent_type = determine_agent_type(goal.status.clone())?;

        let ctx = self.build_context(&goal, cell.clone()).await?;

        let cancel_token = CancellationToken::new();
        let agent = self.create_agent(agent_type.clone(), cancel_token);

        let output = agent.run(
            ctx,
            self.transport.clone(),
            self.llm.clone(),
            self.emitter.clone(),
        ).await?;

        let stored = self.handle_and_store(output, agent_type.clone(), &cell, &goal.id).await?;

        // 持久化到 DB
        let handler = self.handlers.get(&agent_type).ok_or_else(|| {
            AppError::Agent(format!("No handler registered for agent type: {:?}", agent_type))
        })?;
        handler.persist(&self.db, &stored).await?;

        Ok(stored)
    }

    /// CellManager 访问
    pub fn cell_manager(&self) -> &Arc<CellManager> {
        &self.cell_manager
    }

    /// DB pool 访问
    pub fn db(&self) -> &DatabasePool {
        &self.db
    }

    // ======================================================================
    // 内部方法
    // ======================================================================

    async fn load_workspace_goal(&self, workspace_id: &str) -> AppResult<(crate::workhub::WorkSpace, Goal)> {
        let workspace = WorkSpaceRepo::get_by_id(&self.db, workspace_id).await?;
        let goal = GoalsRepo::get_by_id(&self.db, &workspace.goal_id).await?;
        Ok((workspace, goal))
    }

    fn create_agent(&self, agent_type: AgentType, cancel_token: CancellationToken) -> Box<dyn SideCarAgent> {
        match agent_type {
            AgentType::Summarizer => Box::new(GoalSummarizerAgent::new(cancel_token)),
            AgentType::Builder => Box::new(BuilderAgent::new(cancel_token)),
            AgentType::Executor => Box::new(ExecutorAgent::new(cancel_token)),
            AgentType::Optimizer => Box::new(OptimizerAgent::new(cancel_token)),
        }
    }

    /// Handler 处理 + 存储到 Cell
    async fn handle_and_store(
        &self,
        output: AgentOutput,
        agent_type: AgentType,
        cell: &Arc<dyn SessionCell>,
        goal_id: &str,
    ) -> AppResult<StoredResult> {
        let handler = self.handlers.get(&agent_type).ok_or_else(|| {
            AppError::Agent(format!("No handler registered for agent type: {:?}", agent_type))
        })?;

        let stored = handler.handle(output, &HandlerContext {
            session_id: cell.cell_id().to_string(),
            workspace_id: cell.workspace_id().to_string(),
            goal_id: goal_id.to_string(),
        }).await?;

        cell.set_result(stored.clone()).await;

        Ok(stored)
    }

    /// 构建 RuntimeContext
    pub(crate) async fn build_context(
        &self,
        goal: &Goal,
        cell: Arc<dyn SessionCell>,
    ) -> AppResult<RuntimeContext> {
        let current_milestone = match &goal.current_milestone_id {
            Some(mid) => Some(MilestonesRepo::get_by_id(&self.db, mid).await?),
            None => None,
        };

        let available_skills = match &current_milestone {
            Some(m) => SkillsRepo::list_by_milestone(&self.db, &m.id).await?,
            None => vec![],
        };

        let agent_logs = AgentLogsRepo::list_by_goal(&self.db, &goal.id).await?;

        let workspace = WorkHub::get_workspace_by_goal(&self.db, &goal.id).await?;
        let (workspace_id, current_node_id) = match workspace {
            Some(ws) => (Some(ws.id), ws.current_node_id),
            None => (None, None),
        };

        Ok(RuntimeContext {
            goal: goal.clone(),
            current_milestone,
            available_skills,
            agent_logs,
            metadata: HashMap::new(),
            workspace_id,
            current_node_id,
            cell,
        })
    }
}

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
