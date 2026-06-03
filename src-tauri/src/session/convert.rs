//! SessionLine → ChatMessage 转换
//!
//! SessionLine 是 JSONL 持久化格式，ChatMessage 是 LLM 通信的类型化格式。

use crate::llm::message::ChatMessage;
use crate::session::SessionLine;

/// SessionLine → ChatMessage（用于构建 LLM 请求上下文）
pub fn session_lines_to_chat_messages(lines: &[SessionLine]) -> Vec<ChatMessage> {
    lines
        .iter()
        .filter_map(|line| match line {
            SessionLine::Message { role, content, .. } => {
                let msg = match role.as_str() {
                    "user" => ChatMessage::user(content),
                    "assistant" => ChatMessage::assistant_text(content),
                    "system" => ChatMessage::system(content),
                    _ => ChatMessage::assistant_text(content),
                };
                Some(msg)
            }
            SessionLine::Effect { .. } => None,
        })
        .collect()
}
