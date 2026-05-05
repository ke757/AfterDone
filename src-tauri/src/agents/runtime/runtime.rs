use std::collections::HashMap;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

use crate::agents::traits::SideCarAgent;
use crate::agents::types::{AgentOutput, RuntimeContext};
use crate::agents::{
    GoalSummarizerAgent, BuilderAgent, ExecutorAgent, OptimizerAgent,
};
use crate::agents::runtime::handlers::{
    HandlerContext, ResultHandlerRegistry,
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
use crate::memory::{ResultMemory, StoredResult};
use crate::session::SessionManager;
use crate::workhub::{
    AgentType, GoalsRepo, MilestonesRepo, SkillsRepo, AgentLogsRepo,
    Goal, GoalStatus, WorkSpaceRepo, WorkHub,
};

/// AgentRuntime — Agent 执行环境
///
/// 职责：
/// - 上下文读取：从 DB 构建 RuntimeContext
/// - Agent 单次 turn 运行：创建 Agent 实例 → 调用 run()
/// - 结果收集：通过 ResultHandler 转换 → 存入 ResultMemory
///
/// Supervisor 通过最小接口 (run_turn / run_task / get_result) 调用。
pub struct AgentRuntime {
    db: DatabasePool,
    transport: Arc<dyn Transport>,
    llm: Arc<dyn LlmProvider>,
    emitter: EventBridge,
    sessions: Arc<SessionManager>,
    handlers: ResultHandlerRegistry,
    result_memory: Arc<ResultMemory>,
}

impl AgentRuntime {
    pub fn new(
        db: DatabasePool,
        transport: Arc<dyn Transport>,
        llm: Arc<dyn LlmProvider>,
        emitter: EventBridge,
        sessions: Arc<SessionManager>,
    ) -> Self {
        let result_memory = Arc::new(ResultMemory::new());

        let mut handlers = ResultHandlerRegistry::new();
        handlers.register(Box::new(SummarizerResultHandler));
        handlers.register(Box::new(BuilderResultHandler));
        handlers.register(Box::new(ExecutorResultHandler));
        handlers.register(Box::new(OptimizerResultHandler));

        Self {
            db,
            transport,
            llm,
            emitter,
            sessions,
            handlers,
            result_memory,
        }
    }

    // ======================================================================
    // 对外接口
    // ======================================================================

    /// 执行一轮对话 turn（用于 deliver_message 模式）
    ///
    /// 加载上下文 → 追加 user 消息 → agent.run() → handler → store → 返回
    pub async fn run_turn(
        &self,
        workspace_id: &str,
        message: &str,
    ) -> AppResult<StoredResult> {
        let (workspace, goal) = self.load_workspace_goal(workspace_id).await?;
        let agent_type = determine_agent_type(goal.status.clone())?;

        let ctx: RuntimeContext = self.build_context(&goal).await?;
        let session_id = workspace_id.to_string();

        // 追加 user 消息到 session
        ctx.session.add_message("user", message, &agent_type.to_string()).await;

        // 创建 agent 实例
        let cancel_token = CancellationToken::new();
        let agent = self.create_agent(agent_type.clone(), cancel_token);

        // 运行
        let output: AgentOutput = agent.run(
            ctx,
            self.transport.clone(),
            self.llm.clone(),
            self.emitter.clone(),
        ).await?;

        // 结果处理 → 存储
        let stored = self.handle_and_store(
            output,
            agent_type,
            &session_id,
            &workspace.id,
            &goal.id,
        ).await?;

        Ok(stored)
    }

    /// 执行一次完整 Agent 任务（用于 start_agent 的 spawned task）
    ///
    /// 与 run_turn 的区别：不追加 user 消息，适用于 Builder/Executor/Optimizer
    /// 这些 Agent 的任务由 goal context 驱动而非用户对话。
    ///
    /// 完整流程：context → agent.run() → handler.handle() → store Memory → handler.persist() → DB
    pub async fn run_task(&self, workspace_id: &str) -> AppResult<StoredResult> {
        let (workspace, goal) = self.load_workspace_goal(workspace_id).await?;
        let agent_type = determine_agent_type(goal.status.clone())?;

        let ctx = self.build_context(&goal).await?;
        let session_id = workspace_id.to_string();

        let cancel_token = CancellationToken::new();
        let agent = self.create_agent(agent_type.clone(), cancel_token);

        let output = agent.run(
            ctx,
            self.transport.clone(),
            self.llm.clone(),
            self.emitter.clone(),
        ).await?;

        let stored = self.handle_and_store(
            output,
            agent_type.clone(),
            &session_id,
            &workspace.id,
            &goal.id,
        ).await?;

        // 持久化到 DB（内置能力，不依赖 Services 层）
        let handler = self.handlers.get(&agent_type).ok_or_else(|| {
            AppError::Agent(format!("No handler registered for agent type: {:?}", agent_type))
        })?;
        handler.persist(&self.db, &stored).await?;

        Ok(stored)
    }

    /// 从 ResultMemory 查询指定 session 的最新结果
    pub async fn get_result(&self, session_id: &str) -> Option<StoredResult> {
        self.result_memory.get_latest(session_id).await
    }

    /// DB pool 访问（供 Supervisor 持久化使用）
    pub fn db(&self) -> &DatabasePool {
        &self.db
    }

    /// ResultMemory 访问
    pub fn result_memory(&self) -> &Arc<ResultMemory> {
        &self.result_memory
    }

    // ======================================================================
    // 内部方法
    // ======================================================================

    /// 加载 workspace 和关联的 goal
    async fn load_workspace_goal(&self, workspace_id: &str) -> AppResult<(crate::workhub::WorkSpace, Goal)> {
        let workspace = WorkSpaceRepo::get_by_id(&self.db, workspace_id).await?;
        let goal = GoalsRepo::get_by_id(&self.db, &workspace.goal_id).await?;
        Ok((workspace, goal))
    }

    /// 创建 Agent 实例
    fn create_agent(&self, agent_type: AgentType, cancel_token: CancellationToken) -> Box<dyn SideCarAgent> {
        match agent_type {
            AgentType::Summarizer => Box::new(GoalSummarizerAgent::new(cancel_token)),
            AgentType::Builder => Box::new(BuilderAgent::new(cancel_token)),
            AgentType::Executor => Box::new(ExecutorAgent::new(cancel_token)),
            AgentType::Optimizer => Box::new(OptimizerAgent::new(cancel_token)),
        }
    }

    /// Handler 处理 + 存储
    async fn handle_and_store(
        &self,
        output: AgentOutput,
        agent_type: AgentType,
        session_id: &str,
        workspace_id: &str,
        goal_id: &str,
    ) -> AppResult<StoredResult> {
        let handler = self.handlers.get(&agent_type).ok_or_else(|| {
            AppError::Agent(format!("No handler registered for agent type: {:?}", agent_type))
        })?;

        let stored = handler.handle(output, &HandlerContext {
            session_id: session_id.to_string(),
            workspace_id: workspace_id.to_string(),
            goal_id: goal_id.to_string(),
        }).await?;

        self.result_memory.store(stored.clone()).await;

        Ok(stored)
    }

    /// 构建 RuntimeContext（从 Supervisor 迁移）
    pub(crate) async fn build_context(&self, goal: &Goal) -> AppResult<RuntimeContext> {
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

        let session = if let Some(ref node_id) = current_node_id {
            self.sessions.get_or_create_node(node_id)?
        } else {
            self.sessions.get_or_create_temp(workspace_id.as_deref().unwrap_or(&goal.id)).await
        };

        Ok(RuntimeContext {
            goal: goal.clone(),
            current_milestone,
            available_skills,
            agent_logs,
            metadata: HashMap::new(),
            workspace_id,
            current_node_id,
            session,
        })
    }
}

/// 根据 goal 状态确定 agent 类型
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
