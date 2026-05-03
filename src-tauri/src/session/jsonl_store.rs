use std::path::PathBuf;
use tokio::sync::{Mutex, RwLock};
use async_trait::async_trait;

use crate::session::Message;
use crate::error::AppResult;
use super::Session;

/// NodeSession — 持久化会话，关联到 WorkNode
///
/// 内存缓存 + JSONL 文件持久化。
/// 每次 add_message 立即追加一行 JSON 到文件。
#[derive(Debug)]
pub struct NodeSession {
    pub node_id: String,
    jsonl_path: PathBuf,
    messages: RwLock<Vec<Message>>,
    /// 序列化文件写入
    file_lock: Mutex<()>,
}

impl NodeSession {
    /// 创建或加载 node 会话
    pub fn new(node_id: &str, nodes_dir: &std::path::Path) -> AppResult<Self> {
        let node_dir = nodes_dir.join(node_id);
        std::fs::create_dir_all(&node_dir)?;
        let jsonl_path = node_dir.join("session.jsonl");

        let messages = if jsonl_path.exists() {
            Self::load_from_file(&jsonl_path)?
        } else {
            Vec::new()
        };

        Ok(Self {
            node_id: node_id.to_string(),
            jsonl_path,
            messages: RwLock::new(messages),
            file_lock: Mutex::new(()),
        })
    }

    /// 从 JSONL 文件逐行加载 Message
    fn load_from_file(path: &std::path::Path) -> AppResult<Vec<Message>> {
        let content = std::fs::read_to_string(path)?;
        let mut messages = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(msg) = serde_json::from_str::<Message>(trimmed) {
                messages.push(msg);
            }
        }

        Ok(messages)
    }
}

#[async_trait]
impl Session for NodeSession {
    async fn add_message(&self, role: &str, content: &str, agent_type: &str) {
        let now = chrono::Utc::now().to_rfc3339();
        let msg = Message {
            role: role.to_string(),
            content: content.to_string(),
            agent_type: agent_type.to_string(),
            created_at: now,
        };

        // 1. 写入内存缓存
        self.messages.write().await.push(msg.clone());

        // 2. 追加一行 JSON 到文件
        let line = serde_json::to_string(&msg).unwrap_or_default();

        let _guard = self.file_lock.lock().await;
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.jsonl_path)
            .expect("Failed to open session JSONL file");
        writeln!(file, "{}", line).expect("Failed to write session JSONL line");
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
        // 截断文件
        let _ = std::fs::write(&self.jsonl_path, "");
    }
}
