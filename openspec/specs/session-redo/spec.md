## 新增需求

### 需求: 按行号截断 Session
系统必须支持在指定 0-based 行号处截断 session，移除该索引及之后的所有行，同时保留 CellMeta 首行。

#### 场景: 从中间截断
- **WHEN** `truncate(3)` 被调用，session 包含 CellMeta + 6 行（索引 0-5）
- **THEN** 索引 3、4、5 的行必须被移除；索引 0、1、2 的行必须保留；CellMeta 首行必须保持不变

#### 场景: 从末尾截断
- **WHEN** `truncate(6)` 被调用，session 包含 CellMeta + 6 行
- **THEN** 不得移除任何行（索引 6 超出最后一行）

#### 场景: 从头截断
- **WHEN** `truncate(0)` 被调用，session 包含 CellMeta + 3 行
- **THEN** 所有 3 行必须被移除；CellMeta 首行必须保持不变

### 需求: JSONL 截断持久化
对于 `JsonlSession`，`truncate()` 必须通过重写 JSONL 文件（仅保留剩余行）来持久化变更。

#### 场景: 截断持久化到磁盘
- **WHEN** `truncate(2)` 被调用，JsonlSession 的 JSONL 文件包含 CellMeta + 5 行
- **THEN** 操作后磁盘上的 JSONL 文件必须只包含 CellMeta + 第 0 和 1 行

### 需求: 内存 Session 截断
对于 `InMemorySession`，`truncate()` 必须直接修改内存中的行集合。

#### 场景: 内存 session 截断
- **WHEN** `truncate(1)` 被调用，InMemorySession 包含 3 行
- **THEN** `get_lines()` 必须只返回第一行

### 需求: 截断索引仅针对 SessionLine
截断索引必须只对应于 `SessionLine` 的索引，不包括 CellMeta 首行。

#### 场景: 索引排除 CellMeta
- **WHEN** 一个 JSONL 文件包含 1 行 CellMeta 首行和 5 行 SessionLine 条目
- **THEN** `truncate(2)` 必须移除第 3、4、5 条 SessionLine 条目，保留 CellMeta + 前 2 条 SessionLine
