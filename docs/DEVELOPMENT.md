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

### Phase 3 — SQLite
Database connection, migrations, repositories, transactions.

### Phase 4 — ZPL
ZplGenerator, LabelLayout, Code 128, two-column layout with tests.

### Phase 5 — Printer Transport
PrinterTransport trait, TcpPrinterTransport, connection testing.

### Phase 6 — Application Layer
Use cases: ConfigurePrinter, TestPrinterConnection, GetCurrentSequence, PreviewLabel, PrintLabels.

### Phase 7 — Tauri
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
| Application/UseCases | `configure_printer.rs`, `get_printer_config.rs`, `test_printer.rs`, `preview_label.rs`, `print_labels.rs` | ✅ |
| Infrastructure/Database | `connection.rs`, `migrations.rs`, `sqlite_sequence_repository.rs`, `sqlite_printer_repository.rs`, `sqlite_print_job_repository.rs` | ✅ |
| Infrastructure/Printer | `tcp_transport.rs`, `zebra_printer.rs` | ✅ |
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
