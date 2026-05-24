拆分后的职责:
```
┌─────────────────────────────────────────────┐
│ Session (trait) — 纯消息存取 + 持久化策略    │
│   add_message() / get_history() / clear()   │
└─────────────────────────────────────────────┘
          ↑                    ↑
  ┌──────────────┐    ┌──────────────┐
  │InMemorySession│    │ JsonlSession │
  │  (临时会话)   │    │ (JSONL持久化)│
  └──────────────┘    └──────────────┘
          ↑                    ↑
          │                    │
┌─────────┴────────────────────┴──────────┐
│ SessionCell (trait) — Cell 身份 + 行为   │
│   cell_id / workspace_id / node_id      │
│   cell_type() / agent_type()            │
│   result / status                       │
│   session: Arc<dyn Session>  ← 委托     │
└─────────────────────────────────────────┘
          ↑          ↑          ↑
  ┌──────────┐ ┌────────┐ ┌──────────┐
  │GoalSummary│ │BuildCell│ │GoalInfo  │
  │(InMemory) │ │(Jsonl) │ │(InMemory)│
  └──────────┘ └────────┘ └──────────┘
```

每个具体的 Cell struct 构造时决定注入哪种 Session，外部完全无感。对于当前场景，GoalSummaryCell 一定用 InMemory，BuildCell 一定用 Jsonl——这是构造阶段的决策，不需要运行时切换。
调整后的目录
```
src-tauri/src/session/
├── mod.rs              # 暴露公共接口
├── types.rs            # Message
├── session/            # [NEW] Session 层
│   ├── mod.rs          # Session trait
│   ├── in_memory.rs    # InMemorySession（从 cell/in_memory.rs 迁入）
│   └── jsonl.rs        # JsonlSession（从 cell/jsonl_cell.rs 迁入）
├── cell/               # Cell 层
│   ├── mod.rs          # SessionCell trait + CellStatus + CellInfo
│   ├── result.rs       # StoredResult + ResultStatus（保持不变）
│   ├── goal_summary.rs # GoalSummaryCell: impl SessionCell
│   ├── build.rs        # BuildCell: impl SessionCell
│   ├── executor.rs     # ExecutorCell: impl SessionCell
│   ├── optimizer.rs    # OptimizerCell: impl SessionCell
│   └── goal_info.rs    # GoalInfoCell: impl SessionCell
└── manager.rs          # CellManager（lift 到 session/ 根目录）
```

CellManager 提到 session/ 根目录是因为它管理的是 Cell，不是 Session，放 cell/ 下形成 cell::manager 也可以，但概念上它和 cell 目录平级更干净。

文件目录：
```
nodes/{node_id}/cells/{cell_id}.jsonl
```