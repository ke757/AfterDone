# AGENTS.md

## Project Overview

AfterDone — a Tauri 2 desktop app: Rust backend + React 19 frontend. A goal-persistence agent workbench where goals drive SideCar agents (Summarizer → Builder → Executor → Optimizer) through lifecycle phases until solidified.

Code is changing fast — no need to worry about architectural compatibility for now.

## Commands

- `pnpm tauri dev` — run the full app (starts Vite dev server on port 1420, then launches Tauri)
- `pnpm tauri build` — production build
- `pnpm dev` — frontend-only dev server (no Rust backend)
- `pnpm build` — frontend-only typecheck + Vite build (`tsc && vite build`)

No test, lint, or format commands are configured yet.

## Architecture

- **Frontend** (`src/`): React 19 + TypeScript (strict mode, `noUnusedLocals`, `noUnusedParameters`) + Zustand 5 + Tailwind CSS v4 + Vite 7. Services in `src/services/` wrap Tauri IPC commands. Stores in `src/stores/`.
- **Backend** (`src-tauri/`): Rust with Tauri 2. The lib crate name is `after_done_lib` (not `after_done`) due to a Windows Cargo naming conflict.
- **Database**: SQLite via sqlx 0.8. Migrations are embedded at compile time with `include_str!` from `src-tauri/migrations/`. Path defaults to `{data_dir}/rig-your-goal/rig-your-goal.db`.
- **Config**: TOML at `{data_dir}/rig-your-goal/config.toml`. Defaults are generated on first run. Env var overrides use `RYG_` prefix: `RYG_OPENCLAW_URL`, `RYG_OPENCLAW_AUTH_TOKEN`, `RYG_LLM_API_KEY`, `RYG_LLM_BASE_URL`, `RYG_LLM_MODEL`, `RYG_LOG_LEVEL`.
- **Transport**: Currently defaults to `MockTransport`. Real communication uses `OpenClawClient` (WebSocket to OpenClaw Gateway).
- **LLM**: `rig-core` based, supports OpenAI/Anthropic/Local providers. Configured via config or env vars.

## Key Backend Modules

| Path | Purpose |
|---|---|
| `agents/` | SideCar agent implementations + `AgentSupervisor` coordinator |
| `adapter/` | Transport layer: `Transport` trait, `IpcTransport`, `OpenClawClient`, `MockTransport` |
| `llm/` | `LlmProvider` trait + `RigProvider` implementation |
| `workhub/` | Domain logic: Goals, Milestones, Skills, WorkSpaces, WorkNodes, GeneratorTasks |
| `workhub/repos/` | Repository pattern for DB access (sqlx queries) |
| `db/` | DB initialization + migration runner |
| `config/` | TOML config loading with env overrides |
| `events/` | `EventBridge` pushes events to frontend via Tauri app handle |
| `commands/` | Tauri IPC command handlers registered in `lib.rs` |
| `state/` | `AppState` holds db pool, config, and `AgentSupervisor` |
| `session/` | In-memory chat memory for agents |

## Agent Lifecycle

Goal status determines which agent runs (`supervisor.rs:determine_agent_type`):
- `Draft` → Summarizer
- `Pinned`/`Building`/`Failed` → Builder
- `Reached`/`Optimizing` → Optimizer

The `Executor` exists but is not wired in the status-to-agent mapping yet.

## Conventions

- Rust code contains Chinese comments alongside English ones
- Frontend uses Zustand stores per domain (goals, agents, chat, milestones, settings, noderepo)
- TypeScript strict mode is enabled — no unused locals or parameters allowed
- Tauri commands follow the pattern `{domain}_{action}` (e.g., `goals_create`, `config_get_llm`)
- The `dist/` directory and `src-tauri/target/` are gitignored; `node_modules/` is gitignored