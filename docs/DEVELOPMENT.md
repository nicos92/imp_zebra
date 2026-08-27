# Development

## 1. Required Tools

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | 1.97+ | Backend language |
| Cargo | 1.97+ | Rust package manager |
| Node.js | 22+ | Frontend runtime |
| pnpm | 10+ | Frontend package manager |
| Tauri CLI | 2.11+ | Tauri build tooling |
| create-tauri-app | 4.7+ | Project scaffolding |

## 2. Project Creation

```bash
cargo create-tauri-app zebra-printer -m pnpm -t vue-ts --tauri-version 2
```

## 3. Commands

```bash
# Development (starts both Vite dev server and Tauri)
pnpm tauri dev

# Build production
pnpm tauri build

# Rust tests
cd src-tauri && cargo test

# Frontend tests
pnpm vitest

# Lint frontend
pnpm eslint src/

# Format Rust
cd src-tauri && cargo fmt

# Clippy (Rust linter)
cd src-tauri && cargo clippy
```

## 4. Folder Structure

See [ARCHITECTURE.md](./ARCHITECTURE.md) for the complete folder tree.

## 5. Database

SQLite database is stored in the platform-specific app data directory:
- Windows: `%APPDATA%/zebra-printer/zebra-printer.db`
- macOS: `~/Library/Application Support/zebra-printer/zebra-printer.db`
- Linux: `~/.local/share/zebra-printer/zebra-printer.db`

Migrations run automatically at app startup. See [DATABASE.md](./DATABASE.md).

## 6. Implementation Phases

### Phase 1 — Scaffolding ✅ COMPLETED
Create project, frontend, backend, folder structure, initial configuration.

### Phase 2 — Domain ✅ COMPLETED
Implement Barcode, Sequence, PrintJob, Printer config with tests.

### Phase 3 — SQLite ✅ COMPLETED
Database connection, migrations, repositories, transactions.

### Phase 4 — ZPL ✅ COMPLETED
ZplGenerator, LabelLayout, Code 128, two-column layout with tests.

### Phase 5 — Printer Transport ✅ COMPLETED
PrinterTransport trait, TcpPrinterTransport, connection testing.

### Phase 6 — Application Layer ✅ COMPLETED
Use cases: ConfigurePrinter, TestPrinterConnection, GetCurrentSequence, PreviewLabel, PrintLabels.

### Phase 7 — Tauri ✅ COMPLETED
Tauri commands as thin transport layer.

### Phase 8 — Vue
Printer Settings, Printing, Preview, History, Status views.

### Phase 9 — Integration
Full flow: Vue → Tauri → Rust → SQLite → ZPL → TCP → Zebra.

### Phase 10 — Hardening
Error handling, logging, recovery, concurrency, edge cases, tests.

## 7. Testing

```bash
# All Rust tests
cd src-tauri && cargo test

# Specific test
cd src-tauri && cargo test --lib domain::entities::sequence

# With output
cd src-tauri && cargo test -- --nocapture

# Frontend tests
pnpm vitest run

# Frontend tests in watch mode
pnpm vitest
```

## 8. Troubleshooting

| Problem | Solution |
|---------|---------|
| SQLite locked | Check `busy_timeout` in connection setup |
| TCP connection refused | Verify IP, port 9100, firewall rules |
| ZPL not printing | Verify orientation, dimensions, printer calibration |
| Sequence duplicated | Check transaction isolation, verify last_used_code update |
| Tauri build fails | Run `pnpm install`, check Node.js/Rust versions |
| SQLx compile error | Run `cargo sqlx prepare` or use `SQLX_OFFLINE=true` |

## 9. Production Build

```bash
pnpm tauri build
```

Output:
- Windows: `src-tauri/target/release/bundle/msi/`
- macOS: `src-tauri/target/release/bundle/dmg/`
- Linux: `src-tauri/target/release/bundle/deb/`

## 10. Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level (trace, debug, info, warn, error) |
| `SQLX_OFFLINE` | `false` | Use offline SQLx data for compilation |

## 11. Phase 1 Implementation Summary

### Date: 2026-08-26

### What was implemented

**Documentation (docs/):**
- ARCHITECTURE.md - System diagram, folder structure, dependency decisions
- DATABASE.md - Schema, migrations, concurrent access strategy
- PRINTING.md - Flow, states, consistency guarantees
- ZPL.md - Commands, layout, distribution
- DEVELOPMENT.md - Environment, commands, phases

**Backend Rust (src-tauri/src/):**

| Layer | Files | Status |
|-------|-------|--------|
| Domain/Entities | `sequence.rs`, `printer.rs`, `print_job.rs` | ✅ |
| Domain/ValueObjects | `barcode.rs`, `printer_config.rs` | ✅ |
| Domain/Repositories | `sequence_repository.rs`, `printer_repository.rs`, `print_job_repository.rs` (traits) | ✅ |
| Domain/Services | `sequence_service.rs`, `label_service.rs` | ✅ |
| Application/DTO | `printer_dto.rs`, `print_dto.rs` | ✅ |
| Application/UseCases | `configure_printer.rs`, `get_printer_config.rs`, `test_printer.rs`, `get_current_sequence.rs`, `get_configured_printer.rs`, `list_print_jobs.rs`, `preview_label.rs`, `print_labels.rs` | ✅ |
| Infrastructure/Database | `connection.rs`, `migrations.rs`, `sqlite_sequence_repository.rs`, `sqlite_printer_repository.rs`, `sqlite_print_job_repository.rs` | ✅ |
| Infrastructure/Printer | `printer_transport.rs`, `tcp_transport.rs` | ✅ |
| Infrastructure/ZPL | `generator.rs`, `label_layout.rs` | ✅ |
| Commands | `printer_commands.rs`, `print_commands.rs` | ✅ |
| Errors | `domain_error.rs`, `application_error.rs`, `infrastructure_error.rs` | ✅ |
| State | `app_state.rs` | ✅ |
| Entry points | `main.rs`, `lib.rs` | ✅ |

**Migrations:**
- `001_initial.sql` - sequence_state, printers, print_jobs tables

**Frontend:**
- `package.json` - Dependencies configured
- `vite.config.ts` - Tauri dev server config
- `tsconfig.json` - TypeScript config

### Tests (34 passing)

| Module | Tests | Status |
|--------|-------|--------|
| `domain::entities::sequence` | 12 tests (new, from_code, next, next_n, reserve_range, rollover, to_code, parse_code, display) | ✅ |
| `domain::value_objects::barcode` | 5 tests (valid, invalid prefix, invalid chars, too short, too long) | ✅ |
| `domain::value_objects::printer_config` | 7 tests (valid, invalid dpi/width/columns/port, dimensions_in_dots, connection_type) | ✅ |
| `domain::services::label_service` | 2 tests (generate_codes, calculate_positions) | ✅ |
| `infrastructure::zpl::generator` | 3 tests (single label, two labels, empty batch) | ✅ |
| `infrastructure::zpl::label_layout` | 4 tests (default, from_printer_config, total_width, label_positions) | ✅ |

### Compilation

```
cargo check: OK (18 warnings - unused code, expected in early phases)
cargo test: 34/34 passing
```

### Dependencies added

**Rust:**
- tauri 2, tokio 1, serde/serde_json 1, sqlx 0.8, chrono 0.4, thiserror 2
- tracing 0.1, tracing-subscriber 0.3, uuid 1, async-trait 0.1

**Frontend:**
- vue ^3.5, vue-router ^4.5, @tauri-apps/api ^2
- typescript ~5.7, vite ^8.0, vitest ^3.0

### Key decisions verified

1. `last_used_code` storage works correctly with rollover
2. Range reservation produces correct sequences
3. ZPL generator produces valid ZPL II with Code 128
4. Label layout calculates correct dot positions for 203 DPI
5. Error types propagate correctly through layers

## 12. Phase 2 Implementation Summary

### Date: 2026-08-26

### Status: COMPLETED ✅

### What was implemented

**Architectural Fixes:**
- Added `InvalidStateTransition` and `InvalidPrintJobStatus` error variants to `DomainError`
- Added state transition guards to `PrintJob` entity (Pending→Printing→Completed/Failed)
- Fixed `PrintJobStatus::from_str` to return `Result` instead of `Option`
- Added IPv4 format validation to `PrinterConfig`
- Updated `sqlite_print_job_repository` to handle new `Result` type from `from_str`

### Tests Added (53 total, +19 from Phase 1)

| Module | Tests | Status |
|--------|-------|--------|
| `domain::entities::sequence` | 12 tests | ✅ |
| `domain::value_objects::barcode` | 5 tests | ✅ |
| `domain::value_objects::printer_config` | 10 tests (+2 IP validation) | ✅ |
| `domain::services::label_service` | 2 tests | ✅ |
| `domain::services::sequence_service` | 3 tests (+3 new) | ✅ |
| `domain::entities::printer` | 4 tests (+4 new) | ✅ |
| `domain::entities::print_job` | 10 tests (+10 new) | ✅ |
| `infrastructure::zpl::generator` | 3 tests | ✅ |
| `infrastructure::zpl::label_layout` | 4 tests | ✅ |
| **Total** | **53 tests** | **All passing** |

### Compilation

```
cargo check: OK
cargo test: 53/53 passing
```

### New Error Variants

```rust
#[error("Invalid state transition from {from} to {to}")]
InvalidStateTransition { from: String, to: String },

#[error("Invalid print job status: {0}")]
InvalidPrintJobStatus(String),
```

### Key decisions verified

1. State transitions are properly guarded (Pending→Printing→Completed/Failed)
2. Invalid transitions return descriptive errors
3. IPv4 format validation prevents invalid IP addresses
4. SequenceService correctly persists changes to repository
5. All entity methods return proper error types

## 13. Phase 3 Implementation Summary

### Date: 2026-08-26

### Status: COMPLETED ✅

### What was implemented

**Database / Migrations:**
- Replaced hardcoded DDL in `migrations.rs` with SQLx migration runner `sqlx::migrate!("./migrations")`
- Migration `.sql` file (`001_initial.sql`) now embedded at compile-time and executed at startup
- Added `test_helpers.rs` — `create_test_pool()` for in-memory SQLite integration tests

**Transactions (critical concurrency fix):**
- Added `reserve_range()` to `SequenceRepository` trait
- Implemented atomic sequence reservation in `sqlite_sequence_repository.rs` using `BEGIN IMMEDIATE`
- SELECT → calculate → UPDATE happens inside a single transaction, preventing duplicate code ranges under concurrency
- Updated `SequenceService::reserve_range()` to delegate atomicity to the repository

### Tests Added (67 total, +14 from Phase 2)

| Module | Tests | Status |
|--------|-------|--------|
| All Phase 1 + 2 modules | 53 tests | ✅ |
| `infrastructure::database::migrations` | 1 test (migrations on empty DB) | ✅ |
| `infrastructure::database::repositories::sqlite_sequence_repository` | 4 tests (get_initial, update, reserve, reserve_sequential) | ✅ |
| `infrastructure::database::repositories::sqlite_printer_repository` | 5 tests (save+find, find_all, update, delete, find_nonexistent) | ✅ |
| `infrastructure::database::repositories::sqlite_print_job_repository` | 4 tests (save+find, update_status, find_recent, find_recent_empty) | ✅ |
| **Total** | **67 tests** | **All passing** |

### Compilation

```
cargo check: OK (24 warnings - unused code, expected in early phases)
cargo test: 67/67 passing
```

### Files Modified/Created

| File | Change |
|------|--------|
| `docs/PHASE3_PLAN.md` | **New** — implementation plan |
| `src-tauri/src/infrastructure/database/migrations.rs` | `sqlx::migrate!()` replaces hardcoded DDL + test |
| `src-tauri/src/infrastructure/database/test_helpers.rs` | **New** — `create_test_pool()` |
| `src-tauri/src/infrastructure/database/mod.rs` | Added `test_helpers` module |
| `src-tauri/src/domain/repositories/sequence_repository.rs` | Added `reserve_range()` to trait |
| `src-tauri/src/infrastructure/database/repositories/sqlite_sequence_repository.rs` | Implemented atomic `reserve_range()` + 4 tests |
| `src-tauri/src/infrastructure/database/repositories/sqlite_printer_repository.rs` | Added 5 tests |
| `src-tauri/src/infrastructure/database/repositories/sqlite_print_job_repository.rs` | Added 4 tests |
| `src-tauri/src/domain/services/sequence_service.rs` | Delegate reserve atomicity to repo + updated fake |

### Key decisions verified

1. `sqlx::migrate!()` correctly embeds migration SQL at compile-time (path `./migrations`)
2. `BEGIN IMMEDIATE` transaction ensures atomic sequence reservation
3. In-memory SQLite (`sqlite::memory:`) with `max_connections(1)` works for integration tests
4. Foreign key constraints enforce printer_id references
5. Print job tests require creating a valid printer first (FK)
6. Migration table seed produces `Z0000000` initial code

## 14. Phase 4 Implementation Summary

### Date: 2026-08-27

### Status: COMPLETED ✅

### What was implemented

**`LabelPosition` struct (align with docs/ZPL.md):**
- New typed `LabelPosition { row, column }` in `label_layout.rs` replacing raw `(String, u32, u32)` tuples
- `ZplGenerator::generate_batch` signature now `&[(String, LabelPosition)]`
- `LabelService::calculate_positions` returns `Vec<(String, LabelPosition)>`
- Eliminated inline position-calculation duplication in `PrintLabels` (now uses `LabelService`)
- `PreviewLabel` uses `LabelPosition { row: 0, column: 0 }`

**No Code 128 Rust encoder:** ZPL `^BC` encodes Code 128 natively on the printer; `^BCN` + `^BY2` retained.

### Tests Added (68 total, +1 from Phase 3)

| Module | Tests | Status |
|--------|-------|--------|
| All Phase 1-3 modules | 67 tests | ✅ |
| `infrastructure::zpl::generator` | +1 (`test_generate_odd_quantity_no_phantom`) | ✅ |
| **Total** | **68 tests** | **All passing** |

### Compilation

```
cargo test:  68/68 passing
cargo check: OK (21 warnings, all pre-existing unused-code)
cargo fmt:   clean
cargo clippy: no new warnings
```

### Files Modified/Created

| File | Change |
|------|--------|
| `docs/PHASE4_PLAN.md` | **New** — implementation plan |
| `src/infrastructure/zpl/label_layout.rs` | +`LabelPosition` struct |
| `src/infrastructure/zpl/generator.rs` | signature `&[(String, LabelPosition)]` + tests |
| `src/domain/services/label_service.rs` | return `Vec<(String, LabelPosition)>` + test |
| `src/application/use_cases/print_labels.rs` | use `LabelService::calculate_positions` |
| `src/application/use_cases/preview_label.rs` | use `LabelPosition` |
| `docs/DEVELOPMENT.md` | this summary + phase marked complete |
| `docs/ZPL.md` | interface already documents `LabelPosition` (verified aligned) |

### Key decisions verified

1. `LabelPosition` aligns the code with the documented interface in `docs/ZPL.md` §9
2. Distribution rule (odd→left, even→right) centralized in `LabelService`
3. Odd quantities produce no phantom ZPL for the empty position
4. Refactor produces identical ZPL output; no behavior change

## 15. Phase 5 Implementation Summary

### Date: 2026-08-27

### Status: COMPLETED ✅

### What was implemented

**`PrinterTransport` trait (dependency inversion):**
- New `infrastructure/printer/printer_transport.rs` with async trait:
  ```rust
  #[async_trait]
  pub trait PrinterTransport: Send + Sync {
      async fn send(&self, data: &[u8]) -> Result<(), InfrastructureError>;
      async fn test_connection(&self) -> Result<(), InfrastructureError>;
  }
  ```
- Reuses `InfrastructureError` (no new `PrinterError` — no overengineering rule §40)
- Enables testing without a physical Zebra and swapping TCP→USB/Serial later (acceptance #19, #20)

**`TcpPrinterTransport` (rename of `TcpTransport`):**
- Implements `PrinterTransport` with TCP socket (connect + write_all + flush) with timeouts
- `new(ip, port)` defaults: 5s connect, 30s write
- `new_with_timeouts(...)` added for testability
- Error mapping: connect/write errors → `PrinterConnection`, timeouts → `PrinterTimeout`

**`ZebraPrinter` now depends on `Arc<dyn PrinterTransport>`:**
- `new(printer)` builds a `TcpPrinterTransport` by default
- `with_transport` (test-only) injects a `FakePrinterTransport`
- `send_zpl` / `test_connection` delegate through the trait

**Consumer updates:**
- `TestPrinter` imports `TcpPrinterTransport` (rename) + `PrinterTransport` trait

### Tests Added (74 total, +6 from Phase 4)

| Module | New Tests | Status |
|--------|-----------|--------|
| All Phase 1-4 modules | 68 tests | ✅ |
| `tcp_transport.rs` | `test_send_to_listener`, `test_connection_refused`, `test_timeout_mapping` | ✅ |
| `zebra_printer.rs` | `test_send_zpl_ok`, `test_send_zpl_error`, `test_test_connection_ok` | ✅ |
| **Total** | **74 tests** | **All passing** |

### Verification sequence (as required)

1. `cargo check` ✅
2. `cargo fmt` + `cargo clippy` ✅ (no errors)
3. `cargo test` ✅ (74/74, 2.04s)
4. `cargo check` (repeat) ✅
5. `cargo fmt` + `cargo clippy` (repeat) ✅
6. No new warnings beyond the pre-existing unused-code set

### Files Modified/Created

| File | Change |
|------|--------|
| `docs/PHASE5_PLAN.md` | **New** — implementation plan |
| `src/infrastructure/printer/printer_transport.rs` | **New** — `PrinterTransport` trait |
| `src/infrastructure/printer/tcp_transport.rs` | rename `TcpPrinterTransport`, impl trait, `new_with_timeouts`, +tests |
| `src/infrastructure/printer/zebra_printer.rs` | `Arc<dyn PrinterTransport>`, `with_transport`, fake tests |
| `src/infrastructure/printer/mod.rs` | register `printer_transport` |
| `src/application/use_cases/test_printer.rs` | rename `TcpTransport`→`TcpPrinterTransport` + trait import |
| `docs/DEVELOPMENT.md` | this summary + phase marked complete |

### Key decisions verified

1. `PrinterTransport` trait established the abstraction boundary in infrastructure (§13/§43)
2. Reused `InfrastructureError` instead of adding a `PrinterError` layer (§40 no overengineering)
3. `test_connection` retained over `is_connected(): bool` (more informative, aligned with `TestPrinter`)
4. Only the application use-case DI was deferred to Phase 6 (per scope decision); infrastructure + `ZebraPrinter` fully abstracted now

---

## 16. Phase 6 Implementation Summary

### Date: 2026-08-27

### Status: COMPLETED ✅

### What was implemented

**Dependency Inversion en los casos de uso de impresora (testabilidad, criterio #19):**
- `TestPrinter` y `PrintLabels` ahora reciben `Arc<dyn PrinterTransport>` inyectado por constructor, en lugar de construir concretos de infraestructura (`TcpPrinterTransport`, `ZebraPrinter`) inline.
- La composición (construir el `TcpPrinterTransport` concreto desde la config de impresora) se realiza en la capa de **commands** (composition root), manteniendo la capa de aplicación libre de infraestructura concreta (§43).
- **`ZebraPrinter` eliminado** (se volvió código muerto en producción tras el refactor DI; §40): los casos de uso usan `transport.send()` directamente. Su cobertura quedó superada por los tests de `TestPrinter`/`PrintLabels` con `FakePrinterTransport`.

**`GetCurrentSequence` — nuevo caso de uso:**
- `get_current_sequence.rs` devuelve `SequenceInfoDto { last_used_code, next_code }` vía `SequenceService` (computa `next_code` sin persistir). Sirve al Dashboard "Próximo código" (criterio #4).

**Corrección de correctitud:**
- Los 3 `Result` ignorados de `job.start_printing()/complete()/fail()` en `PrintLabels` ahora se propagan con `?` (elimina los warnings `unused Result`).

**Commands:**
- `test_printer_connection`: carga impresora y construye el transporte antes de invocar `TestPrinter`.
- `print_labels`: carga impresora y construye el transporte antes de invocar `PrintLabels`.
- **`get_current_sequence`** command nuevo + registrado en `lib.rs`.

### Tests Added

| Módulo | Nuevos tests | Resultado |
|--------|--------------|-----------|
| `get_current_sequence.rs` | `test_get_current_sequence_returns_last_and_next`, `test_get_current_sequence_rollover_next` | ✅ |
| `test_printer.rs` | `test_connection_ok`, `test_connection_fails`, `test_printer_not_configured` | ✅ |
| `print_labels.rs` | `test_print_labels_happy_path`, `test_print_labels_send_failure_marks_failed`, `test_print_labels_missing_printer` | ✅ |
| Removidos | `zebra_printer.rs` (3 tests del facade eliminado) | — |
| **Total** | **79 tests** (74 previos + 8 − 3) | **All passing** |

### Verification sequence (as required)

1. `cargo check` ✅
2. `cargo fmt` + `cargo clippy` ✅ (sin errores)
3. `cargo test` ✅ (79/79, ~2s)
4. `cargo check` (repeat) ✅
5. `cargo fmt` + `cargo clippy` (repeat) ✅
6. Sin warnings nuevos: se eliminaron los 3 `unused Result` y el código muerto de `ZebraPrinter`; el resto son warnings dead-code pre-existentes de fases anteriores.

### Files Modified/Created

| File | Change |
|------|--------|
| `docs/PHASE6_PLAN.md` | **New** — implementation plan |
| `src/application/use_cases/get_current_sequence.rs` | **New** — caso de uso + tests |
| `src/application/use_cases/test_printer.rs` | DI `Arc<dyn PrinterTransport>` + tests |
| `src/application/use_cases/print_labels.rs` | DI transporte + propagar `Result` + tests |
| `src/application/use_cases/mod.rs` | +`get_current_sequence` |
| `src/commands/printer_commands.rs` | componer transporte; +`get_current_sequence` |
| `src/commands/print_commands.rs` | componer transporte en `print_labels` |
| `src/lib.rs` | registrar `get_current_sequence` |
| `src/infrastructure/printer/zebra_printer.rs` | **Eliminado** (código muerto post-DI, §40) |
| `src/infrastructure/printer/mod.rs` | quitar `zebra_printer` |
| `docs/DEVELOPMENT.md` | this summary + phase marked complete |

### Key decisions verified

1. `Arc<dyn PrinterTransport>` inyectado en `TestPrinter`/`PrintLabels` → testeables con `FakePrinterTransport` sin Zebra física (criterio #19)
2. Composición por-command (composition root) mantiene la capa de aplicación libre de infraestructura concreta (§43)
3. `ZebraPrinter` eliminado por quedar redundante tras el refactor (§40 no sobreingeniería)
4. `GetCurrentSequence` añadido y cableado end-to-end (criterio #4)

---

## 18. Phase 7 Implementation Summary

### Date: 2026-08-27

### Status: COMPLETED ✅

### What was implemented

La capa de commands Tauri (transporte delgado entre Vue y la app, §17) ya estaba
mayoritariamente construida. Esta fase cerró los 3 gaps reales restantes:

**`ListPrintJobs` — nuevo caso de uso + comando `list_print_jobs`:**
- Envuelve `PrintJobRepository::find_recent(limit)` y devuelve `Vec<PrintJobDto>`.
- Alimenta la vista Historial (§32) con: fecha, cantidad, código inicial/final, estado, impresora.
- Límite saneado con `clamp(1..500)`, default 50.

**`GetConfiguredPrinter` — nuevo caso de uso + comando `get_configured_printer`:**
- Devuelve `Option<PrinterDto>` de la primera impresora de `find_all()`.
- Resuelve la ergonomía de descubrimiento: el dashboard (§32) ahora puede mostrar la impresora
  configurada al arrancar sin conocer el `id` (UX de impresora única). Se mantiene `get_printer_config(id)`.

**Mapper compartido `From<PrintJob> for PrintJobDto`:**
- Extraído a `application/dto/print_dto.rs` y reutilizado por `get_print_job` y `ListPrintJobs`
  → elimina duplicación de mapeo inline.

**Commands totales registrados en `lib.rs` (9):**
`get_printer_config`, `save_printer_config`, `test_printer_connection`, `get_current_sequence`,
`get_configured_printer`, `print_labels`, `preview_label`, `get_print_job`, `list_print_jobs`.

### Tests Added

| Módulo | Nuevos tests | Resultado |
|--------|--------------|-----------|
| `list_print_jobs.rs` | `test_list_print_jobs_maps_entities`, `test_list_print_jobs_empty`, `test_list_print_jobs_default_limit_when_none` | ✅ |
| `get_configured_printer.rs` | `test_get_configured_printer_returns_first`, `test_get_configured_printer_none_when_empty` | ✅ |
| **Total** | **84 tests** (79 + 5) | **All passing** |

### Verification sequence (as required)

1. `cargo check` ✅
2. `cargo fmt` + `cargo clippy` ✅ (sin errores; sin warnings nuevos)
3. `cargo test` ✅ (84/84, ~2s)
4. `cargo check` (repeat) ✅
5. `cargo fmt` + `cargo clippy` (repeat) ✅
6. Sin warnings nuevos: `find_recent` ya no aparece como no usado (lo consume `ListPrintJobs`);
   el resto son warnings dead-code pre-existentes de fases anteriores.

### Files Modified/Created

| File | Change |
|------|--------|
| `docs/PHASE7_PLAN.md` | **New** — plan de la fase |
| `src/application/use_cases/list_print_jobs.rs` | **New** — caso de uso + 3 tests |
| `src/application/use_cases/get_configured_printer.rs` | **New** — caso de uso + 2 tests |
| `src/application/dto/print_dto.rs` | + `impl From<PrintJob> for PrintJobDto` |
| `src/application/use_cases/mod.rs` | + `list_print_jobs`, `+ get_configured_printer` |
| `src/commands/printer_commands.rs` | + `get_configured_printer` command |
| `src/commands/print_commands.rs` | `get_print_job` usa `From`; + `list_print_jobs` command |
| `src/lib.rs` | registrar `get_configured_printer`, `list_print_jobs` |
| `docs/DEVELOPMENT.md` | this summary + phase marked complete |

### Key decisions verified

1. `list_print_jobs` + `get_configured_printer` pasan por casos de uso (commands delgados, §17)
2. Composición por-command mantenida (cada command constituye sus repos, §43)
3. Mapper `From<PrintJob>` deduplica y centraliza el mapeo a DTO (§40, §17)
4. Mismatch resuelto con coerciones `as` de `Arc` concretos a trait object en tests con fakes
