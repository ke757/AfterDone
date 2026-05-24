## 新增需求

### 需求: Agent 结果存储为 Effect 行
系统必须将每次 Agent 运行的输出记录为 `effect_type: "result"` 的 `SessionLine::Effect`，序列化后的 `AgentOutput` 作为 `content`。

#### 场景: Builder agent 运行创建 result effect
- **WHEN** Builder agent 完成一次运行
- **THEN** 系统必须向 cell 的 session 中添加一条 `SessionLine::Effect { effect_type: Result, content: <序列化后的 AgentOutput> }`

#### 场景: Summarizer 对话 turn 创建 result effect
- **WHEN** Summarizer 完成一次对话 turn
- **THEN** 系统必须向 cell 的 session 中添加一条 `SessionLine::Effect { effect_type: Result, content: <序列化后的 ConversationTurn> }`

### 需求: 最新 result 检索
系统必须提供从 session 中检索最近一条 `Effect::Result` 行的方法。

#### 场景: 从包含多条结果的 session 中获取最新一条
- **WHEN** `latest_effect(EffectType::Result)` 被调用，session 中包含 result 行在位置 3、7、12
- **THEN** 系统必须返回位置 12 的行

#### 场景: 从无结果的 session 中获取最新一条
- **WHEN** `latest_effect(EffectType::Result)` 被调用，session 中没有任何 `Effect::Result` 行
- **THEN** 系统必须返回 `None`

### 需求: Result effect 对 Agent 上下文不可见
系统在构建 Agent 上下文（LLM 输入）时必须排除所有 `Effect` 行。

#### 场景: Agent 上下文排除 effect
- **WHEN** 从包含 Message 和 Effect 行的 session 中构建 Agent 上下文
- **THEN** 上下文必须只包含 `SessionLine::Message` 行

### 需求: StoredResult 退役
系统必须不再使用 `StoredResult` 或 `ResultStatus`。所有结果数据必须通过 `Effect::Result` 行流动。

#### 场景: SessionCell 接口中无 StoredResult
- **WHEN** 开发者检查 `SessionCell` trait
- **THEN** trait 上必须没有 `set_result` 或 `get_result` 方法

#### 场景: Summarizer confirm 从 effect 行读取
- **WHEN** summarizer confirm 命令被调用
- **THEN** 必须从 `latest_effect(EffectType::Result)` 读取结果，而非从 `get_result()`
