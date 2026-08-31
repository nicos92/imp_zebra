# Architecture

## 1. Overview

Desktop application for Zebra thermal printer label management. Tauri 2 (Rust backend) + Vue 3 + TypeScript (frontend) + SQLite + ZPL II.

## 2. Architecture Style

Clean Architecture with Domain-Driven Design. The dependency rule is strictly enforced:

```
Presentation (Vue)
    ↓
Application (Use Cases)
    ↓
Domain (Entities, Value Objects, Traits)
    ↑
Infrastructure (SQLite, TCP, ZPL)
```

Infrastructure depends on abstractions defined by Domain/Application. Domain never depends on anything external.

## 3. System Diagram

```
┌─────────────────────────────────────────────────────────┐
│                      Vue 3 Frontend                      │
│  ┌──────────┐  ┌──────────┐  ┌───────────────────────┐  │
│  │ Printer  │  │ Dashboard│  │     History           │  │
│  │ Settings │  │ (status, │  │   (list_print_jobs)   │  │
│  │          │  │  print,  │  │                       │  │
│  │          │  │ preview) │  │                       │  │
│  └────┬─────┘  └────┬─────┘  └───────────┬───────────┘  │
│       │              │                    │              │
│  ┌────▼──────────────▼────────────────────▼───────────┐  │
│  │                    Pinia Store                      │  │
│  │               (stores/printer.ts)                   │  │
│  └─────────────────────────┬───────────────────────────┘  │
│  ┌─────────────────────────▼───────────────────────────┐  │
│  │              Infrastructure/Tauri                   │  │
│  │   (tauriClient.ts, printerApi.ts, printingApi.ts)   │  │
│  └──────────────────────┬─────────────────────────────┘  │
└─────────────────────────┼───────────────────────────────┘
                          │ invoke()
┌─────────────────────────▼───────────────────────────────┐
│                  Tauri Commands (Rust)                    │
│  ┌──────────────────┐  ┌──────────────────────────────┐  │
│  │ printer_commands │  │       print_commands         │  │
│  │   get_config     │  │     print_labels             │  │
│  │   save_config    │  │     preview_label            │  │
│  │   test_conn      │  │     get_print_job            │  │
│  └────────┬─────────┘  └────────────┬─────────────────┘  │
└───────────┼─────────────────────────┼────────────────────┘
            │                         │
┌───────────▼─────────────────────────▼────────────────────┐
│                  Application Layer                        │
│  ┌────────────────────┐  ┌───────────────────────────┐   │
│  │ ConfigurePrinter   │  │     PrintLabels           │   │
│  │ GetPrinterConfig   │  │     PreviewLabel          │   │
│  │ TestPrinter        │  │     GetCurrentSequence    │   │
│  └────────┬───────────┘  └────────────┬──────────────┘   │
└───────────┼───────────────────────────┼──────────────────┘
            │                           │
┌───────────▼───────────────────────────▼──────────────────┐
│                    Domain Layer                           │
│  ┌────────────┐  ┌────────────┐  ┌────────────────────┐  │
│  │  Barcode   │  │  Sequence  │  │   PrintJob         │  │
│  │ (VO)       │  │ (Entity)   │  │   (Entity)         │  │
│  └────────────┘  └────────────┘  └────────────────────┘  │
│  ┌────────────────────────────────────────────────────┐  │
│  │            SequenceService / LabelService          │  │
│  │        (use repository traits, not impls)          │  │
│  └────────────────────────────────────────────────────┘  │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────────┐   │
│  │Sequence  │  │Printer   │  │  PrintJob            │   │
│  │Repo Trait│  │Repo Trait│  │  Repo Trait          │   │
│  └──────────┘  └──────────┘  └──────────────────────┘   │
└──────────────────────────────────────────────────────────┘
            │               │               │
┌───────────▼───────────────▼───────────────▼──────────────┐
│                Infrastructure Layer                       │
│  ┌────────────────────────────────────────────────────┐  │
│  │              SQLite (SQLx)                         │  │
│  │  ┌────────────┐ ┌──────────┐ ┌─────────────────┐  │  │
│  │  │ Sequence   │ │ Printer  │ │   PrintJob      │  │  │
│  │  │ Repo Impl  │ │ Repo Impl│ │   Repo Impl     │  │  │
│  │  └────────────┘ └──────────┘ └─────────────────┘  │  │
│  └────────────────────────────────────────────────────┘  │
│  ┌──────────────────────┐  ┌──────────────────────────┐  │
│  │  TCP Transport       │  │    ZPL Generator         │  │
│  │  (tokio::net)        │  │  (ZplGenerator)          │  │
│  │  implements          │  │  (LabelLayout)            │  │
│  │  PrinterTransport    │  │                           │  │
│  └──────────────────────┘  └──────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
            │
            ▼
   Zebra Printer (TCP:9100)
```

## 4. Folder Structure

```
zebra-printer/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   │
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── printer_commands.rs
│   │   │   └── print_commands.rs
│   │   │
│   │   ├── domain/
│   │   │   ├── mod.rs
│   │   │   ├── entities/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── printer.rs
│   │   │   │   ├── print_job.rs
│   │   │   │   └── sequence.rs
│   │   │   ├── value_objects/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── barcode.rs
│   │   │   │   └── printer_config.rs
│   │   │   ├── repositories/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── sequence_repository.rs
│   │   │   │   ├── printer_repository.rs
│   │   │   │   └── print_job_repository.rs
│   │   │   └── services/
│   │   │       ├── mod.rs
│   │   │       ├── sequence_service.rs
│   │   │       └── label_service.rs
│   │   │
│   │   ├── application/
│   │   │   ├── mod.rs
│   │   │   ├── dto/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── printer_dto.rs
│   │   │   │   └── print_dto.rs
│   │   │   └── use_cases/
│   │   │       ├── mod.rs
│   │   │       ├── configure_printer.rs
│   │   │       ├── get_printer_config.rs
│   │   │       ├── test_printer.rs
│   │   │       ├── preview_label.rs
│   │   │       └── print_labels.rs
│   │   │
│   │   ├── infrastructure/
│   │   │   ├── mod.rs
│   │   │   ├── database/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── connection.rs
│   │   │   │   ├── migrations.rs
│   │   │   │   └── repositories/
│   │   │   │       ├── mod.rs
│   │   │   │       ├── sqlite_sequence_repository.rs
│   │   │   │       ├── sqlite_printer_repository.rs
│   │   │   │       └── sqlite_print_job_repository.rs
│   │   │   ├── printer/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── printer_transport.rs
│   │   │   │   └── tcp_transport.rs
│   │   │   └── zpl/
│   │   │       ├── mod.rs
│   │   │       ├── generator.rs
│   │   │       └── label_layout.rs
│   │   │
│   │   ├── errors/
│   │   │   ├── mod.rs
│   │   │   ├── domain_error.rs
│   │   │   ├── application_error.rs
│   │   │   └── infrastructure_error.rs
│   │   │
│   │   └── state/
│   │       ├── mod.rs
│   │       └── app_state.rs
│   │
│   ├── migrations/
│   │   ├── 001_initial.sql
│   │   └── 002_completed_at_check.sql
│   │
│   └── Cargo.toml
│
├── src/
│   ├── main.ts
│   ├── App.vue
│   ├── components/
│   │   ├── printer/
│   │   │   ├── PrinterForm.vue
│   │   │   └── PrinterStatus.vue
│   │   ├── printing/
│   │   │   ├── PrintQuantityForm.vue
│   │   │   ├── PrintProgress.vue
│   │   │   ├── PrintResult.vue
│   │   │   └── LabelPreview.vue
│   │   └── common/
│   │       ├── AppButton.vue
│   │       ├── AppInput.vue
│   │       └── AppModal.vue
│   ├── composables/
│   │   └── usePrintProgress.ts
│   ├── infrastructure/
│   │   └── tauri/
│   │       ├── tauriClient.ts
│   │       ├── printerApi.ts
│   │       └── printingApi.ts
│   ├── router/
│   │   └── index.ts
│   ├── stores/
│   │   └── printer.ts
│   ├── styles/
│   │   └── main.css
│   ├── types/
│   │   └── index.ts
│   ├── utils/
│   │   └── format.ts
│   └── views/
│       ├── DashboardView.vue
│       ├── HistoryView.vue
│       └── PrinterSettingsView.vue
│
├── package.json
├── tsconfig.json
├── vite.config.ts
├── vitest.config.ts
└── README.md
```

## 5. File Responsibilities

### Domain Layer

| File | Responsibility | Allowed Dependencies | Forbidden Dependencies |
|------|---------------|---------------------|----------------------|
| `entities/sequence.rs` | Sequence increment, validation, range generation | None | Everything external |
| `entities/printer.rs` | Printer entity, configuration model | Value objects | Infrastructure |
| `entities/print_job.rs` | Print job entity, status tracking | Domain entities | Infrastructure |
| `value_objects/barcode.rs` | Barcode format validation | None | Everything |
| `value_objects/printer_config.rs` | Printer config validation (IP, port, DPI) | None | Everything |
| `repositories/sequence_repository.rs` | Trait for sequence persistence | Domain entities | SQLite, TCP |
| `repositories/printer_repository.rs` | Trait for printer config persistence | Domain entities | SQLite, TCP |
| `repositories/print_job_repository.rs` | Trait for print job persistence | Domain entities | SQLite, TCP |
| `services/sequence_service.rs` | Sequence logic (next, reserve range) | Domain entities, repo traits | Infrastructure |
| `services/label_service.rs` | Label generation coordination | Domain entities, repo traits | Infrastructure |

### Application Layer

| File | Responsibility | Allowed Dependencies | Forbidden Dependencies |
|------|---------------|---------------------|----------------------|
| `use_cases/configure_printer.rs` | Save/update printer config | Domain, DTOs | Infrastructure |
| `use_cases/get_printer_config.rs` | Retrieve printer config | Domain, DTOs | Infrastructure |
| `use_cases/get_configured_printer.rs` | Get first configured printer | Domain, DTOs | Infrastructure impls |
| `use_cases/test_printer.rs` | Test printer connection | Domain, DTOs, `PrinterTransport` trait | Infrastructure impls |
| `use_cases/get_current_sequence.rs` | Return last used + next code | Domain, DTOs | Infrastructure impls |
| `use_cases/list_print_jobs.rs` | List recent print jobs | Domain, DTOs | Infrastructure impls |
| `use_cases/preview_label.rs` | Generate label preview | Domain, DTOs | Infrastructure impls |
| `use_cases/print_labels.rs` | Full print flow coordination | Domain, DTOs, `PrinterTransport` trait | Infrastructure impls |
| `dto/printer_dto.rs` | Printer config DTO for Tauri | serde | Nothing else |
| `dto/print_dto.rs` | Print request/result DTO + `From<PrintJob>` | serde, `PrintJob` | Nothing else |

### Infrastructure Layer

| File | Responsibility | Allowed Dependencies | Forbidden Dependencies |
|------|---------------|---------------------|----------------------|
| `database/connection.rs` | SQLite pool creation | sqlx | Domain, Tauri |
| `database/migrations.rs` | Migration runner | sqlx | Domain |
| `database/repositories/sqlite_*.rs` | Repository implementations | sqlx, domain traits | Tauri, TCP |
| `printer/tcp_transport.rs` | TCP raw socket transport | tokio, domain traits | SQLite, Tauri |
| `zpl/generator.rs` | ZPL II string generation | domain entities | SQLite, TCP |
| `zpl/label_layout.rs` | Label positioning math | domain entities | Everything |

### Commands Layer

| File | Responsibility | Allowed Dependencies | Forbidden Dependencies |
|------|---------------|---------------------|----------------------|
| `commands/printer_commands.rs` | Tauri commands for printer config | Application use cases | Infrastructure |
| `commands/print_commands.rs` | Tauri commands for printing | Application use cases | Infrastructure |

### Frontend Layer

| File | Responsibility | Allowed Dependencies | Forbidden Dependencies |
|------|---------------|---------------------|----------------------|
| `infrastructure/tauri/tauriClient.ts` | Tauri invoke wrapper | @tauri-apps/api | Everything else |
| `infrastructure/tauri/printerApi.ts` | Printer API calls | tauriClient | Nothing else |
| `infrastructure/tauri/printingApi.ts` | Print API calls | tauriClient | Nothing else |
| `stores/printer.ts` | Shared state (printer, sequence, status) | tauriClient, printerApi, printingApi | Direct Tauri invoke |
| `composables/usePrintProgress.ts` | Print stage machine (§33) | Nothing | Business logic |
| `router/index.ts` | Route definitions | views | Business logic |
| `utils/format.ts` | Date/status/error formatting | Nothing | Nothing |
| `components/**/*.vue` | UI components | TypeScript types, utils | Direct Tauri invoke |
| `views/*.vue` | View composition | Components, stores, APIs | Business logic |
| `types/index.ts` | Shared TypeScript types | Nothing | Nothing |

## 6. Dependency Decisions

### Rust Crates

| Crate | Version | Purpose | Justification |
|-------|---------|---------|---------------|
| `tauri` | 2 | Desktop framework | Required for app shell |
| `tokio` | 1 | Async runtime | Required for TCP, SQLx async |
| `serde` / `serde_json` | 1 | Serialization | DTO serialization for Tauri |
| `sqlx` | 0.8 | SQLite ORM | Compile-time checked queries, migrations |
| `chrono` | 0.4 | Date/time | Timestamps in labels and DB |
| `thiserror` | 2 | Error derivation | Typed errors without boilerplate |
| `tracing` | 0.1 | Logging | Structured logging |
| `tracing-subscriber` | 0.3 | Log output | Env-filter for log levels |

**Excluded (and why):**
- `anyhow`: Not used in domain/application. Errors must be typed for proper propagation to frontend.
- `uuid`: Using custom Z-prefixed codes, not UUIDs.
- `rand`: No randomness needed. Sequences are deterministic.
- `log`: Using `tracing` instead (structured logging).
- `rusqlite`: SQLx is preferred for async + compile-time checks.

### Frontend Packages

| Package | Version | Purpose | Justification |
|---------|---------|---------|---------------|
| `vue` | ^3.5 | UI framework | Required |
| `vue-router` | ^4.5 | Navigation | Multiple views |
| `pinia` | ^4.0 | State management | Shared state across views |
| `@tauri-apps/api` | ^2 | Tauri frontend API | invoke() bridge |
| `typescript` | ~5.7 | Type safety | Required |
| `vite` | ^8.0 | Build tool | Required |
| `vitest` | ^3.0 | Testing | Frontend tests |
| `jsdom` | ^30.0 | Test DOM | Component mount tests (dev only) |

## 7. Error Propagation

```
InfrastructureError (DatabaseError, PrinterConnectionFailed, etc.)
    │ impl From<X> for ApplicationError
    ▼
DomainError (InvalidBarcode, SequenceOverflow, etc.)
    │ impl From<X> for ApplicationError
    ▼
ApplicationError (wraps all errors, implements Serialize)
    │ impl Into<InvokeError>
    ▼
Tauri Command (returns Result<T, InvokeError>)
    │
    ▼
Vue (receives { code: string, message: string })
```

Each error type is defined with `thiserror`:

- **DomainError**: Business rule violations (invalid barcode, sequence overflow, invalid quantity)
- **ApplicationError**: Use case failures (printer not configured, database error)
- **InfrastructureError**: System-level failures (TCP connection refused, SQLite locked)

Frontend receives structured JSON:
```json
{
  "code": "PRINTER_CONNECTION_FAILED",
  "message": "No fue posible conectarse con la impresora en 192.168.1.100:9100"
}
```

## 8. Tauri State

Tauri manages an `AppState` struct that holds:
- Database connection pool (Arc<SqlitePool>)

Repositories and services are instantiated at startup and injected into command handlers.

```rust
pub struct AppState {
    pub db: Arc<SqlitePool>,
}
```

## 9. Key Architectural Decisions

### Decision 1: Store last_used_code, not next_code
Storing the last used code is safer. The "next" is always `increment(last_used)`. This avoids off-by-one errors when reserving ranges.

### Decision 2: Range-based print job storage
Print jobs store ranges (start_code, end_code, quantity) instead of one row per label. This keeps the database clean for large batches (10,000+ labels would mean 10,000 rows otherwise).

### Decision 3: Sequence reserved before sending
The sequence is updated in DB before sending ZPL to the printer. This prevents duplicate ranges even if the app crashes mid-print. The tradeoff: if TCP send succeeds but the app crashes before marking `completed`, the job stays `pending`/`printing`. This is acceptable because:
- It's better to lose numbers than duplicate them
- Users can review and clean up the history

### Decision 4: No "exactly once printing" promise
Raw TCP to port 9100 does not provide delivery confirmation. The app provides "at least once sequence reservation" but cannot guarantee physical printing occurred. Documented as a known limitation.

### Decision 5: Domain traits for testability
Repository traits and PrinterTransport trait allow unit testing without SQLite or a physical printer. Fakes are used instead of mocks.

## 10. Implementation Status

### Phase 1: Scaffolding ✅ COMPLETED

| Component | Status | Files |
|-----------|--------|-------|
| Domain Entities | ✅ | `sequence.rs`, `printer.rs`, `print_job.rs` |
| Domain Value Objects | ✅ | `barcode.rs`, `printer_config.rs` |
| Domain Repository Traits | ✅ | `sequence_repository.rs`, `printer_repository.rs`, `print_job_repository.rs` |
| Domain Services | ✅ | `sequence_service.rs`, `label_service.rs` |
| Application DTOs | ✅ | `printer_dto.rs`, `print_dto.rs` |
| Application Use Cases | ✅ | 5 use cases implemented |
| Infrastructure Database | ✅ | Connection, migrations, 3 SQLite repositories |
| Infrastructure Printer | ✅ | TCP transport, Zebra printer |
| Infrastructure ZPL | ✅ | Generator, label layout |
| Tauri Commands | ✅ | `printer_commands.rs`, `print_commands.rs` |
| Error Types | ✅ | Domain, Application, Infrastructure errors |
| State Management | ✅ | `app_state.rs` |
| Documentation | ✅ | 5 docs files created |

### Tests: 98 passing (backend) + 66 passing (frontend)

## 11. Phase 10 Hardening Notes

- **Structured logging** — `tracing_subscriber` env-filter init in `lib.rs::run()`
  (`RUST_LOG`, default `info`); `#[instrument]` + events on print flow, printer config,
  connection test, sequence reservation, and TCP transport.
- **Max print quantity** — `Sequence::MAX_PRINT_QUANTITY = 10_000` enforced in
  `reserve_range`; new `DomainError::QuantityTooLarge`. Concurrency of range reservation
  exercised by an explicit test (8 tasks × 25 codes).
- **`completed_at` constraint** — migration `002_completed_at_check.sql` rebuilds
  `print_jobs` with `CHECK (status NOT IN ('completed','failed') OR completed_at IS NOT NULL)`
  + backfill of legacy terminal rows. A blanket `NOT NULL` was avoided (pending/printing
  jobs legitimately have no completion time).
- **History detail** — `HistoryView.vue` uses `get_print_job` via `getPrintJob` for an
  on-demand detail panel.
- Kept calibration change in `label_layout.rs` (barcode_height 150, title_font_size 10);
  do not revert.
