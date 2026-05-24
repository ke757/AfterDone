//! JsonlSession — JSONL 持久化会话
//!
//! JSONL 文件格式：首行为 session 元数据（CellMeta），后续行为 SessionLine 记录。
//! 路径：nodes/{node_id}/cells/{cell_id}.jsonl

use std::path::{Path, PathBuf};
use tokio::sync::{Mutex, RwLock};
use async_trait::async_trait;

use super::Session;
use crate::session::SessionLine;
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
    lines: RwLock<Vec<SessionLine>>,
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
            lines: RwLock::new(Vec::new()),
            jsonl_path: jsonl_path.to_path_buf(),
            file_lock: Mutex::new(()),
        })
    }

    /// 从已有 JSONL 文件加载，返回 Session 实例 + 元信息
    pub fn load(jsonl_path: &Path) -> AppResult<(Self, JsonlSessionMeta)> {
        let content = std::fs::read_to_string(jsonl_path)?;
        let mut lines = Vec::new();
        let mut meta = None;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if meta.is_none() {
                let header_result = serde_json::from_str::<SessionHeader>(trimmed);
                if let Ok(header) = header_result {
                    if header.record_type == "cell_meta" {
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
            if let Ok(sl) = serde_json::from_str::<SessionLine>(trimmed) {
                lines.push(sl);
            }
        }

        let meta = meta.ok_or_else(|| {
            crate::error::AppError::Agent("Missing session header in JSONL file".to_string())
        })?;

        Ok((
            Self {
                lines: RwLock::new(lines),
                jsonl_path: jsonl_path.to_path_buf(),
                file_lock: Mutex::new(()),
            },
            meta,
        ))
    }

    /// 扫描 cells 目录，发现所有 cell_id（从 .jsonl 文件名中提取）
    pub fn discover_cells(cells_dir: &Path) -> AppResult<Vec<String>> {
        let mut ids = Vec::new();
        if cells_dir.exists() {
            for entry in std::fs::read_dir(cells_dir)? {
                let entry = entry?;
                let file_name = entry.file_name().to_string_lossy().to_string();
                if entry.file_type()?.is_file() && file_name.ends_with(".jsonl") {
                    ids.push(file_name.trim_end_matches(".jsonl").to_string());
                }
            }
        }
        Ok(ids)
    }

    /// 重写整个 JSONL 文件（保留首行 header，写入 lines）
    fn rewrite_file(&self) -> AppResult<()> {
        let _guard = self.file_lock.blocking_lock();
        let content = std::fs::read_to_string(&self.jsonl_path)?;
        let header_end = content.find('\n').map(|i| i + 1).unwrap_or(0);
        let header = &content[..header_end];

        let lines = self.lines.blocking_read();
        let mut output = String::from(header);
        for line in lines.iter() {
            if let Ok(json) = serde_json::to_string(line) {
                output.push_str(&json);
                output.push('\n');
            }
        }
        std::fs::write(&self.jsonl_path, output)?;
        Ok(())
    }
}

#[async_trait]
impl Session for JsonlSession {
    async fn add_line(&self, line: SessionLine) {
        self.lines.write().await.push(line.clone());

        let json = serde_json::to_string(&line).unwrap_or_default();
        let _guard = self.file_lock.lock().await;
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.jsonl_path)
        {
            let _ = writeln!(file, "{}", json);
        }
    }

    fn get_lines(&self) -> Vec<SessionLine> {
        self.lines.blocking_read().clone()
    }

    fn line_count(&self) -> usize {
        self.lines.blocking_read().len()
    }

    async fn truncate(&self, at_index: usize) {
        {
            let mut lines = self.lines.write().await;
            if at_index < lines.len() {
                lines.truncate(at_index);
            } else {
                return;
            }
        }
        let _ = self.rewrite_file();
    }

    async fn clear(&self) {
        self.lines.write().await.clear();
        let _guard = self.file_lock.lock().await;
        if let Ok(content) = std::fs::read_to_string(&self.jsonl_path) {
            if let Some(first_line_end) = content.find('\n') {
                let header = &content[..=first_line_end];
                let _ = std::fs::write(&self.jsonl_path, header);
            }
        }
    }
}
