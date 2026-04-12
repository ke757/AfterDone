# PinTheGoal

面向用户的目标持久化工作台，同时也是 Harness Agents 的 Sidecar 工具构建平台。以"目标"为核心锚点，驱动 Agent 持续探索、构建与固化。

## 核心理念

**目标持久化，而非任务最大化。**

传统 Agent 框架倾向于最大化单次任务的完成度（`workflow + skills + agent`），而本项目在达成目标后，追求一个**可优化、可固化**的过程（`agent → skill → workflow`）：

- **目标持久化**：目标不是一次性的任务，而是持续演化的锚点。系统以目标的"固化"而非"完成"作为终态，绝大部分探索成果最终应沉淀为可复用的 Skill 或 Workflow。
- **职责分离**：本项目作为"产品经理"角色，负责目标的细化、分解与优化管理；外部 Harness Agents 作为 Skill Generator，负责实际的探索与构建；内部的 Sidecar Agents 专注于目标状态推进与结果固化。
- **上下文隔离**：每个目标对应独立的 NodeSpace（工作空间），模型始终在干净、聚焦的上下文中运作，避免跨目标的信息污染。

## Agent 系统

目标在生命周期内依次经历以下 Agent 处理阶段，由 `AgentSupervisor` 统一调度：

| Agent | 触发状态 | 职责 |
|---|---|---|
| **Summarizer** | `draft` | 解析用户原始输入，提炼目标标题、描述、验收标准与约束条件 |
| **Builder** | `pinned` / `building` / `failed` | 利用 Skills 执行构建，创建 WorkNode 记录执行产物 |
| **Executor** | `building`（执行阶段） | 运行构建产物，捕获 Bug 并输出结论 |
| **Optimizer** | `achieved` / `optimizing` | 对达成的目标进行优化迭代，直至固化为 `solidified` 状态 |

## NodeSpace 机制

每个目标初始化后会创建一个 **NodeSpace**，作为该目标的专属工作空间：

- **NodeSpace**：与 Goal 1:1 绑定，持有 `GOAL.md`（目标描述）和 `PLAN.md`（执行计划）
- **WorkNode**：NodeSpace 内的里程碑执行节点，树状结构，每个节点维护 `BUG.md`、`USER_MANUAL.md`、`CONCLUSION.md`
- **GeneratorTask**：记录向 Harness Agent 派发的 Skill 生成任务及其结果

## 技术栈

| 层次 | 技术 |
|---|---|
| 桌面框架 | Tauri 2 |
| 前端 | React 19 + TypeScript + Vite 7 |
| 样式 | Tailwind CSS v4 |
| 状态管理 | Zustand 5 |
| 后端语言 | Rust |
| 异步运行时 | Tokio |
| 数据库 | SQLite (via sqlx 0.8) |
| LLM 接入 | rig-core 0.33 |
| 外部通信 | WebSocket（对接 OpenClaw） |

## 项目结构

```text
PinTheGoal/
├── src/                        # 前端源码
│   ├── types/                  # TypeScript 类型定义
│   ├── services/               # Tauri IPC 服务封装
│   ├── stores/                 # Zustand 全局状态
│   ├── hooks/                  # 自定义 React Hooks
│   ├── lib/                    # 通用工具函数
│   ├── App.tsx                 # 应用根组件
│   └── globals.css             # Tailwind v4 全局样式
└── src-tauri/                  # Rust 后端
    ├── src/
    │   ├── config/             # 配置加载（TOML）
    │   ├── db/                 # 数据库模型与 Repository
    │   ├── adapter/            # 通信适配层（IPC / OpenClaw / Mock）
    │   ├── llm/                # LLM Provider 抽象（基于 rig-core）
    │   ├── agents/             # Sidecar Agent 系统
    │   ├── noderepo/           # NodeSpace & WorkNode 管理
    │   ├── events/             # 前端事件推送（EventBridge）
    │   ├── commands/           # Tauri 命令注册
    │   └── state/              # AppState（全局运行时状态）
    └── migrations/             # SQLite 迁移脚本
```

## 关于

项目正在积极迭代中，敬请期待！
