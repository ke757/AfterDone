//! InMemorySession — 纯内存会话实现
//!
//! 不持久化，App 重启后丢失。用于 Summarizer 等临时对话场景。

use async_trait::async_trait;
use tokio::sync::RwLock;

use super::Session;
use crate::session::SessionLine;

pub struct InMemorySession {
    lines: RwLock<Vec<SessionLine>>,
}

impl InMemorySession {
    pub fn new() -> Self {
        Self {
            lines: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl Session for InMemorySession {
    async fn add_line(&self, line: SessionLine) {
        self.lines.write().await.push(line);
    }

    fn get_lines(&self) -> Vec<SessionLine> {
        self.lines.blocking_read().clone()
    }

    fn line_count(&self) -> usize {
        self.lines.blocking_read().len()
    }

    async fn truncate(&self, at_index: usize) {
        let mut lines = self.lines.write().await;
        if at_index < lines.len() {
            lines.truncate(at_index);
        }
    }

    async fn clear(&self) {
        self.lines.write().await.clear();
    }
}
