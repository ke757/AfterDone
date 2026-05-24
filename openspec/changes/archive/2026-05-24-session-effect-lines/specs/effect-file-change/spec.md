## 新增需求

### 需求: 文件变更 effect 记录
当 tool 调用修改文件时，系统必须自动记录一条 `effect_type: "file_change"` 的 `SessionLine::Effect`。

#### 场景: Tool 写入文件
- **WHEN** 一次 transport tool 调用成功写入一个或多个文件
- **THEN** 系统必须向 cell 的 session 中添加一条 `SessionLine::Effect { effect_type: FileChange, content: <tool 名, files[], summary> }`

#### 场景: Tool 执行失败
- **WHEN** 一次 transport tool 调用失败
- **THEN** 系统不得记录 `FileChange` effect（没有文件被修改）

### 需求: 文件变更 effect 结构
每条 `Effect::FileChange` 行的 `content` 必须包含 tool 名、受影响的文件路径列表、以及人类可读的变更摘要。

#### 场景: 文件变更 effect 内容格式
- **WHEN** 一个文件变更 effect 被记录
- **THEN** `content` JSON 必须包含 `"tool"`（字符串）、`"files"`（字符串数组）和 `"summary"`（字符串）字段

### 需求: 文件变更累加展示
系统必须支持展示 session 中所有历史 `FileChange` effect 的累加列表。

#### 场景: 检索所有文件变更
- **WHEN** 一个 session 包含 3 条 `Effect::FileChange` 行
- **THEN** 按 `FileChange` 过滤后的 `get_effects()` 必须按插入顺序返回全部 3 条行

### 需求: Agent 不感知文件变更 effect
Agent 上下文中不得包含 `Effect::FileChange` 行。Effect 记录必须在 transport/执行层发生，而非 agent 层。

#### 场景: Agent 上下文排除文件变更 effect
- **WHEN** 构建 Agent 上下文（LLM 输入）时
- **THEN** `Effect::FileChange` 行必须从上下文中排除
