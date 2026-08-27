# Phase 6: Application Layer — Plan de Implementación

## Objetivo
Completar y endurecer la capa de aplicación: **inyectar dependencias** en los casos de uso de impresora para hacerlos testeables (§29/§30, criterio aceptación #19), **eliminar el caso de uso faltante `GetCurrentSequence`**, **eliminar las dependencias concretas de infraestructura** dentro de la capa de aplicación y **añadir tests unitarios**.

## Estado Actual
- ✅ Los casos de uso `ConfigurePrinter`, `GetPrinterConfig`, `TestPrinter`, `PreviewLabel`, `PrintLabels` existen (construidos incrementalmente).
- ❌ `TestPrinter` construye `TcpPrinterTransport` concreto inline → **no testeable** sin Zebra física; tiene **cero tests**.
- ❌ `PrintLabels` construye `ZebraPrinter::new(printer)` concreto inline → **no testeable**; además **ignora** los `Result` de `job.start_printing()/complete()/fail()` (3 warnings `unused Result`).
- ❌ **`GetCurrentSequence` no existe.** `SequenceInfoDto` (`last_used_code`, `next_code`) está definido pero sin uso.
- ✅ 74 tests pasando (baseline, `cargo test`), `cargo check` sin errores.
- ✅ `PrinterTransport` trait + `TcpPrinterTransport` + `FakePrinterTransport` ya existen (Fase 5).

## Problema que resuelve
- **Testabilidad**: los casos de uso que tocan impresora no pueden probarse sin hardware real (viola §29/§30 y criterio #19).
- **Regla de dependencias (§43)**: la capa de aplicación no debe depender de concretos de infraestructura (`TcpPrinterTransport`, `ZebraPrinter`) construidos inline, sino de abstracciones inyectadas.
- **Completitud**: falta `GetCurrentSequence` para el Dashboard ("Próximo código", criterio #4).
- **Correctitud**: los `Result` ignorados de las transiciones de `PrintJob` se propagan explícitamente.

## Decisión arquitectónica
Inyectar **`Arc<dyn PrinterTransport>`** en los casos de uso que tocan impresora (`TestPrinter`, `PrintLabels`). Reutilizar el trait ya existente (§40: no crear abstracciones nuevas sin necesidad). La **composición** (construir un `TcpPrinterTransport` concreto a partir de la config de impresora) ocurre en la capa de **commands (composition root)**, manteniendo la capa de aplicación libre de infraestructura concreta.

- **Sin abstracción `Clock`** (§40): el tiempo `chrono::Local::now()/Utc::now()` del backend es aceptable; los tests verifican estructura (códigos, ZPL, estados), no timestamps exactos.
- **Estilo de composición por-command** (decisión confirmada): cada command construye sus repos y el transporte, igual que el código actual.
- **Cablear `get_current_sequence` ahora** (decisión confirmada): se añade el command y se registra en `lib.rs` aunque Phase 7 formalice commands.

---

## Tarea 1: Caso de uso `GetCurrentSequence` — `get_current_sequence.rs` (nuevo)
- Depende de `Arc<SequenceService>`.
- `execute() -> Result<SequenceInfoDto, ApplicationError>`:
  - `last_used_code` = `sequence.last_used_code()`
  - `next_code` = clonar `Sequence`, llamar `.next()` (sin persistir — semántica de preview)
- Registrar en `application/use_cases/mod.rs`.
- Tests (fake `SequenceRepository` inline como el de `sequence_service.rs`):
  - devuelve `last_used_code` y `next_code` correctos.

## Tarea 2: `TestPrinter` — DI + tests — `test_printer.rs`
- Constructor: `new(printer_repo: Arc<dyn PrinterRepository>, transport: Arc<dyn PrinterTransport>)`
- Eliminar `TcpPrinterTransport::new(...)` inline; llamar `transport.test_connection()`.
- Tests (fake transporte + repo impresora en memoria):
  - conexión ok → `Ok(true)`
  - conexión falla → propaga error (`ApplicationError::Infrastructure`)

## Tarea 3: `PrintLabels` — DI + corregir Results + tests — `print_labels.rs`
- Constructor gana `transport: Arc<dyn PrinterTransport>`.
- Reemplazar `ZebraPrinter::new(printer).send_zpl(&zpl)` por `transport.send(zpl.as_bytes())`.
- Propagar los 3 `Result` ignorados con `?` (mapeados a `ApplicationError::Domain`).
- Tests (fakes en memoria de seq/printer/job repos + fake transporte):
  - flujo completo ok: job `Pending`→`Printing`→`Completed`, `start_code`/`end_code` correctos, transporte recibe el ZPL.
  - fallo de envío: job `Failed`, devuelve `ApplicationError::PrintJobFailed`.
  - impresora no configurada: `ApplicationError::PrinterNotConfigured`.

## Tarea 4: Commands — composición por-command + nuevo command
- `commands/printer_commands.rs` `test_printer_connection`: cargar impresora → construir `Arc<dyn PrinterTransport>` → `TestPrinter::new(repo, transport)`.
- **Nuevo** `get_current_sequence`: construir repo secuencia → `SequenceService` → `GetCurrentSequence`.
- `commands/print_commands.rs` `print_labels`: cargar impresora → construir transporte → `PrintLabels::new(seq, printer_repo, job_repo, transport)`.
- `lib.rs`: registrar `get_current_sequence` en `invoke_handler`.

## Tarea 5: Documentación
- `docs/PHASE6_PLAN.md` — este plan
- `docs/DEVELOPMENT.md` — resumen Fase 6, marcar ✅ COMPLETED
- `docs/ARCHITECTURE.md` — alinear si documenta los seams de DI de los casos de uso

---

## Verificación (orden exigido)
1. `cargo check`
2. `cargo fmt` + `cargo clippy`
3. Tests (`cargo test`) — esperado ~82-84 pasando
4. Repetir `cargo check`
5. Repetir `cargo fmt` + `cargo clippy`
6. Sin warnings nuevos; los 3 `unused Result` deben desaparecer.

## Archivos que se tocan
| Archivo | Acción |
|---------|--------|
| `src/application/use_cases/get_current_sequence.rs` | **Nuevo** — caso de uso + tests |
| `src/application/use_cases/test_printer.rs` | DI transporte + tests |
| `src/application/use_cases/print_labels.rs` | DI transporte + propagar Result + tests |
| `src/application/use_cases/mod.rs` | +`get_current_sequence` |
| `src/commands/printer_commands.rs` | componer transporte; +`get_current_sequence` |
| `src/commands/print_commands.rs` | componer transporte en `print_labels` |
| `src/lib.rs` | registrar `get_current_sequence` |
| `docs/PHASE6_PLAN.md` | este plan |
| `docs/DEVELOPMENT.md` | resumen Fase 6 |
| `docs/ARCHITECTURE.md` | alinear DI si aplica |
