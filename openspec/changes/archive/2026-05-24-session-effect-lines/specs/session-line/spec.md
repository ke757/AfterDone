## 新增需求

### 需求: SessionLine 枚举类型
系统必须提供 `SessionLine` 枚举，包含 `Message` 和 `Effect` 两个变体。

#### 场景: Message 序列化为 JSON
- **WHEN** 一个 `SessionLine::Message` 被序列化为 JSON
- **THEN** 输出必须包含 `"type": "message"`、`"role"`、`"content"`、`"agent_type"`、`"created_at"` 字段

#### 场景: Effect 序列化为 JSON
- **WHEN** 一个 `SessionLine::Effect` 被序列化为 JSON
- **THEN** 输出必须包含 `"type": "effect"`、`"effect_type"`、`"agent_type"`、`"content"`、`"created_at"` 字段

#### 场景: 从 JSON 反序列化 type tag
- **WHEN** 一个包含 `"type": "message"` 的 JSON 对象被反序列化
- **THEN** 系统必须将其解析为 `SessionLine::Message`

### 需求: JSONL 中的 Session 行存储
系统必须将每个 `SessionLine` 作为 JSONL 文件的一行持久化，首行为 `CellMeta` 记录。

#### 场景: 写入 Session 行到 JSONL 文件
- **WHEN** 一个 `SessionLine` 被添加到 session
- **THEN** 该行必须作为单行 JSON 对象追加到 JSONL 文件

#### 场景: 从 JSONL 文件加载 Session 行
- **WHEN** 一个 JSONL 文件被加载
- **THEN** 系统必须将首行解析为 `CellMeta`，后续行解析为 `SessionLine`

### 需求: Session trait 接口
`Session` trait 必须暴露 `add_line`、`get_lines`、`line_count`、`truncate` 和 `clear` 方法。

#### 场景: 添加一行到 session
- **WHEN** `add_line(line)` 被调用
- **THEN** 该行必须追加到内存集合中并持久化（若适用）

#### 场景: 获取所有行
- **WHEN** `get_lines()` 被调用
- **THEN** 系统必须按插入顺序返回所有 `SessionLine` 值

#### 场景: 获取 Message 子集
- **WHEN** `get_messages()` 被调用
- **THEN** 系统必须只返回 `type` 为 `"message"` 的行，保持顺序

#### 场景: 获取 Effect 子集
- **WHEN** `get_effects()` 被调用
- **THEN** 系统必须只返回 `type` 为 `"effect"` 的行，保持顺序

### 需求: Effect 类型枚举
系统必须提供 `EffectType` 枚举，至少包含 `Result` 和 `FileChange` 两种变体，序列化为 snake_case 字符串。

#### 场景: Result effect type 序列化
- **WHEN** `EffectType::Result` 被序列化
- **THEN** 输出必须是 `"result"`

#### 场景: FileChange effect type 序列化
- **WHEN** `EffectType::FileChange` 被序列化
- **THEN** 输出必须是 `"file_change"`
