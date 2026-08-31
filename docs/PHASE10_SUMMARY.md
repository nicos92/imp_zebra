# Phase 10 — Hardening Summary

## Date: 2026-08-31

## Status: COMPLETED ✅

## What was implemented

### 1. Structured logging (`tracing`)

- **`lib.rs`**: `tracing_subscriber::fmt().with_env_filter(...)` initialized in `run()`,
  driven by `RUST_LOG` with default `"info"`.
- **`print_labels.rs`**: `#[instrument(skip_all, fields(printer_id, quantity))]` + events
  `print job created` / `print job started` / `print job completed` / `print job failed`.
- **`configure_printer.rs` / `test_printer.rs`**: `#[instrument]` + `printer configured` /
  `printer updated` / `printer connection test ok` events.
- **`tcp_transport.rs`**: `#[instrument]` on `send`/`test_connection` + `warn!` on
  connect/write/flush timeouts and connection errors (address field, no secrets).
- **`sequence_service.rs`**: `#[instrument]` + `sequence range reserved` event.
- Repository-level error logging intentionally skipped to respect §40 — failures already
  propagate through the instrumented use-case envelope.

### 2. Max quantity validation + explicit concurrency test

- **`Sequence::MAX_PRINT_QUANTITY = 10_000`** in `sequence.rs`, enforced in `reserve_range`:
  `quantity > MAX_PRINT_QUANTITY` → `DomainError::QuantityTooLarge`; `quantity == 0` →
  `InvalidQuantity`. Enforced at the domain choke point, covering both service and use case.
- **`domain_error.rs`**: new variant `QuantityTooLarge { value, max }`.
- **`sqlite_sequence_repository.rs`**: new `test_concurrent_reserve_ranges_do_not_overlap`
  (8 tasks × 25 codes) asserting contiguous, non-overlapping ranges — exercises the atomic
  `BEGIN IMMEDIATE` reservation under concurrency.

### 3. `completed_at` consistency (migration `002_completed_at_check.sql`)

- Rebuilt `print_jobs` and added constraint
  `terminal_requires_completed_at` (`status NOT IN ('completed','failed') OR completed_at IS NOT NULL`).
- Backfilled any legacy **terminal** rows missing `completed_at` with `created_at`.
- **Design note:** a blanket `NOT NULL` was intentionally avoided — pending/printing jobs
  legitimately have no completion time, so the conditional `CHECK` enforces the real
  invariant ("terminal ⟹ has timestamp") without corrupting the lifecycle.
- `migrations.rs` test covers: terminal + NULL → rejected; pending + NULL → accepted;
  terminal + timestamp → accepted.

### 4. History detail panel (`get_print_job` wired)

- **`printingApi.ts`**: new `getPrintJob(jobId)` → `get_print_job`.
- **`HistoryView.vue`**: a `Detalle` button per row opens a detail panel (ID, printer,
  quantity, codes, status, created/completed) fetched on demand via `get_print_job`, with
  loading and error states.

### 5. Docs

- `docs/PHASE10_PLAN.md` / `docs/PHASE10_SUMMARY.md` added.
- `DEVELOPMENT.md` and `ARCHITECTURE.md` updated (Phase 10 ✅ + decisions).
- Kept the uncommitted `label_layout.rs` calibration change (barcode_height 150,
  title_font_size 10) per user decision — not reverted.

## Tests

| Suite | Antes | Después |
|-------|-------|---------|
| Backend (`cargo test`) | 94 | **98** |
| Frontend (`pnpm test`) | 63 | **66** |

New backend tests: 2 × `Sequence` (`test_reserve_range_quantity_too_large`,
`test_reserve_range_max_quantity_ok`), 1 × concurrency (`test_concurrent_reserve_ranges_do_not_overlap`),
1 × migration (`test_terminal_job_requires_completed_at`).
New frontend tests: 2 × `HistoryView` (detail open, detail error), 1 × `printingApi` (`getPrintJob`).

## Verification sequence

1. `cargo test` ✅ (98/98)
2. `cargo fmt --all` ✅ + `cargo clippy --all-targets` ✅ (15 pre-existing warnings, 0 new)
3. `pnpm test` ✅ (66/66)
4. `pnpm build` ✅ (vue-tsc + vite) + `pnpm lint` ✅ (0 issues)

## Out of scope (unchanged)

- Real-time Tauri `emit` progress events (local/fake `usePrintProgress` kept).
- Retry/queue for failed jobs.
- USB/Serial transports (§40).
