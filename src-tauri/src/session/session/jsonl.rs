//! JsonlSession — JSONL 持久化会话
//!
//! JSONL 文件格式：首行为 session 元数据，后续行为 Message 记录。
//! 路径由 Cell 层指定：nodes/{node_id}/cells/{cell_id}/session.jsonl

use std::path::{Path, PathBuf};
use tokio::sync::{Mutex, RwLock};
use async_trait::async_trait;

use super::Session;
use crate::session::Message;
use crate::session::cell::CellType;
use crate::error::AppResult;
use crate::workhub::AgentType;

/// JSONL 首行元数据
#[derive(serde::Serialize, serde::Deserialize)]
struct SessionHeader {
    #[serde(rename = "type")]
    record_type: String,
    cell_id: String,
    workspace_id: String,
    node_id: String,
    cell_type: CellType,
    agent_type: String,
    created_at: String,
}

/// 从 JSONL 加载时还原的元信息，供 Cell 层重建
pub struct JsonlSessionMeta {
    pub cell_id: String,
    pub workspace_id: String,
    pub node_id: String,
    pub cell_type: CellType,
    pub agent_type: AgentType,
}

pub struct JsonlSession {
    messages: RwLock<Vec<Message>>,
    jsonl_path: PathBuf,
    file_lock: Mutex<()>,
}

impl JsonlSession {
    /// 创建新的 JsonlSession 并写入元数据首行
    pub fn create(jsonl_path: &Path, meta: &JsonlSessionMeta) -> AppResult<Self> {
        if let Some(parent) = jsonl_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let header = SessionHeader {
            record_type: "cell_meta".to_string(),
            cell_id: meta.cell_id.clone(),
            workspace_id: meta.workspace_id.clone(),
            node_id: meta.node_id.clone(),
            cell_type: meta.cell_type,
            agent_type: meta.agent_type.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        let header_line = serde_json::to_string(&header)? + "\n";
        std::fs::write(jsonl_path, header_line)?;

        Ok(Self {
            messages: RwLock::new(Vec::new()),
            jsonl_path: jsonl_path.to_path_buf(),
            file_lock: Mutex::new(()),
        })
    }

    /// 从已有 JSONL 文件加载，返回 Session 实例 + 元信息
    pub fn load(jsonl_path: &Path) -> AppResult<(Self, JsonlSessionMeta)> {
        let content = std::fs::read_to_string(jsonl_path)?;
        let mut messages = Vec::new();
        let mut meta = None;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if meta.is_none() {
                if let Ok(header) = serde_json::from_str::<SessionHeader>(trimmed) {
                    if header.record_type == "cell_meta" && meta.is_none() {
                        meta = Some(JsonlSessionMeta {
                            cell_id: header.cell_id,
                            workspace_id: header.workspace_id,
                            node_id: header.node_id,
                            cell_type: header.cell_type,
                            agent_type: header.agent_type.as_str().try_into()
                                .unwrap_or(AgentType::Builder),
                        });
                        continue;
                    }
                }
            }
            if let Ok(msg) = serde_json::from_str::<Message>(trimmed) {
                messages.push(msg);
            }
        }

        let meta = meta.ok_or_else(|| {
            crate::error::AppError::Agent("Missing session header in JSONL file".to_string())
        })?;

        Ok((
            Self {
                messages: RwLock::new(messages),
                jsonl_path: jsonl_path.to_path_buf(),
                file_lock: Mutex::new(()),
            },
            meta,
        ))
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
}

#[async_trait]
impl Session for JsonlSession {
    async fn add_message(&self, msg: Message) {
        self.messages.write().await.push(msg.clone());

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
        if let Ok(content) = std::fs::read_to_string(&self.jsonl_path) {
            if let Some(first_line_end) = content.find('\n') {
                let header = &content[..=first_line_end];
                let _ = std::fs::write(&self.jsonl_path, header);
            }
        }
    }
}
