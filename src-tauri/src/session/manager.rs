//! CellManager — 管理所有 SessionCell 生命周期
//!
//! 主键 cell_id → Arc<dyn SessionCell>
//! 次级索引 node_id → [cell_id]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::cell::{CellInfo, CellType, SessionCell};
use super::cell::goal_summary::GoalSummaryCell;
use super::cell::build::BuildCell;
use super::cell::executor::ExecutorCell;
use super::cell::optimizer::OptimizerCell;
use super::session::in_memory::InMemorySession;
use super::session::jsonl::{JsonlSession, JsonlSessionMeta};
use crate::config::loader::config_dir;
use crate::error::AppResult;

pub struct CellManager {
    cells: RwLock<HashMap<String, Arc<dyn SessionCell>>>,
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
    pub async fn create_cell(
        &self,
        workspace_id: &str,
        node_id: &str,
        cell_type: CellType,
    ) -> AppResult<Arc<dyn SessionCell>> {
        let cell_id = uuid::Uuid::new_v4().to_string();

        let cell: Arc<dyn SessionCell> = match cell_type {
            CellType::GoalSummary => {
                let session = Arc::new(InMemorySession::new());
                Arc::new(GoalSummaryCell::new(
                    cell_id,
                    workspace_id.to_string(),
                    node_id.to_string(),
                    session,
                ))
            }
            CellType::Build => {
                let jsonl_path = self.cell_jsonl_path(node_id, &cell_id);
                let meta = JsonlSessionMeta {
                    cell_id: cell_id.clone(),
                    workspace_id: workspace_id.to_string(),
                    node_id: node_id.to_string(),
                    cell_type: CellType::Build,
                    agent_type: crate::workhub::AgentType::Builder,
                };
                let session = Arc::new(JsonlSession::create(&jsonl_path, &meta)?);
                Arc::new(BuildCell::new(
                    cell_id,
                    workspace_id.to_string(),
                    node_id.to_string(),
                    session,
                ))
            }
            CellType::Executor => {
                let jsonl_path = self.cell_jsonl_path(node_id, &cell_id);
                let meta = JsonlSessionMeta {
                    cell_id: cell_id.clone(),
                    workspace_id: workspace_id.to_string(),
                    node_id: node_id.to_string(),
                    cell_type: CellType::Executor,
                    agent_type: crate::workhub::AgentType::Executor,
                };
                let session = Arc::new(JsonlSession::create(&jsonl_path, &meta)?);
                Arc::new(ExecutorCell::new(
                    cell_id,
                    workspace_id.to_string(),
                    node_id.to_string(),
                    session,
                ))
            }
            CellType::Optimizer => {
                let jsonl_path = self.cell_jsonl_path(node_id, &cell_id);
                let meta = JsonlSessionMeta {
                    cell_id: cell_id.clone(),
                    workspace_id: workspace_id.to_string(),
                    node_id: node_id.to_string(),
                    cell_type: CellType::Optimizer,
                    agent_type: crate::workhub::AgentType::Optimizer,
                };
                let session = Arc::new(JsonlSession::create(&jsonl_path, &meta)?);
                Arc::new(OptimizerCell::new(
                    cell_id,
                    workspace_id.to_string(),
                    node_id.to_string(),
                    session,
                ))
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

    /// 启动时从磁盘加载所有 JsonlSession 对应的 Cell
    pub async fn load_all_cells(&self) -> AppResult<()> {
        if !self.cells_dir.exists() {
            return Ok(());
        }

        for node_entry in std::fs::read_dir(&self.cells_dir)? {
            let node_entry = node_entry?;
            if !node_entry.file_type()?.is_dir() {
                continue;
            }
            let node_id = node_entry.file_name().to_string_lossy().to_string();
            let cells_subdir = node_entry.path().join("cells");

            if !cells_subdir.exists() {
                continue;
            }

            for cell_entry in std::fs::read_dir(&cells_subdir)? {
                let cell_entry = cell_entry?;
                if !cell_entry.file_type()?.is_file() {
                    continue;
                }
                let file_name = cell_entry.file_name().to_string_lossy().to_string();
                if !file_name.ends_with(".jsonl") {
                    continue;
                }
                let cell_id = file_name.trim_end_matches(".jsonl").to_string();
                let jsonl_path = cell_entry.path();

                match JsonlSession::load(&jsonl_path) {
                    Ok((session, meta)) => {
                        let session: Arc<dyn super::session::Session> = Arc::new(session);
                        let cell: Arc<dyn SessionCell> = match meta.cell_type {
                            CellType::Build => Arc::new(BuildCell::new(
                                meta.cell_id,
                                meta.workspace_id,
                                meta.node_id,
                                session,
                            )),
                            CellType::Executor => Arc::new(ExecutorCell::new(
                                meta.cell_id,
                                meta.workspace_id,
                                meta.node_id,
                                session,
                            )),
                            CellType::Optimizer => Arc::new(OptimizerCell::new(
                                meta.cell_id,
                                meta.workspace_id,
                                meta.node_id,
                                session,
                            )),
                            CellType::GoalSummary => {
                                tracing::warn!(
                                    "GoalSummary cell found on disk (should be InMemory) {}: skipping",
                                    cell_id
                                );
                                continue;
                            }
                        };
                        self.cells
                            .write()
                            .await
                            .insert(cell_id.clone(), cell);
                        self.node_index
                            .write()
                            .await
                            .entry(node_id.clone())
                            .or_default()
                            .push(cell_id);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load cell {}: {}", cell_id, e);
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

    // helpers

    fn cell_jsonl_path(&self, node_id: &str, cell_id: &str) -> PathBuf {
        self.cells_dir
            .join(node_id)
            .join("cells")
            .join(format!("{}.jsonl", cell_id))
    }
}
