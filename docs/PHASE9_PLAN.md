# Phase 9 — Integration Plan

## Date: 2026-08-28

## Problem

The full stack (Vue → Tauri → Rust → SQLite → ZPL → TCP → Zebra) is implemented and
unit-tested layer by layer, but the layers were never exercised together:

- **Critical wiring bug:** `AppState` (the SQLite pool) is created in `lib.rs` `setup()`
  and discarded — never registered with `.manage()`. All 9 commands request
  `State<'_, AppState>`, so any real invocation fails at runtime with
  *"state not managed by the application"*.
- **0 tests** on the Tauri command layer (the wiring between commands and state).
- Frontend has no view-level tests; only API/component/unit/smoke coverage.
- No way to verify the real chain without a physical Zebra.

## Decisions

1. **Fix the wiring first.** Register `AppState` via `.manage()` inside `setup()` and
   propagate the pool-creation error instead of `.expect()`.
2. **Backend integration tests on the command layer** (`src/commands/integration_tests.rs`),
   using Tauri's mock runtime (`tauri` feature `test`, added only to `[dev-dependencies]`):
   `tauri::test::mock_app()` + `app.manage(AppState)` + direct calls to the command fns.
   Real SQLite (in-memory pool via `test_helpers`) and a real local TCP listener
   (`TcpListener` on `127.0.0.1:0`) acting as the "printer".
3. **Frontend integration tests on the 3 views**, reusing the established
   `vi.mock("@tauri-apps/api/core")` pattern to simulate the Tauri bridge.
4. **`mock_zebra` dev tool** (`src-tauri/examples/mock_zebra.rs`): TCP listener that
   captures the ZPL bytes sent to port 9100, for a manual live E2E runbook without a
   physical Zebra.
5. Document the runbook and results in `docs/PHASE9_SUMMARY.md`.

## Out of scope (Phase 10 — Hardening)

- Structured logging (`tracing` already declared, unused).
- Tauri `emit` events → Vue progress, retries/queue, `completed_at` NOT NULL migration.
- `get_print_job` detail panel in HistoryView (command is tested; UI deferred).
- USB/Serial transports.

## Files

| File | Change |
|------|--------|
| `src-tauri/src/lib.rs` | **Fix** — `.manage(AppState)` in `setup()`, propagate pool error |
| `src-tauri/Cargo.toml` | + `[dev-dependencies] tauri = { version = "2", features = ["test"] }` |
| `src-tauri/src/commands/integration_tests.rs` | **New** — 9 commands against SQLite + local TCP |
| `src-tauri/src/commands/mod.rs` | + `#[cfg(test)] mod integration_tests;` |
| `src-tauri/examples/mock_zebra.rs` | **New** — dev TCP "Zebra" capturing ZPL |
| `src/views/DashboardView.spec.ts` | **New** — full dashboard flow |
| `src/views/PrinterSettingsView.spec.ts` | **New** — settings save/test flow |
| `src/views/HistoryView.spec.ts` | **New** — history load/render flow |
| `docs/PHASE9_PLAN.md` | **New** — this plan |
| `docs/PHASE9_SUMMARY.md` | **New** — results + E2E runbook |
| `docs/DEVELOPMENT.md` | Phase 9 ✅ + summary |

## Verification order

1. `cd src-tauri && cargo test`
2. `cargo fmt` + `cargo clippy`
3. `pnpm test`
4. `pnpm build` + `pnpm lint`
5. Manual runbook: `mock_zebra` + `pnpm tauri dev`