//! ResultHandler trait + registry
//!
//! 每个 Agent 类型对应一个 ResultHandler 实现，负责：
//! - handle()：将 AgentOutput 转换为统一的 StoredResult → 存入 ResultMemory
//! - persist()：将 StoredResult 持久化到数据库

pub mod summarizer;
pub mod builder;
pub mod executor;
pub mod optimizer;

use std::collections::HashMap;
use async_trait::async_trait;

use crate::workhub::AgentType;
use crate::agents::AgentOutput;
use crate::db::DatabasePool;
use crate::error::AppResult;
use crate::session::cell::result::StoredResult;

/// Handler 需要的上下文信息
#[derive(Debug, Clone)]
pub struct HandlerContext {
    pub session_id: String,
    pub workspace_id: String,
    pub goal_id: String,
}

/// ResultHandler trait — AgentOutput → StoredResult 转换 + DB 持久化
#[async_trait]
pub trait ResultHandler: Send + Sync {
    fn agent_type(&self) -> AgentType;

    /// 将 AgentOutput 转换为 StoredResult
    async fn handle(&self, output: AgentOutput, ctx: &HandlerContext) -> AppResult<StoredResult>;

    /// 将 StoredResult 持久化到数据库
    ///
    /// 从 stored.output 反序列化 AgentOutput，根据 agent_type 执行对应的 DB 操作。
    async fn persist(&self, db: &DatabasePool, stored: &StoredResult) -> AppResult<()>;
}

/// ResultHandler 注册表 — 按 AgentType 查找对应的 Handler
pub struct ResultHandlerRegistry {
    handlers: HashMap<AgentType, Box<dyn ResultHandler>>,
}

impl ResultHandlerRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register(&mut self, handler: Box<dyn ResultHandler>) {
        self.handlers.insert(handler.agent_type(), handler);
    }

    pub fn get(&self, agent_type: &AgentType) -> Option<&Box<dyn ResultHandler>> {
        self.handlers.get(agent_type)
    }
}
