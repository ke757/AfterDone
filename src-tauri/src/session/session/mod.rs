//! Session trait — 会话消息的存取抽象
//!
//! 提供统一的 add_line / get_lines / truncate / clear 接口，
//! 由 InMemorySession 或 JsonlSession 实现不同的持久化策略。

pub mod in_memory;
pub mod jsonl;

use async_trait::async_trait;
use crate::session::{EffectType, SessionLine};

#[async_trait]
pub trait Session: Send + Sync {
    async fn add_line(&self, line: SessionLine);
    fn get_lines(&self) -> Vec<SessionLine>;
    fn line_count(&self) -> usize;
    async fn truncate(&self, at_index: usize);
    async fn clear(&self);

    fn get_messages(&self) -> Vec<SessionLine> {
        self.get_lines()
            .into_iter()
            .filter(|l| matches!(l, SessionLine::Message { .. }))
            .collect()
    }

    fn get_effects(&self) -> Vec<SessionLine> {
        self.get_lines()
            .into_iter()
            .filter(|l| matches!(l, SessionLine::Effect { .. }))
            .collect()
    }

    fn latest_effect(&self, effect_type: EffectType) -> Option<SessionLine> {
        self.get_lines()
            .into_iter()
            .rev()
            .find(|l| matches!(l, SessionLine::Effect { effect_type: et, .. } if *et == effect_type))
    }
}
