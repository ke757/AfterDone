# AfterDone

A goal-persistence workbench for users, and a Sidecar tool-building platform for Harness Agents. Goals are the core anchor — driving Agents to continuously explore, build, and solidify.

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.90+-orange?style=flat-square&logo=rust" alt="Rust 1.90+" />
  <img src="https://img.shields.io/badge/Desktop-Tauri-24C8D8?style=flat-square&logo=tauri&logoColor=white" alt="Tauri" />
  <img src="https://img.shields.io/badge/Runtime-Tokio-10B981?style=flat-square" alt="Tokio Runtime" />
  <img src="https://img.shields.io/badge/Framework-Rig-blue?style=flat-square" alt="Rig Framework" />
  <img src="https://img.shields.io/badge/License-MIT-black?style=flat-square" alt="MIT License" />
</p>

<p align="center">
  <a href="#about">About</a> ·
  <a href="./README_CN.md">中文文档</a>
</p>

---

## Core Philosophy

**Goal persistence over task maximization.**

Traditional Agent frameworks optimize for task completion (`workflow + skills + agent`). After achieving its goals, this project pursues a process that is **optimizable and solidifiable** (`agent → skill → workflow`):

- **Goal Persistence**: Goals are not one-shot tasks but continuously evolving anchors. The system treats "solidification" — not "completion" — as the terminal state. Most exploration results should eventually settle into reusable Skills or Workflows.
- **Separation of Concerns**: This project acts as the "Product Manager" — responsible for goal refinement, decomposition, and optimization. External Harness Agents serve as Skill Generators for actual exploration and building. Internal Sidecar Agents focus on goal-state progression and result solidification.
- **Context Isolation**: Each goal gets its own NodeSpace (workspace), keeping the model's context clean and focused, free from cross-goal contamination.

## Agent System

Goals pass through the following Agent phases over their lifecycle, coordinated by `AgentSupervisor`:

| Agent | Trigger Status | Responsibility |
|---|---|---|
| **Summarizer** | `draft` | Parses raw user input; extracts title, description, acceptance criteria, and constraints |
| **Builder** | `pinned` / `building` / `failed` | Executes builds using available Skills; creates WorkNodes to record artifacts |
| **Executor** | `building` (execution phase) | Runs build artifacts, captures bugs, and produces conclusions |
| **Optimizer** | `achieved` / `optimizing` | Iteratively optimizes achieved goals until solidified into `solidified` status |

## NodeSpace Mechanism

Each goal, once initialized, is assigned a **NodeSpace** as its dedicated workspace:

- **NodeSpace**: Bound 1:1 to a Goal; holds `GOAL.md` (goal description) and `PLAN.md` (execution plan)
- **WorkNode**: Milestone execution nodes within a NodeSpace, organized as a tree; each node maintains `BUG.md`, `USER_MANUAL.md`, and `CONCLUSION.md`
- **GeneratorTask**: Tracks Skill-generation tasks dispatched to Harness Agents, along with their results

## Tech Stack

| Layer | Technology |
|---|---|
| Desktop Framework | Tauri 2 |
| Frontend | React 19 + TypeScript + Vite 7 |
| Styling | Tailwind CSS v4 |
| State Management | Zustand 5 |
| Backend Language | Rust |
| Async Runtime | Tokio |
| Database | SQLite (via sqlx 0.8) |
| LLM Integration | rig-core 0.33 |
| External Communication | WebSocket (OpenClaw Gateway) |

## Project Structure

```text
AfterDone/
├── src/                        # Frontend source
│   ├── types/                  # TypeScript type definitions
│   ├── services/               # Tauri IPC service wrappers
│   ├── stores/                 # Zustand global state
│   ├── hooks/                  # Custom React Hooks
│   ├── lib/                    # Utility functions
│   ├── App.tsx                 # Root application component
│   └── globals.css             # Tailwind v4 global styles
└── src-tauri/                  # Rust backend
    ├── src/
    │   ├── config/             # Configuration loading (TOML)
    │   ├── db/                 # Database models & Repositories
    │   ├── adapter/            # Communication adapters (IPC / OpenClaw / Mock)
    │   ├── llm/                # LLM Provider abstraction (rig-core based)
    │   ├── agents/             # Sidecar Agent system
    │   ├── noderepo/           # NodeSpace & WorkNode management
    │   ├── events/             # Frontend event push (EventBridge)
    │   ├── commands/           # Tauri command registration
    │   └── state/              # AppState (global runtime state)
    └── migrations/             # SQLite migration scripts
```

<a id="about"></a>

## About

The project is actively evolving, stay tuned and coming soon!
