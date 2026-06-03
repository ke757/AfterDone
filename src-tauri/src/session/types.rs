use serde::{Deserialize, Serialize};

/// Effect 类型 — 区分 agent 副作用的语义类别
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EffectType {
    /// Agent 阶段产出摘要 — 前端只展示最新一条
    Result,
    /// Tool 调用导致的文件变更摘要 — 前端历史累加展示
    FileChange,
}

/// 统一的 session 行类型 — JSONL 文件中每一行
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SessionLine {
    Message {
        role: String,
        content: String,
        agent_type: String,
        created_at: String,
    },
    Effect {
        effect_type: EffectType,
        agent_type: String,
        content: serde_json::Value,
        created_at: String,
    },
}

impl SessionLine {
    /// 获取该 session 行的创建时间字符串（ISO-8601 格式）
    pub fn created_at_str(&self) -> &str {
        match self {
            SessionLine::Message { created_at, .. }
          | SessionLine::Effect { created_at, .. } => created_at,
        }
    }

    /// 获取该 session 行对应的 agent 类型名称
    pub fn agent_type_str(&self) -> &str {
        match self {
            SessionLine::Message { agent_type, .. }
          | SessionLine::Effect { agent_type, .. } => agent_type,
        }
    }
}
