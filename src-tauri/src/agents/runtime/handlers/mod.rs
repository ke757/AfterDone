//! ResultHandler trait + registry
//!
//! 每个 Agent 类型对应一个 ResultHandler 实现，负责：
//! - to_effect_content()：将 AgentOutput 转为 effect payload（serde_json::Value）
//! - persist()：将 AgentOutput 持久化到数据库

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

/// ResultHandler trait — AgentOutput → effect content + DB 持久化
#[async_trait]
pub trait ResultHandler: Send + Sync {
    fn agent_type(&self) -> AgentType;

    /// 将 AgentOutput 转为 effect content (serde_json::Value)
    fn to_effect_content(&self, output: &AgentOutput) -> AppResult<serde_json::Value>;

    /// 将 AgentOutput 持久化到数据库（goal 状态更新、agent log）
    async fn persist(
        &self,
        db: &DatabasePool,
        output: &AgentOutput,
        goal_id: &str,
        workspace_id: &str,
    ) -> AppResult<()>;
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
