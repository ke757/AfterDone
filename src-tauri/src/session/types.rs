use serde::{Deserialize, Serialize};

/// 会话消息 — 会话的基本单元，同时作为 JSONL 持久化格式
///
/// 每个 Message 就是 JSONL 文件中的一行。
/// node_id 不需要存在 Message 中 — 文件路径 `nodes/{node_id}/session.jsonl` 已隐含归属。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub agent_type: String,
    pub created_at: String,
}
