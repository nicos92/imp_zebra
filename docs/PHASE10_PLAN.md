# Phase 10 — Hardening Plan

## Date: 2026-08-31

## Problem

Phases 1–9 delivered the full stack (Vue → Tauri → Rust → SQLite → ZPL → TCP → Zebra)
with 94 backend + 63 frontend tests. Remaining hardening gaps from `prompt_maestro.md` §38
and the deferred list in `PHASE9_PLAN.md`:

1. **Structured logging** — `tracing`/`tracing-subscriber` declared in `Cargo.toml` but
   never initialized; no runtime visibility into print flow, printer config, or TCP failures.
2. **Max-quantity validation + concurrency** — print quantity was unbounded at the domain
   level, and the atomic range reservation had no explicit concurrency test.
3. **`completed_at` consistency** — terminal jobs (completed/failed) could persist with a
   NULL `completed_at`.
4. **History detail** — `get_print_job` command existed and was tested but had no UI.
5. **Documentation** — Phase 10 status not reflected in the docs.

## Decisions

1. **Structured logging** — initialize a `tracing_subscriber` env-filter subscriber
   (`RUST_LOG`, default `info`) in `lib.rs::run()`. Add `#[instrument]` + event/warn
   logging on the hot paths (print flow, printer config, connection test, sequence
   reservation, TCP transport). Failures are surfaced via use-case instrumentation and
   the "print job failed" event; **no** repository-level logging to respect §40
   (failures already propagate through the instrumented envelope).
2. **Max quantity + concurrency** — add `Sequence::MAX_PRINT_QUANTITY = 10_000` with a
   `QuantityTooLarge` guard in `reserve_range` (`0` → `InvalidQuantity`). Add an explicit
   concurrency test on `SqliteSequenceRepository::reserve_range` (8 tasks × 25 codes,
   asserting contiguous non-overlapping ranges). Domain-level limit so both the service
   and use-case paths are covered.
3. **`completed_at` consistency** — SQLite cannot add `NOT NULL`/`CHECK` via `ALTER`, so
   rebuild `print_jobs` in a new migration (`002_completed_at_check.sql`) that:
   - backfills any legacy **terminal** rows missing `completed_at` with `created_at`,
   - adds a `CHECK` constraint `terminal_requires_completed_at`
     (`status NOT IN ('completed','failed') OR completed_at IS NOT NULL`).
   A blanket `NOT NULL` is intentionally **not** used: pending/printing jobs legitimately
   have no completion time, so a conditional constraint enforces the real invariant
   without corrupting the lifecycle.
4. **History detail panel** — wire the existing `get_print_job` command into `HistoryView.vue`
   (a `Detalle` button per row opens a detail panel fetched on demand), with `getPrintJob`
   in `printingApi.ts` and matching tests.
5. **Docs** — `PHASE10_PLAN.md` + `PHASE10_SUMMARY.md`; mark Phase 10 ✅ and record the
   decisions in `DEVELOPMENT.md` / `ARCHITECTURE.md`.

## Out of scope

- Real-time Tauri `emit` progress events to Vue (stays local/fake `usePrintProgress`).
- Retry/queue for failed jobs.
- USB/Serial transports (§40).

## Files

| File | Change |
|------|--------|
| `src-tauri/src/lib.rs` | init `tracing_subscriber` env-filter in `run()` |
| `src-tauri/src/application/use_cases/print_labels.rs` | `#[instrument]` + events |
| `src-tauri/src/application/use_cases/configure_printer.rs` | `#[instrument]` + events |
| `src-tauri/src/application/use_cases/test_printer.rs` | `#[instrument]` + events |
| `src-tauri/src/infrastructure/printer/tcp_transport.rs` | `#[instrument]` + `warn!` timeouts/errors |
| `src-tauri/src/domain/services/sequence_service.rs` | `#[instrument]` + reserve event |
| `src-tauri/src/errors/domain_error.rs` | `+ QuantityTooLarge` variant |
| `src-tauri/src/domain/entities/sequence.rs` | `MAX_PRINT_QUANTITY` + guards + tests |
| `src-tauri/src/infrastructure/database/repositories/sqlite_sequence_repository.rs` | concurrency test |
| `src-tauri/migrations/002_completed_at_check.sql` | **New** — CHECK constraint + backfill |
| `src-tauri/src/infrastructure/database/migrations.rs` | + CHECK-constraint test |
| `src/infrastructure/tauri/printingApi.ts` | `+ getPrintJob` |
| `src/views/HistoryView.vue` | detail panel via `get_print_job` |
| `src/views/HistoryView.spec.ts` | + detail tests |
| `src/infrastructure/tauri/printingApi.spec.ts` | + `getPrintJob` test |
| `docs/PHASE10_PLAN.md` | **New** — this plan |
| `docs/PHASE10_SUMMARY.md` | **New** — results |
| `docs/DEVELOPMENT.md` | Phase 10 ✅ + summary |
| `docs/ARCHITECTURE.md` | logging init, max quantity, CHECK constraint, detail panel |

## Verification order

1. `cd src-tauri && cargo test`
2. `cargo fmt` + `cargo clippy --all-targets`
3. `pnpm test`
4. `pnpm build` + `pnpm lint`
