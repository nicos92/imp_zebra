# Phase 9 — Integration Summary

## Date: 2026-08-28

## Status: COMPLETED ✅

## Problem

The full chain (Vue → Tauri → Rust → SQLite → ZPL → TCP → Zebra) was implemented and
unit-tested layer by layer, but the layers had never been exercised together. Three
real defects were found and fixed:

1. **Critical wiring bug** (`src-tauri/src/lib.rs`): the SQLite pool (`AppState`) was
   created in `setup()` and discarded — never registered with `.manage()`. Every command
   requests `State<'_, AppState>`, so any real invocation failed at runtime with *"state
   not managed by the application"*. **Fixed**: `.manage(AppState::new(Arc::new(pool)))`
   inside `setup()`, with the pool error mapped into `Box<dyn Error>` (instead of `.expect`).
2. **Windows test crash** (`STATUS_ENTRYPOINT_NOT_FOUND 0xc0000139`): `cargo test` on the
   library binary failed to *start*. Root cause is the well-known Tauri issue
   ([#13419](https://github.com/tauri-apps/tauri/issues/13419)): test executables are not
   embedded with the Windows app manifest. **Fixed** in `src-tauri/build.rs`: switch to
   `tauri_build::try_build` with `WindowsAttributes::new_without_app_manifest()` and embed
   `windows-app-manifest.xml` into **every** artifact with `cargo:rustc-link-arg`
   (`/MANIFEST:EMBED` + `/MANIFESTINPUT`), so the test binary has the same
   `Microsoft.Windows.Common-Controls` v6 manifest as the production binary.
3. **Domain off-by-one bug** (`src-tauri/src/domain/services/sequence_service.rs`):
   `SequenceService::codes_for_range` treated the `start` code (the *first* code to use,
   e.g. `Z0000001`) as the last-used code, so `print_labels` generated barcodes
   `Z0000002..Z0000005` for quantity 4 while the job reported `Z0000001..Z0000004`.
   **Fixed**: derive the codes from `Sequence::new(parse(start) - 1).next_n(quantity)`.
   The integration test asserts `Z0000001` and `Z0000004` are present and `Z0000005` is not.
   The existing unit test was strengthened to assert the full code vector.

## What was implemented

### Backend — command-layer integration tests (10 new scenarios)

`src-tauri/src/commands/integration_tests.rs` + `#[cfg(test)] mod integration_tests;` in
`src-tauri/src/commands/mod.rs`. Uses Tauri's mock runtime (`tauri` feature `test`, added
only to `[dev-dependencies]`), **real** SQLite (in-memory pool via
`infrastructure/database/test_helpers::create_test_pool`) and **real** local TCP sockets
(`TcpListener` on `127.0.0.1:0`) acting as the printer.

| Scenario | Verifies |
|----------|----------|
| `save_and_read_printer_config` | save → get → list; `get_configured_printer` returns it |
| `get_configured_printer_is_none_when_empty` | fresh DB is empty |
| `test_printer_connection_against_local_listener` | real TCP connect OK |
| `test_printer_connection_refused_is_error` | `PRINTER_CONNECTION_FAILED` |
| `get_current_sequence_starts_at_first_code` | `Z0000000` → next `Z0000001` |
| `preview_label_generates_zpl_for_next_code` | ZPL `^XA`…`^XZ`, `^BC`, next code correct |
| `print_labels_sends_zpl_to_local_tcp_listener` | ZPL `^XA`…`^XZ`, `^BC`, codes in range, job `completed`, sequence advances |
| `print_labels_marks_job_failed_when_tcp_refused` | `PRINT_JOB_FAILED`, job `failed` with `completed_at` |
| `print_labels_with_missing_printer_is_error` | `PRINTER_NOT_CONFIGURED` |
| `list_print_jobs_is_empty_..._get_unknown_returns_none` | empty list; unknown job id → `None` |

### `mock_zebra` dev tool

`src-tauri/examples/mock_zebra.rs`: a TCP "Zebra" that binds a host/port (default
`127.0.0.1:9100`; pass `0` to let the OS pick a free port), reads the raw ZPL of each
connection to EOF and appends it to an output file (default `mock_zebra.zpl`) with a
`=== Zebra mock print #N — unix <ts> ===` separator. This is the manual E2E enabler.

### Frontend — view-level integration tests (13 new)

`src/views/DashboardView.spec.ts` (4), `PrinterSettingsView.spec.ts` (5),
`HistoryView.spec.ts` (4), reusing `vi.mock("@tauri-apps/api/core")` + real Pinia. Cover:
mount/load + preview, empty state + disabled print, print success + refresh + result,
backend error surfacing, save persistence + success/error message, persist-before-test and
test-without-save paths, history load/empty/error and reload button.

## Bug fixes

| Bug | File | Impact |
|-----|------|--------|
| `AppState` never managed | `src-tauri/src/lib.rs` | Command layer failed at runtime |
| Test binary missing Windows manifest | `src-tauri/build.rs` + `windows-app-manifest.xml` | `cargo test` could not start on Windows |
| Print codes off-by-one | `domain/services/sequence_service.rs` | Barcodes printed `start+1`… instead of `start`… |

## Verification

Backend (`src-tauri`):

1. `cargo test --no-run` ✅ (compiles; before manifest fix the test binary crashed on start)
2. `cargo test` ✅ **94/94** (lib: 94 + bin: 0)
3. `cargo fmt --all` ✅ (applied)
4. `cargo clippy --all-targets` ✅ (15 pre-existing dead-code warnings, none new)
5. `cargo build --example mock_zebra` ✅ + real TCP smoke test capturing ZPL bytes

Frontend:

6. `pnpm test` ✅ **63/63** (17 files; was 50)
7. `pnpm build` ✅
8. `pnpm lint` ✅ (no warnings)

## Files

| File | Change |
|------|--------|
| `src-tauri/src/lib.rs` | **Fix** — `.manage(AppState)` in `setup()`, propagate pool error |
| `src-tauri/build.rs` | **Fix** — embed Windows app manifest in all artifacts |
| `src-tauri/windows-app-manifest.xml` | **New** — app manifest (Common-Controls 6) |
| `src-tauri/Cargo.toml` | + `tauri { features = ["test"] }` in dev-deps |
| `src-tauri/src/commands/integration_tests.rs` | **New** — 9 command scenarios (SQLite + TCP) |
| `src-tauri/src/commands/mod.rs` | + `#[cfg(test)] mod integration_tests;` |
| `src-tauri/src/domain/services/sequence_service.rs` | **Fix** — `codes_for_range` off-by-one |
| `src-tauri/examples/mock_zebra.rs` | **New** — dev TCP "Zebra" |
| `src/views/DashboardView.spec.ts` | **New** |
| `src/views/PrinterSettingsView.spec.ts` | **New** |
| `src/views/HistoryView.spec.ts` | **New** |
| `docs/PHASE9_PLAN.md` | **New** — plan |
| `docs/PHASE9_SUMMARY.md` | **New** — this document |
| `docs/DEVELOPMENT.md` | §20 Phase 9 ✅ |

## Out of scope (Phase 10 — Hardening)

- Structured logging (`tracing` declared, unused) and richer error reporting.
- Tauri `emit` events → Vue progress; retries/queue.
- `completed_at NOT NULL` migration.
- `get_print_job` detail panel in `HistoryView` (command tested; UI deferred).
- USB/Serial transports.

---

## E2E Runbook (manual, no physical Zebra required)

Prereq: `pnpm` and `cargo` available; terminal at repo root `D:\Desarrollo\Rust\imp_zebra`.

1. **Start the mock printer** in terminal **A**:

   `cd src-tauri && cargo run --example mock_zebra -- 127.0.0.1 0`

   Note the printed port, e.g. `listening on 127.0.0.1:60822`.
   (Default port `9100` is used if no args; on this machine 9100 is held by Zebra Print
   Manager, so prefer `0`.)

2. **Start the app** in terminal **B**:

   `pnpm tauri dev`

3. **Configure the printer** (route `/settings` — "Configuración de impresora"):
   - Nombre / Modelo: any (e.g. `Mock`, `ZD421`)
   - Dirección IP: `127.0.0.1`, Puerto: the port from step 1 (e.g. `60822`)
   - **Probar conexión** → "Conexión exitosa con la impresora."

4. **Print labels** (route `/` — Dashboard):
   - The preview must refresh showing the next code (e.g. `Z0000001`).
   - Set a quantity (e.g. `4`) → **Imprimir** → "Impresión enviada" with codes
     `Z0000001 → Z0000004`.

5. **Inspect captured ZPL**:
   - Terminal **A** logs `job #1 captured N bytes -> mock_zebra.zpl`.
   - Check `src-tauri/mock_zebra.zpl`: must contain `^XA`, `^BC` (Code 128), the codes
     `Z0000001`…`Z0000004` and end with `^XZ`.

6. **History** (route `/history`): the job appears as **Completado** with the code range.

7. **Failure path**:
   - Stop terminal **A** (Ctrl+C on `mock_zebra`) and print again.
   - The Dashboard shows the backend error alert; History marks the job **Fallido**.

8. **Sequence continuity**: after printing, Dashboard's "Próximo código" advances
   (`Z0000005` after a 4-label job), matching `get_current_sequence`.