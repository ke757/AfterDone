## Why

当前 Session 只支持 `Message` 一种行类型，Agent 的副作用（结果产出、文件变更）使用一个独立的、未持久化的 `StoredResult` 管理。这导致：副作用未计入 session 时间线（前端无法完整渲染）、Result 与 Message 割裂（确认流程需额外 API）、`StoredResult` 的 `serde_json::Value` 黑洞丢失类型信息。引入统一的 `SessionLine` 和 `Effect` 概念，使 session 成为 agent 完整行为的可信记录线。

## What Changes

- **BREAKING**: 引入 `SessionLine` 枚举统一 Message 和 Effect 两种行类型
- **BREAKING**: 废弃 `StoredResult` / `ResultStatus`，由 `Effect::Result` 行替代
- **BREAKING**: `Session` trait 接口重写：`add_line` / `get_lines` / `truncate`
- 新增 `EffectType` 枚举：`Result`（最新优先展示）、`FileChange`（历史累加展示）
- `SessionCell` trait 移除 `set_result`/`get_result`，新增 `add_effect`/`latest_effect`/`redo_from`
- Agent 不感知 Effect，所有 Effect 由 AgentRuntime（tool 交互点）+ ResultHandler（结果处理点）写入
- Redo 场景：`truncate(at_index)` 截断后续行，重新追加
- JSONL 首行 CellMeta 保留（合并原 cell.json 用途）
- 前端 `CellInfo` 字段变更（`message_count` → `line_count`，新增 `has_result`/`last_result_at`）

## Capabilities

### New Capabilities

- `session-line`: 统一的 session 行模型（Message + Effect 枚举），支持追加与截断
- `effect-result`: Agent 阶段产出结果以 Effect::Result 行记录，前端取最新一条展示
- `effect-file-change`: Tool 调用导致的文件变更自动记录为 Effect::FileChange 行
- `session-redo`: 指定行号截断 session 后续内容，支持对话回退重来

### Modified Capabilities

<!-- 无现有 specs，无需修改 -->

## Impact

- `session/types.rs` — `Message` struct 变为 `SessionLine::Message` variant
- `session/cell/result.rs` — **删除**
- `session/session/` — `Session` trait 接口重写，InMemory + JSONL 实现适配
- `session/cell/` — `SessionCell` trait 接口变更，所有 concrete cell 适配
- `session/manager.rs` — `CellInfo` 字段变更，`load_all_cells` 适配新 JSONL 格式
- `agents/runtime/runtime.rs` — `handle_and_store` 改为 `handle_output`（写 effect + persist）
- `agents/runtime/handlers/` — `ResultHandler` trait 从产出 `StoredResult` 改为产出 `serde_json::Value` + `persist` 单独调用
- `commands/sessions.rs` — `CellHistoryResponse` 返回 `Vec<SessionLine>`
- `commands/summarizer.rs` — 用 `latest_effect(Result)` 替代 `get_result()`
- `services/summarizer.rs` — 接收 `&[SessionLine]` 替代 `&[Message]`
- `src/services/commands.ts` — `CellInfo` 接口更新
- `src/stores/chatStore.ts` — 消息列表改为 `SessionLine[]`
