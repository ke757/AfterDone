## Context

AfterDone 的 session 模块是 workbench 核心——Cell 概念源自 Jupyter Notebook，用于划分 session 上下文。当前 Session 仅管理 `Message` 行，Agent 的副作用（结果产出、文件变更）通过独立的 `StoredResult`（未持久化）管理。在前一次 change（`session-cell-refactor`）完成后，Session 与 Cell 已解耦为两层结构，为本次统一行模型铺平了道路。

**当前架构**:
```
Session (trait)  →  InMemorySession / JsonlSession  → 仅管理 Vec<Message>
SessionCell (trait)  →  GoalSummaryCell / BuildCell / ...  →  持有 session + result
StoredResult (独立)  →  存储于 Cell，不持久化
```

**约束**:
- Agent 不感知 Effect（effect 对 agent 的 LLM context 不可见）
- JSONL 为持久化格式，首行 CellMeta + 后续行序列
- 前端需根据行类型差异化渲染（Message 为对话气泡，Effect 为结果面板/文件变更摘要）
- 不考虑向后兼容旧 JSONL 格式（开发初期）

## Goals / Non-Goals

**Goals:**
- 引入统一的 `SessionLine` 枚举替代分散的 `Message` struct 和 `StoredResult`
- `EffectType::Result` 替代 `StoredResult` 的"最新结果"语义
- `EffectType::FileChange` 记录 tool 调用的文件副作用
- Redo 能力：`truncate(at_index)` 支持编辑历史消息后重新生成
- CellMeta 合入 JSONL 首行，去 `cell.toml`

**Non-Goals:**
- 不改变 Agent 的 `SideCarAgent::run()` 签名（Agent 不直接写入 effect）
- 不实现前端 Cell 组件渲染（属于后续 UI change）
- 不实现 `EffectType::Plan` / `Error` / `Decision` 等（枚举封闭，后续可扩展）
- 不改变 WorkHub / Goal / WorkNode 的领域模型

## Decisions

### 1. SessionLine 枚举统一 Message 和 Effect

**选择**: `enum SessionLine { Message { ... }, Effect { ... } }`，带 `#[serde(tag = "type")]`

**替代方案**: Message 和 Effect 作为两个独立 struct，Session 维护两个 Vec
- 否决原因：无法保证时间顺序、JSONL 行无序、前端渲染顺序不确定

**Rationale**: 统一行类型保证时间线完整性，JSONL 每行 self-describing，前端按 `type` tag 过滤渲染即可

### 2. CellMeta 合入 JSONL 首行

**选择**: JSONL 首行为 `{"type":"cell_meta",...}`，后续行为 `SessionLine` 序列

**替代方案 A**: 分离 `cell.toml`
- 否决原因：多文件一致性问题、Session 仍需知道 cell_meta 信息才能重建

**替代方案 B**: 元数据作为 `Effect` 行
- 否决原因：Effect 语义上不应包含"Cell 创建"这类基础元数据

**Rationale**: 单文件自描述，`JsonlSession::load()` 返回 `(Session, JsonlSessionMeta)`，由 `CellManager::load_all_cells()` 根据 meta 重建 concrete cell

### 3. Effect 写入机制

**选择**: AgentRuntime 内的 tool 调用点自动写 `Effect::FileChange`，ResultHandler 写 `Effect::Result`

```
AgentRuntime::run_task()
  ├─ llm.stream()      ──▶ 写 Message 行
  ├─ transport.execute()──▶ 写 Effect::FileChange 行（自动，Agent 无感）
  └─ ResultHandler      ──▶ 写 Effect::Result 行（自动，Agent 无感）
                                 └─ persist(DB) 单独调用
```

**Rationale**: Agent 保持纯粹（输入 context → 产生文本/调用 tool → 输出结构化结果），副作用记录是框架层职责

### 4. ResultHandler 接口变更

**选择**: 拆分为 `to_effect_content(output) → serde_json::Value` + `persist(db, output, goal_id, workspace_id)`

```rust
pub trait ResultHandler {
    fn agent_type(&self) -> AgentType;
    fn to_effect_content(&self, output: &AgentOutput) -> AppResult<serde_json::Value>;
    async fn persist(&self, db: &DatabasePool, output: &AgentOutput, goal_id: &str, workspace_id: &str) -> AppResult<()>;
}
```

**替代方案**: 维持现有 `handle() → StoredResult` 然后 AgentRuntime 再拆解
- 否决原因：多余的 StoredResult 中间态、两次序列化（AgentOutput→Value→StoredResult→Value）

**Rationale**: 直接序列化 `AgentOutput` 为 effect content，`persist()` 直接解构 `AgentOutput`（非 `StoredResult`），减少一次 serde 往返

### 5. Redo 机制

**选择**: `Session::truncate(at_index: usize)` — 从指定行号起截断所有行（不含首行 CellMeta）

**行为**: 用户编辑第 4 行消息 → `truncate(4)` → JSONL 文件只保留前 4 行（行1 CellMeta + 行2-4 前三条 SessionLine）→ 重新追加后续内容

**JSONL 实现**: 读取全部行入内存，截断 `lines[..at_index]`，重写整个 JSONL 文件（首行 CellMeta 保留在前）

**Rationale**: 简单可靠，无需维护行号偏移或追加式 undo log

### 6. Effect 展示策略

| EffectType | 存储方式 | 前端展示 |
|-----------|---------|---------|
| `Result` | 追加写入（JSONL 中每次 agent run 一行） | 只展示最新一条 `latest_effect(Result)` |
| `FileChange` | 追加写入（每次 tool call 一行） | 历史累加展示（所有行） |

`SessionCell::latest_effect(effect_type)` 从 `get_lines()` 中反向查找第一个匹配的 Effect 行

### 7. 前端类型适配

`CellInfo` 字段变更：

```
旧: { cell_id, cell_type, agent_type, status, message_count, result_status }
新: { cell_id, cell_type, agent_type, status, line_count, has_result, last_result_at }
```

`SessionLine` TypeScript 接口：

```typescript
type SessionLine = 
  | { type: "message"; role: string; content: string; agent_type: string; created_at: string }
  | { type: "effect"; effect_type: "result" | "file_change"; agent_type: string; content: any; created_at: string }
```

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| `truncate()` 写整文件在大 session 下慢 | 当前 session 规模小（数十行），后续可优化为 `seek` 截断 |
| Effect 行增多后 `latest_effect()` 全量扫描 | 合理大小（<100行）下无性能问题，后续可加 effect 索引 |
| `SessionLine::Message` 的 `agent_type` 与 `SessionLine::Effect` 的 `agent_type` 字段重复 | 接受——统一字段便于前端通用渲染，无需根据 variant 判断字段存在性 |
| 前端需适配 SessionLine 渲染（按 type 过滤） | 属于后续 UI change，本次仅变更类型接口 |

## Migration Plan

1. 旧 JSONL 文件无向后兼容——开发初期无生产数据，直接删除重新生成
2. 部署后首次启动，旧 JSONL 文件解析失败（无 `"type"` tag），`load_all_cells` 跳过并 warning
3. StoredResult 数据丢失——Summarizer confirm 后数据已在 DB，不受影响
4. 回滚方案：恢复旧 session 代码（旧 JSONL 格式） + 删除新格式文件

## Open Questions

- 无
