# Phase 8 — Vue Frontend — Plan

## Contexto / Problema que resuelve

El frontend es hoy el scaffold por defecto de Tauri+Vue (`App.vue` con "greet").
Esta fase lo convierte en la UI completa que cumple la interfaz de §32 y los
criterios de aceptación (§42), consumiendo los 9 commands construidos en fases
previas mediante `invoke`. **El frontend no genera ZPL ni códigos secuenciales** (§16).

## Decisiones arquitectónicas (confirmadas con el usuario)

1. **Pinia mínimo (1 store `printer`)**: estado compartido entre las 3 vistas
   (impresora configurada, próximo código, estado de conexión). Se refresca tras
   guardar en Configuración y tras imprimir en Dashboard. Justificado porque el
   estado cruza varias vistas (§16).
2. **3 vistas consolidadas**: `DashboardView` (estado + próximo código + cantidad
   + imprimir + preview), `PrinterSettingsView` (form + probar conexión),
   `HistoryView` (tabla `list_print_jobs`). Alineado con la pantalla principal
   unificada de §32.
3. **vue-router**: ya es dependencia; se configura con las 3 rutas.
4. **Capa `infrastructure/tauri` + tipos TS**: `tauriClient` envuelve `invoke` y
   traduce el rechazo `{ code, message }` en `TauriError`; `printerApi`/`printingApi`
   exponen funciones tipadas por command. El resto de la app no conoce `invoke`.
5. **Sin ZPL ni secuencia en frontend**: solo renderiza el ZPL que devuelve
   `preview_label` y muestra códigos que vienen del backend.
6. **`get_print_job` se omite** (§40): ninguna vista lo requiere; Historial usa
   `list_print_jobs`. Se añadirá con una vista de detalle si hace falta.

## Commands consumidos → API TS

| Command Rust | Función TS | DTO devuelto |
|---|---|---|
| `get_configured_printer()` | `getConfiguredPrinter()` | `Printer \| null` |
| `get_current_sequence()` | `getCurrentSequence()` | `SequenceInfo` |
| `print_labels({quantity, printer_id})` | `printLabels(...)` | `PrintResult` |
| `preview_label(w,h,cols,dpi)` | `previewLabel(...)` | `LabelPreview` |
| `test_printer_connection(id)` | `testPrinterConnection(id)` | `boolean` |
| `get_printer_config(id)` | `getPrinterConfig(id)` | `Printer \| null` |
| `save_printer_config(config)` | `savePrinterConfig(config)` | `Printer` |
| `list_print_jobs(limit?)` | `listPrintJobs(limit)` | `PrintJob[]` |

## Archivos

### Nuevos (frontend `src/`)
| Archivo | Responsabilidad |
|---------|-----------------|
| `docs/PHASE8_PLAN.md` | Plan de la fase |
| `src/router/index.ts` | 3 rutas (dashboard, settings, history) |
| `src/stores/printer.ts` | Store Pinia `usePrinterStore` |
| `src/composables/usePrintProgress.ts` | Máquina de pasos de impresión (§33) |
| `src/infrastructure/tauri/tauriClient.ts` | Wrapper de `invoke` + `TauriError` |
| `src/infrastructure/tauri/printerApi.ts` | Funciones por command (printer) |
| `src/infrastructure/tauri/printingApi.ts` | Funciones por command (printing) |
| `src/types/index.ts` | Tipos TS (DTOs + errores) |
| `src/utils/format.ts` | Formateo de fecha/estado/errores |
| `src/styles/main.css` | Reset + tokens globales |
| `src/views/DashboardView.vue` | Pantalla principal |
| `src/views/PrinterSettingsView.vue` | Configuración de impresora |
| `src/views/HistoryView.vue` | Historial de trabajos |
| `src/components/common/AppButton.vue` | Botón base |
| `src/components/common/AppInput.vue` | Input base |
| `src/components/common/AppModal.vue` | Modal base (preview) |
| `src/components/printer/PrinterStatus.vue` | Estado impresora + próximo código |
| `src/components/printer/PrinterForm.vue` | Form de configuración |
| `src/components/printing/PrintQuantityForm.vue` | Form cantidad (§32) |
| `src/components/printing/PrintProgress.vue` | Progreso §33 |
| `src/components/printing/PrintResult.vue` | Resultado/códigos |
| `src/components/printing/LabelPreview.vue` | Preview de etiqueta |
| tests (`src/**/__tests__` o `*.spec.ts`) | Vitest: api + store + composable |

### Modificados
| Archivo | Responsabilidad |
|---------|-----------------|
| `src/main.ts` | Configurar router + pinia + mount |
| `src/App.vue` | Shell con nav + `<router-view>` |
| `index.html` | Título |
| `package.json` | + `pinia` |
| eslint/prettier config | flat configs para `lint`/`format` |
| `docs/DEVELOPMENT.md` | Phase 8 ✅ + resumen |
| `docs/ARCHITECTURE.md` | Alinear Frontend Layer |

## Dependencias nuevas
- `pinia` (~3.x). `vue-router`, `@tauri-apps/api` ya presentes. Sin más adiciones.

## Tests
- `tauriClient`/apis: mockear `invoke`, verificar argumentos y mapeo de error.
- `stores/printer`: `load`, `refreshAfterSave`, estados.
- `usePrintProgress`: transiciones y manejo de error.
- Componentes: solo monto/smoke.

## Verificación (orden)
1. `pnpm test` (vitest)
2. `pnpm build` (vue-tsc --noEmit + vite build)
3. `pnpm lint` + `pnpm format`
4. Sin dependencias innecesarias nuevas (solo pinia)
5. Actualizar `docs/DEVELOPMENT.md` y `docs/ARCHITECTURE.md`

## Notas
- ESLint/Prettier no tienen config en el repo → crear flat configs acordes a los scripts.
- Los custom commands no requieren cambios en `capabilities`.
