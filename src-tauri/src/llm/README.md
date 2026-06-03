# provider

```
┌──────────────────────────────────────────────────┐
│                   ChatMessage                     │  对话层面：角色区分的消息
│  UserMessage | AssistantMessage | SystemMessage  │
│  ToolResultMessage                               │
└──────────────┬───────────────────────────────────┘
               │
    ┌──────────▼──────────┐
    │  AssistantMessage   │
    │  content: Vec<...>  │
    └──────────┬──────────┘
               │
    ┌──────────▼───────────────────────────────────┐
    │         AssistantContentBlock                 │  内容块层面：可独立流式传输
    │  TextContent | ThinkingContent | ToolCallContent│
    └──────────┬───────────────────────────────────┘
               │  (streaming)
    ┌──────────▼───────────────────────────────────┐
    │             StreamEvent                       │  流传输层面：ContentBlock 生命周期
    │  ContentBlockStart → ContentBlockDelta*       │
    │  → ContentBlockStop → MessageComplete         │
    └──────────────────────────────────────────────┘
```

用法
```
// 新:
let system = SystemMessage { content: system_prompt.to_string() };
let user_msg = ChatMessage::user(user_prompt);
let mut messages = session_lines_to_chat_messages(&history);
messages.push(user_msg);
let mut stream = llm.stream(&system, &messages).await?;
while let Some(event) = stream.next().await {
    match event? {
        StreamEvent::ContentBlockDelta { index, delta: ContentDelta::Text(text) } => {
            emitter.emit_agent_stream(workspace_id, &text, false);
            full_response.push_str(&text);
        }
        StreamEvent::ContentBlockDelta { delta: ContentDelta::Thinking(t), .. } => {
            tracing::debug!("Thinking: {}", t);
        }
        StreamEvent::ContentBlockDelta { delta: ContentDelta::ToolCallName(n), .. } => {
            tracing::debug!("Tool call: {}", n);
        }
        StreamEvent::ContentBlockDelta { delta: ContentDelta::ToolCallArguments(a), .. } => {
            tracing::debug!("Tool args: {}", a);
        }
        StreamEvent::MessageComplete { message } => {
            emitter.emit_agent_stream(workspace_id, "", true);
            // 可从 message.content 提取完整文本
            if let Some(AssistantContentBlock::Text(tc)) = message.content.first() {
                full_response = tc.text.clone();
            }
        }
        _ => {}
    }
}
```

现有的 emit_agent_stream(delta, finished) 继续可用（用于 agent 内部状态推送），但 agent 消费 StreamEvent 时直接逐块调用。

这个方案中 adapt_rig_stream 的状态机实现是最复杂的部分。

SessionLine（持久化格式）↔ ChatMessage（LLM 通信格式）↔ rig::Message（rig 原生格式）三向转换