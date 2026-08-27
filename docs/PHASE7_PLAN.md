# Phase 7 — Tauri Commands — Plan

## Contexto / Problema que resuelve

La capa de commands Tauri es el transporte delgado entre Vue y la aplicación Rust (§17).
Se construyó incrementalmente en fases previas: los 7 comandos de §17 existen y están
registrados en `lib.rs`.

Para cerrar Phase 7 quedan 3 gaps reales:

1. **Historial**: `PrintJobRepository::find_recent(limit)` existe, pero no hay caso de uso
   ni comando → la vista Historial (§32) no puede cargar datos (Fase 8).
2. **Descubrimiento de impresora**: `get_printer_config(id)` exige un `id` que el frontend
   no conoce al arrancar → el dashboard (§32) no puede mostrar la impresora configurada.
   `GetPrinterConfig::get_all()` no está expuesto.
3. **Limpieza** del mapeo `PrintJob → PrintJobDto` (hoy inline en el command `get_print_job`)
   para mantener commands delgados (§17) y evitar duplicación.

## Decisiones arquitectónicas (confirmadas)

- **`get_configured_printer`** (elegido): devuelve `Option<PrinterDto>` de la primera
  impresora de `find_all()`. UX de impresora única; se mantiene `get_printer_config(id)`.
- **`list_print_jobs`** (elegido): caso de uso `ListPrintJobs` que envuelve `find_recent(limit)`
  + comando delgado.
- **Composición por-command** (mantener decisión Phase 6): cada command construye sus repos;
  no centralizar en AppState.
- **Mapper compartido `From<PrintJob> for PrintJobDto`** en `print_dto.rs` → elimina duplicación.
- **Sin `Clock`** (decisión Phase 6). Sin abstracciones innecesarias (§40).
- `get_print_job` se mantiene como query thin que llama al repo y mapea vía el `From` compartido
  (sin añadir un use case extra, alineado con §40). Los queries que alimentan vistas
  (`list_print_jobs`, `get_configured_printer`) sí pasan por use cases, donde el dominio se testea.

## Archivos

### Nuevos
| Archivo | Responsabilidad |
|---------|-----------------|
| `docs/PHASE7_PLAN.md` | Plan de la fase |
| `src/application/use_cases/list_print_jobs.rs` | `find_recent(limit)` → `Vec<PrintJobDto>` + tests |
| `src/application/use_cases/get_configured_printer.rs` | primera impresora → `Option<PrinterDto>` + tests |

### Modificados
| Archivo | Responsabilidad |
|---------|-----------------|
| `src/application/dto/print_dto.rs` | `+ impl From<PrintJob> for PrintJobDto` |
| `src/application/use_cases/mod.rs` | `+ list_print_jobs`, `+ get_configured_printer` |
| `src/commands/printer_commands.rs` | `+ get_configured_printer` command |
| `src/commands/print_commands.rs` | usar mapper `From` en `get_print_job`; `+ list_print_jobs` command |
| `src/lib.rs` | registrar `get_configured_printer`, `list_print_jobs` |
| `docs/DEVELOPMENT.md` | Phase 7 ✅ + resumen |
| `docs/ARCHITECTURE.md` | alinear tabla de use cases/commands |

## Tests nuevos

- `list_print_jobs.rs`: lista mapeada; vacío sin trabajos; respeta límite.
- `get_configured_printer.rs`: con impresora → `Some(dto)`; sin impresora → `None`.

## Verificación (orden exigido)

1. `cargo check`
2. `cargo fmt` + `cargo clippy`
3. `cargo test` (79 → ~83)
4. `cargo check` (repetir)
5. `cargo fmt` + `cargo clippy` (repetir)
