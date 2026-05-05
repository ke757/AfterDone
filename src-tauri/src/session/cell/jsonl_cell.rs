//! JsonlCell — 持久化 Cell（Builder/Executor/Optimizer 专用）
//!
//! JSONL 文件格式：第一行 cell 元数据，后续行为 session messages。
//! 路径：nodes/{node_id}/cells/{cell_id}.jsonl

use std::path::{Path, PathBuf};
use tokio::sync::{Mutex, RwLock};
use async_trait::async_trait;

use super::{CellStatus, SessionCell};
use super::result::StoredResult;
use crate::session::Message;
use crate::error::AppResult;
use crate::workhub::AgentType;

/// JSONL 首行元数据格式
#[derive(serde::Serialize, serde::Deserialize)]
struct CellMeta {
    #[serde(rename = "type")]
    record_type: String,       // "cell_meta"
    cell_id: String,
    workspace_id: String,
    node_id: String,
    agent_type: String,
    created_at: String,
}

pub struct JsonlCell {
    cell_id: String,
    workspace_id: String,
    node_id: String,
    agent_type: AgentType,
    jsonl_path: PathBuf,
    status: RwLock<CellStatus>,
    messages: RwLock<Vec<Message>>,
    result: RwLock<Option<StoredResult>>,
    file_lock: Mutex<()>,
}

impl JsonlCell {
    /// 创建新的 JsonlCell 并写入元数据首行
    pub fn create(
        cell_id: String,
        workspace_id: String,
        node_id: String,
        agent_type: AgentType,
        cells_dir: &Path,
    ) -> AppResult<Self> {
        let cell_dir = cells_dir.join(&cell_id);
        std::fs::create_dir_all(&cell_dir)?;
        let jsonl_path = cell_dir.join("session.jsonl");

        let meta = CellMeta {
            record_type: "cell_meta".to_string(),
            cell_id: cell_id.clone(),
            workspace_id: workspace_id.clone(),
            node_id: node_id.clone(),
            agent_type: agent_type.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        let meta_line = serde_json::to_string(&meta)? + "\n";
        std::fs::write(&jsonl_path, meta_line)?;

        Ok(Self {
            cell_id,
            workspace_id,
            node_id,
            agent_type,
            jsonl_path,
            status: RwLock::new(CellStatus::Active),
            messages: RwLock::new(Vec::new()),
            result: RwLock::new(None),
            file_lock: Mutex::new(()),
        })
    }

    /// 从已有 JSONL 文件加载 JsonlCell
    pub fn load(cell_id: &str, jsonl_path: &Path) -> AppResult<Self> {
        let content = std::fs::read_to_string(jsonl_path)?;
        let mut messages = Vec::new();
        let mut workspace_id = String::new();
        let mut node_id = String::new();
        let mut agent_type_str = String::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }
            if let Ok(meta) = serde_json::from_str::<CellMeta>(trimmed) {
                if meta.record_type == "cell_meta" {
                    workspace_id = meta.workspace_id;
                    node_id = meta.node_id;
                    agent_type_str = meta.agent_type;
                    continue;
                }
            }
            if let Ok(msg) = serde_json::from_str::<Message>(trimmed) {
                messages.push(msg);
            }
        }

        let agent_type: AgentType = agent_type_str.as_str().try_into()
            .unwrap_or(AgentType::Builder);

        Ok(Self {
            cell_id: cell_id.to_string(),
            workspace_id,
            node_id,
            agent_type,
            jsonl_path: jsonl_path.to_path_buf(),
            status: RwLock::new(CellStatus::Active),
            messages: RwLock::new(messages),
            result: RwLock::new(None),
            file_lock: Mutex::new(()),
        })
    }

    /// 扫描 cells 目录，发现所有 cell_id
    pub fn discover_cells(cells_dir: &Path) -> AppResult<Vec<String>> {
        let mut ids = Vec::new();
        if cells_dir.exists() {
            for entry in std::fs::read_dir(cells_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    ids.push(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
        Ok(ids)
    }

    /// 获取 cell 的 JSONL 路径（用于发现）
    pub fn jsonl_path_for(cells_dir: &Path, cell_id: &str) -> PathBuf {
        cells_dir.join(cell_id).join("session.jsonl")
    }
}

#[async_trait]
impl SessionCell for JsonlCell {
    fn cell_id(&self) -> &str { &self.cell_id }
    fn agent_type(&self) -> AgentType { self.agent_type.clone() }
    fn status(&self) -> CellStatus { self.status.blocking_read().clone() }
    fn workspace_id(&self) -> &str { &self.workspace_id }
    fn node_id(&self) -> &str { &self.node_id }

    async fn add_message(&self, role: &str, content: &str) {
        let msg = Message {
            role: role.to_string(),
            content: content.to_string(),
            agent_type: self.agent_type.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        self.messages.write().await.push(msg.clone());

        // 追加一行 JSON 到文件
        let line = serde_json::to_string(&msg).unwrap_or_default();
        let _guard = self.file_lock.lock().await;
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.jsonl_path)
        {
            let _ = writeln!(file, "{}", line);
        }
    }

    fn get_history(&self) -> Vec<Message> {
        self.messages.blocking_read().clone()
    }

    fn message_count(&self) -> usize {
        self.messages.blocking_read().len()
    }

    async fn clear(&self) {
        self.messages.write().await.clear();
        let _guard = self.file_lock.lock().await;
        // 保留首行元数据，只清 session messages
        if let Ok(content) = std::fs::read_to_string(&self.jsonl_path) {
            if let Some(first_line_end) = content.find('\n') {
                let meta_line = &content[..=first_line_end];
                let _ = std::fs::write(&self.jsonl_path, meta_line);
            }
        }
    }

    async fn close(&self) {
        *self.status.write().await = CellStatus::Closed;
    }

    async fn set_result(&self, result: StoredResult) {
        *self.result.write().await = Some(result);
    }

    fn get_result(&self) -> Option<StoredResult> {
        self.result.blocking_read().clone()
    }
}
