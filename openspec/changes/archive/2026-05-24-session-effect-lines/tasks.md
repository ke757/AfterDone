## 1. Core Data Types

- [x] 1.1 Define `EffectType` enum (Result, FileChange) with Serialize/Deserialize in `session/types.rs`
- [x] 1.2 Define `SessionLine` enum (Message, Effect) with serde tag="type" in `session/types.rs`
- [x] 1.3 Remove old `Message` struct from `session/types.rs`
- [x] 1.4 Delete `session/cell/result.rs` (StoredResult, ResultStatus retired)
- [x] 1.5 Update `session/mod.rs` re-exports (add SessionLine, EffectType; remove result::*)

## 2. Session Trait & Implementations

- [x] 2.1 Rewrite `Session` trait: add_line / get_lines / line_count / truncate / clear
- [x] 2.2 Add default impl for get_messages(), get_effects(), latest_effect() on Session trait
- [x] 2.3 Update `InMemorySession`: Vec<Message> → Vec<SessionLine>, implement new trait
- [x] 2.4 Update `JsonlSession`: Vec<Message> → Vec<SessionLine>, implement new trait with truncate (file rewrite)
- [x] 2.5 Update `JsonlSession::load()` to parse SessionLine enum variants from JSONL
- [x] 2.6 Update `JsonlSession::create()` to write SessionLine-compatible format
- [x] 2.7 Ensure SessionHeader (cell_meta) first line logic preserved in JsonlSession

## 3. SessionCell Trait & Concrete Cells

- [x] 3.1 Update `SessionCell` trait: remove set_result/get_result, add add_effect/latest_effect/redo_from
- [x] 3.2 Update `CellInfo`: message_count→line_count, add has_result/last_result_at, remove result_status
- [x] 3.3 Update `to_info()` default impl to use get_lines() for has_result/last_result_at
- [x] 3.4 Update `GoalSummaryCell`: remove result field, adapt add_message to SessionLine, impl add_effect/latest_effect
- [x] 3.5 Update `BuildCell`: same pattern as GoalSummaryCell
- [x] 3.6 Update `ExecutorCell`: same pattern
- [x] 3.7 Update `OptimizerCell`: same pattern

## 4. CellManager

- [x] 4.1 Update `load_all_cells()` to parse new JSONL format (SessionLine after CellMeta first line)
- [x] 4.2 Verify `create_cell()` path logic unchanged (only Session internals changed)

## 5. Agent Runtime & Handlers

- [x] 5.1 Update `ResultHandler` trait: split handle() into to_effect_content() → serde_json::Value + persist(db, output, goal_id, workspace_id)
- [x] 5.2 Remove `HandlerContext` from handlers mod.rs
- [x] 5.3 Update `SummarizerResultHandler`: implement new trait methods
- [x] 5.4 Update `BuilderResultHandler`: implement new trait methods
- [x] 5.5 Update `ExecutorResultHandler`: implement new trait methods
- [x] 5.6 Update `OptimizerResultHandler`: implement new trait methods
- [x] 5.7 Rewrite `AgentRuntime::handle_and_store()` as `handle_output()`: call to_effect_content → cell.add_effect(Result) → handler.persist()
- [x] 5.8 Update `run_turn()` and `run_task()` to call handle_output() instead of handle_and_store()
- [x] 5.9 Add FileChange effect recording at transport.execute() call points in AgentRuntime

## 6. Commands

- [x] 6.1 Update `commands/sessions.rs`: CellHistoryResponse returns Vec<SessionLine>
- [x] 6.2 Add `cell_redo` Tauri command (cell_id, at_index) → call cell.truncate(at_index)
- [x] 6.3 Update `commands/summarizer.rs`: summarizer_confirm reads latest_effect(Result) instead of get_result()
- [x] 6.4 Update `commands/summarizer.rs`: summarizer_status uses latest_effect(Result)
- [x] 6.5 Update `services/summarizer.rs`: extract_summary_from_messages takes &[SessionLine], filter Message variant

## 7. Frontend Types

- [x] 7.1 Update `CellInfo` interface in `src/services/commands.ts`: new fields, remove result_status
- [x] 7.2 Add `SessionLine` TypeScript type to `src/services/commands.ts`
- [x] 7.3 Update `cellCreate` signature (unchanged, but CellInfo return type updated)
- [x] 7.4 Update `cellGetHistory` return type to use SessionLine[]
- [x] 7.5 Update `src/types/chat.ts`: Message type replaced by SessionLine
- [x] 7.6 Update `src/stores/chatStore.ts`: messages→lines, filter SessionLine by type="message"

## 8. Build Verification

- [x] 8.1 Run `cargo check` in src-tauri, fix all errors
- [x] 8.2 Run `pnpm run build` (tsc + vite), fix all type errors
