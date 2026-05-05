//! CellManager — 替代 SessionManager，管理所有 SessionCell 生命周期
//!
//! 主键 cell_id → Arc<dyn SessionCell>
//! 次级索引 node_id → [cell_id]（用于 list_cells_by_node）

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{CellInfo, SessionCell};
use super::in_memory::InMemoryCell;
use super::jsonl_cell::JsonlCell;
use crate::config::loader::config_dir;
use crate::error::AppResult;
use crate::workhub::AgentType;

pub struct CellManager {
    cells: RwLock<HashMap<String, Arc<dyn SessionCell>>>,
    /// node_id → [cell_id] 索引
    node_index: RwLock<HashMap<String, Vec<String>>>,
    cells_dir: PathBuf,
}

impl CellManager {
    pub fn new() -> Self {
        let cells_dir = config_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("nodes");

        Self {
            cells: RwLock::new(HashMap::new()),
            node_index: RwLock::new(HashMap::new()),
            cells_dir,
        }
    }

    /// 创建新 Cell
    ///
    /// Summarizer → InMemoryCell（不持久化）
    /// Builder/Executor/Optimizer → JsonlCell（JSONL 持久化）
    pub async fn create_cell(
        &self,
        workspace_id: &str,
        node_id: &str,
        agent_type: AgentType,
    ) -> AppResult<Arc<dyn SessionCell>> {
        let cell_id = uuid::Uuid::new_v4().to_string();

        let cell: Arc<dyn SessionCell> = match agent_type {
            AgentType::Summarizer => {
                Arc::new(InMemoryCell::new(
                    cell_id,
                    workspace_id.to_string(),
                    node_id.to_string(),
                    agent_type,
                ))
            }
            _ => {
                Arc::new(JsonlCell::create(
                    cell_id,
                    workspace_id.to_string(),
                    node_id.to_string(),
                    agent_type,
                    &self.cells_dir,
                )?)
            }
        };

        // 注册索引
        {
            let mut cells = self.cells.write().await;
            cells.insert(cell.cell_id().to_string(), cell.clone());
        }
        {
            let mut node_index = self.node_index.write().await;
            node_index
                .entry(node_id.to_string())
                .or_default()
                .push(cell.cell_id().to_string());
        }

        Ok(cell)
    }

    /// 查询 Cell
    pub async fn get_cell(&self, cell_id: &str) -> Option<Arc<dyn SessionCell>> {
        self.cells.read().await.get(cell_id).cloned()
    }

    /// 列出某 WorkNode 下所有 cells
    pub async fn list_cells_by_node(&self, node_id: &str) -> Vec<CellInfo> {
        let node_index = self.node_index.read().await;
        let cells = self.cells.read().await;

        node_index
            .get(node_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| cells.get(id))
                    .map(|c| c.to_info())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 关闭 Cell
    pub async fn close_cell(&self, cell_id: &str) -> AppResult<()> {
        if let Some(cell) = self.cells.read().await.get(cell_id) {
            cell.close().await;
        }
        Ok(())
    }

    /// 启动时从磁盘加载所有 JsonlCell
    pub async fn load_all_cells(&self) -> AppResult<()> {
        if !self.cells_dir.exists() {
            return Ok(());
        }

        // 遍历 nodes/{node_id}/cells/{cell_id}/session.jsonl
        for node_entry in std::fs::read_dir(&self.cells_dir)? {
            let node_entry = node_entry?;
            if !node_entry.file_type()?.is_dir() { continue; }
            let _node_id = node_entry.file_name().to_string_lossy().to_string();
            let cells_subdir = node_entry.path().join("cells");

            if !cells_subdir.exists() { continue; }

            for cell_entry in std::fs::read_dir(&cells_subdir)? {
                let cell_entry = cell_entry?;
                if !cell_entry.file_type()?.is_dir() { continue; }
                let cell_id = cell_entry.file_name().to_string_lossy().to_string();
                let jsonl_path = cell_entry.path().join("session.jsonl");

                if jsonl_path.exists() {
                    match JsonlCell::load(&cell_id, &jsonl_path) {
                        Ok(cell) => {
                            let node = cell.node_id().to_string();
                            let cell: Arc<dyn SessionCell> = Arc::new(cell);
                            self.cells.write().await.insert(cell_id.clone(), cell);
                            self.node_index.write().await
                                .entry(node)
                                .or_default()
                                .push(cell_id);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to load cell {}: {}", cell_id, e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 目录路径
    pub fn cells_dir(&self) -> &PathBuf {
        &self.cells_dir
    }
}
