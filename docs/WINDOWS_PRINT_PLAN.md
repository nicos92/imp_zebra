# PLAN: Reemplazar impresión TCP directa por diálogo de impresión de Windows

## Problema actual

La aplicación genera ZPL y lo envía por TCP raw (puerto 9100) directamente a la impresora Zebra. Esto causa que el código de barras salga deformado/ancho. Cuando el mismo contenido se imprime desde un archivo .TXT usando el driver de Windows de la impresora (Ctrl+P desde el Bloc de Notas), el ancho del código de barras es correcto.

## Objetivo

Reemplazar el transporte TCP raw por el subsystem de impresión de Windows. Mostrar el diálogo nativo `PrintDlgEx` antes de cada impresión, y enviar el contenido ZPL a través del driver de Windows de la impresora seleccionada.

---

## Fase 1: Módulo Windows Print en Rust (Backend)

### Archivo nuevo: `src-tauri/src/infrastructure/printer/windows_printer.rs`

Tres funciones principales:

1. **`list_installed_printers()`** → `Vec<WindowsPrinterInfo>`
   - Llama a `EnumPrintersW` con flags `PRINTER_ENUM_LOCAL | PRINTER_ENUM_CONNECTIONS`
   - Retorna lista de impresoras instaladas con nombre y driver

2. **`show_print_dialog(hwnd_owner: Option<isize>)`** → `Option<String>`
   - Crea struct `PRINTDLGEXW`, llama a `PrintDlgExW`
   - Si el usuario acepta, retorna el nombre de la impresora seleccionada
   - Si cancela, retorna None
   - Se ejecuta en un thread aparte (el diálogo tiene su propio modal loop)

3. **`print_to_printer(printer_name: &str, data: &[u8])`** → `Result<()>`
   - Secuencia Win32: `OpenPrinterW` → `StartDocPrinterW` (datatype RAW) → `WritePrinter` → `EndDocPrinter` → `ClosePrinter`
   - Envía bytes ZPL a través del spooler de Windows

### Transporte: `WindowsPrintTransport`

Implementa el trait existente `PrinterTransport`:
```rust
pub struct WindowsPrintTransport {
    printer_name: String,
}
// send(data) → windows_printer::print_to_printer(...)
// test_connection() → Ok(()) (conexión manejada por el spooler)
```

Esto permite que `PrintLabels` use case no cambie — recibe un `dyn PrinterTransport` igual que antes.

### Modificar: `src-tauri/src/infrastructure/printer/mod.rs`
- Agregar `pub mod windows_printer;`

### Modificar: `src-tauri/Cargo.toml`
- Agregar crate `windows` con features: `"Win32_System_Printing"`, `"Win32_UI_WindowsAndMessaging"`, `"Win32_Foundation"`

---

## Fase 2: Nuevos comandos Tauri

### Modificar: `src-tauri/src/commands/print_commands.rs`

- **Nuevo comando: `list_windows_printers`** — Llama a `windows_printer::list_installed_printers()`, retorna `Vec<WindowsPrinterInfoDto>`

- **Comando modificado: `print_labels`** — Nuevo flujo:
  1. Llama a `show_print_dialog()` — si el usuario cancela, retorna `ApplicationError::PrintCancelled`
  2. Crea `WindowsPrintTransport(printer_name)`
  3. Ejecuta el caso de uso `PrintLabels` existente (busca dimensiones de etiqueta en DB, genera ZPL, envía via transport)

### Modificar: `src-tauri/src/lib.rs`
- Registrar comando `list_windows_printers`
- Eliminar comando `test_printer_connection`

### Modificar: `src-tauri/src/errors/application_error.rs`
- Agregar variante `#[error("Print cancelled by user")] PrintCancelled` con código `"PRINT_CANCELLED"`

---

## Fase 3: DTO y tipos

### Modificar: `src-tauri/src/application/dto/printer_dto.rs`
- Agregar `WindowsPrinterInfoDto { name: String, driver_name: String }` con Serialize + Deserialize

### Modificar: `src/types/index.ts`
- Agregar `interface WindowsPrinter { name: string; driver_name: string }`

---

## Fase 4: Cambios Frontend

### Modificar: `src/infrastructure/tauri/printingApi.ts`
- Agregar `listWindowsPrinters(): Promise<WindowsPrinter[]>`
- `printLabels()` se mantiene igual (el diálogo ahora lo muestra el backend)

### Modificar: `src/views/DashboardView.vue`
- Cuando el usuario hace clic en "Imprimir", el flujo existente llama a `printLabels()` que dispara el diálogo nativo en el lado Rust
- Flujo de progreso/resultados se mantiene

---

## Fase 5: Simplificar configuración de impresora

### Modificar: `src/components/printer/PrinterForm.vue`
- Eliminar campos: `name`, `model`, `connection_type`, `ip_address`, `port`
- Mantener solo: `dpi`, `label_width_mm`, `label_height_mm`, `columns`
- Eliminar botón "Probar conexión" (ya no aplica para TCP)
- Actualizar defaults: eliminar `ZT410` hardcodeado, puerto `9100`, conexión TCP

### Modificar: `src/views/PrinterSettingsView.vue`
- Eliminar función `handleTest()` y estado relacionado
- Eliminar referencia al botón "Probar conexión"
- Simplificar a solo configuración de dimensiones de etiqueta

### Modificar: `src/infrastructure/tauri/printerApi.ts`
- Eliminar export `testPrinterConnection()`

### Modificar: `src-tauri/src/commands/printer_commands.rs`
- Eliminar comando `test_printer_connection`

---

## Resumen de archivos

| Archivo | Cambio |
|---------|--------|
| `src-tauri/Cargo.toml` | Agregar crate `windows` |
| `src-tauri/src/infrastructure/printer/mod.rs` | Agregar módulo `windows_printer` |
| `src-tauri/src/infrastructure/printer/windows_printer.rs` | **NUEVO** — Wrapper de APIs Win32 de impresión |
| `src-tauri/src/infrastructure/printer/tcp_transport.rs` | Se mantiene (para tests), no se usa en producción |
| `src-tauri/src/commands/print_commands.rs` | Agregar `list_windows_printers`, modificar `print_labels` |
| `src-tauri/src/commands/printer_commands.rs` | Eliminar `test_printer_connection` |
| `src-tauri/src/lib.rs` | Registrar `list_windows_printers`, eliminar `test_printer_connection` |
| `src-tauri/src/errors/application_error.rs` | Agregar variante `PrintCancelled` |
| `src-tauri/src/application/dto/printer_dto.rs` | Agregar `WindowsPrinterInfoDto` |
| `src/types/index.ts` | Agregar interfaz `WindowsPrinter` |
| `src/infrastructure/tauri/printingApi.ts` | Agregar `listWindowsPrinters()` |
| `src/infrastructure/tauri/printerApi.ts` | Eliminar `testPrinterConnection()` |
| `src/views/DashboardView.vue` | Ajustes menores en flujo de impresión |
| `src/components/printer/PrinterForm.vue` | Eliminar campos IP/puerto/conexión |
| `src/views/PrinterSettingsView.vue` | Eliminar lógica de test de conexión |

---

## Qué se mantiene igual

- Caso de uso `PrintLabels` (reserva códigos → genera ZPL → envía via transport)
- `ZplGenerator` y `LabelLayout` (contenido ZPL sin cambios)
- Manejo de secuencias, tracking de trabajos de impresión, schema de base de datos
- Funcionalidad de preview
- Trait `PrinterTransport`

---

## Consideraciones / Riesgos

1. **Threading**: `PrintDlgExW` es una llamada UI bloqueante. Se ejecutará en `std::thread::spawn` ya que crea su propio modal loop. El HWND de la ventana Tauri se pasa como `hwndOwner` para hacerlo modal.

2. **Tests unitarios**: Los tests TCP existentes se mantienen. `WindowsPrintTransport` se testea con tests de integración en máquina Windows. Los tests de `PrintLabels` usan `FakePrinterTransport` y no se ven afectados.

3. **Entidad `PrinterConfig`** en la DB: sigue almacenando dimensiones de etiqueta. La página de settings pasa de "Configuración de impresora" a "Configuración de etiqueta".
