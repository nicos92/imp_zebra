# Phase 5: Printer Transport — Plan de Implementación

## Objetivo
Introducir la abstracción `PrinterTransport` con su implementación TCP (`TcpPrinterTransport`), haciendo que `ZebraPrinter` dependa del trait (inversión de dependencias) y añadiendo tests sin necesidad de una Zebra física.

## Estado Actual
- ✅ `TcpTransport` — struct concreto en `infrastructure/printer/tcp_transport.rs`
- ❌ No existe el trait `PrinterTransport` documentado en PRINTING.md §7
- ❌ `ZebraPrinter` depende de `Arc<TcpTransport>` concreto
- ✅ `TestPrinter`/`PrintLabels` construyen transporte/printer (Fase 6 refactoriza DI)
- ❌ Cero tests de la capa de transporte
- ✅ `tokio` (net, time, macros, rt-multi-thread) y `async-trait` ya disponibles

## Problema que resuelve
- Testabilidad: testear transporte y printer sin Zebra física (§29/§30)
- Cambiar transporte TCP→USB/Serial sin tocar dominio (criterio aceptación #20)
- Respetar la regla de no sobreingeniería (§40): el trait responde a una necesidad real

## Decisión arquitectónica
Trait `PrinterTransport` async en infraestructura; `TcpPrinterTransport` como implementación TCP real; `FakePrinterTransport` (solo tests) para la abstracción. `ZebraPrinter` pasa a `Arc<dyn PrinterTransport>`.

**Alcance:** solo capa de infraestructura + `ZebraPrinter`. NO refactorizar casos de uso (Fase 6). Solo renombrar `TcpTransport`→`TcpPrinterTransport` donde se usa.

---

## Tarea 1: Trait `PrinterTransport` — `printer_transport.rs`
```rust
#[async_trait]
pub trait PrinterTransport: Send + Sync {
    async fn send(&self, data: &[u8]) -> Result<(), InfrastructureError>;
    async fn test_connection(&self) -> Result<(), InfrastructureError>;
}
```
- Reutiliza `InfrastructureError` (no crea `PrinterError` — regla §40)
- `test_connection` (más informativo que `is_connected(): bool`, alineado con `TestPrinter`)

## Tarea 2: `TcpPrinterTransport` implementa el trait — `tcp_transport.rs`
- Renombrar struct `TcpTransport` → `TcpPrinterTransport`
- Implementar `PrinterTransport` (`send` + `test_connection`)
- Añadir `new_with_timeouts(ip, port, connect_timeout, write_timeout)` para tests
- Mantener `new(ip, port)` con defaults (5s connect, 30s write)

## Tarea 3: `ZebraPrinter` depende del trait — `zebra_printer.rs`
- Campo `transport: Arc<dyn PrinterTransport>`
- `new(printer)` construye `Arc::new(TcpPrinterTransport::new(...))`
- `send_zpl` / `test_connection` delegan en el trait

## Tarea 4: Registrar módulo — `mod.rs`
- Añadir `pub mod printer_transport;`

## Tarea 5: Compilar dependencias — casos de uso
- `test_printer.rs`: renombrar `TcpTransport` → `TcpPrinterTransport`
- `print_labels.rs`: sin cambios (usa `ZebraPrinter`)

## Tarea 6: Tests
**`tcp_transport.rs`:**
- `test_address_format` — "ip:port"
- `test_send_to_listener` — TcpListener local + send → Ok
- `test_connection_refused` — puerto cerrado → `PrinterConnection`
- `test_timeout_mapping` — IP incomunicable + connect_timeout corto → `PrinterTimeout`

**`zebra_printer.rs`:**
- `test_send_zpl_ok` — fake success → Ok
- `test_send_zpl_error` — fake error → propaga error
- `test_test_connection` — fake → Ok

**`FakePrinterTransport`** en `printer_transport.rs` (cfg(test)) para los tests de ZebraPrinter.

## Tarea 7: Documentación
- `docs/PHASE5_PLAN.md` — este plan
- `docs/DEVELOPMENT.md` — resumen Fase 5, marcar ✅ COMPLETED
- `docs/PRINTING.md` — alinear interfaz trait con la real

---

## Verificación (orden exigido)
1. `cargo check`
2. `cargo fmt` + `cargo clippy`
3. Tests (`cargo test`)
4. Repetir `cargo check`
5. Repetir `cargo fmt` + `cargo clippy`
6. Sin warnings nuevos introducidos por la fase

## Archivos que se tocan
| Archivo | Acción |
|---------|--------|
| `src/infrastructure/printer/printer_transport.rs` | **Nuevo** — trait + Fake (test) |
| `src/infrastructure/printer/tcp_transport.rs` | rename struct, impl trait, +tests |
| `src/infrastructure/printer/zebra_printer.rs` | `Arc<dyn PrinterTransport>` +tests |
| `src/infrastructure/printer/mod.rs` | +`printer_transport` |
| `src/application/use_cases/test_printer.rs` | rename |
| `docs/PHASE5_PLAN.md` | este plan |
| `docs/DEVELOPMENT.md` | resumen Fase 5 |
| `docs/PRINTING.md` | alinear interfaz |
