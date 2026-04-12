# RigYourGoal
为 Harness Agent 打造的 Sidecar 工作台，用于工具构建。以“目标”为指引，持续巩固。

## 理念
- 目标持久化：本项目以目标的持久化而不是达成作为核心理念。区别于传统的`workflow + skills + agent`的任务最大化设计。本项目追求`agent -> skill -> workflow`的可逆过程。要求最终可以将绝大部分模块做固化；
- 产品经理：简单来说本项目就是一个面对用户的产品经理，外部 Harmess Agent 是项目的 skills generator；将探索、达成交给其他 Harness Agents，内部的 Sidecar Agent 专注于目标优化与管理；
- 保持上下文干净：按“目标”分离workspace，让model专注于他的“作品”；

## 项目结构
```text
RigYourGoal/
├── src/                    # 前端源码
│   ├── types/              # TypeScript 类型
│   ├── services/           # IPC 服务层
│   ├── stores/             # Zustand 状态管理
│   ├── lib/                # 工具函数
│   ├── App.tsx             # 主应用组件
│   └── globals.css         # Tailwind v4 样式
└── src-tauri/              # Tauri 后端
    ├── src/
    │   ├── config/         # 配置模块
    │   ├── db/             # 数据库层
    │   ├── adapter/        # 通信层
    │   ├── llm/            # LLM 提供者
    │   ├── agents/         # Agent 系统
    │   ├── events/         # 事件系统
    │   ├── commands/       # Tauri 命令
    │   └── state/          # 应用状态
    └── migrations/         # SQL 迁移
```

## 关于项目
正在迭代中，敬请期待！