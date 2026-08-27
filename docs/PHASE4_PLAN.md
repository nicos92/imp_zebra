# Phase 4: ZPL — Plan de Implementación

## Objetivo
Solidificar la capa ZPL alineando el código con la interfaz documentada en `docs/ZPL.md`: introducir el struct tipado `LabelPosition { row, column }`, eliminar la duplicación de cálculo de posiciones en los casos de uso y centralizar la regla de distribución 2 columnas en `LabelService`.

## Estado Actual
- ✅ `ZplGenerator::generate_batch` — genera ZPL II `^XA…^XZ` por lote
- ✅ `LabelLayout` — conversión mm→dots con redondeo, posiciones 2 columnas
- ✅ Tests: 3 en `generator.rs`, 4 en `label_layout.rs`, 2 en `label_service.rs`
- ❌ La interfaz usa tuplas crudas `(String, u32, u32)` (código, fila, columna) en vez de `LabelPosition`
- ❌ `PrintLabels` y `PreviewLabel` duplican inline el cálculo de posiciones
- ✅ Frontend en stub (Fase 8 pendiente) → el refactor es seguro internamente

---

## Tarea 1: Definir `LabelPosition` en `label_layout.rs`
Nuevo struct tipado:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LabelPosition {
    pub row: u32,
    pub column: u32,
}
```
- `row` = índice 0-based de fila
- `column` = 0 (izquierda) / 1 (derecha)
- Coherente con la interfaz documentada en `docs/ZPL.md` §9

## Tarea 2: `ZplGenerator::generate_batch(&[(String, LabelPosition)])`
- Cambiar la firma de `&[(String, u32, u32)]` → `&[(String, LabelPosition)]`
- Actualizar el loop para usar `pos.row` / `pos.column`
- Ajustar los 3 tests existentes

## Tarea 3: `LabelService::calculate_positions -> Vec<(String, LabelPosition)>`
- Cambiar el retorno de `Vec<(String, u32, u32)>`
- Mantener distribución: `impar → izquierda, par → derecha`, fila = `i / columns`
- Actualizar su test

## Tarea 4: Usar `LabelService` en los casos de uso (eliminar duplicación)
- `PrintLabels`: usar `LabelService::calculate_positions(&codes, columns)`
- `PreviewLabel`: usar `LabelPosition { row: 0, column: 0 }`
- Centraliza la regla de distribución en un solo lugar

## Tarea 5: Tests
Los tests existentes se adaptan al nuevo tipo (firma). El refactor no cambia el comportamiento del ZPL generado.

## Tarea 6: Verificación
- `cargo test` verde sin regresión (67 actuales)
- `cargo check` sin warnings nuevos por esta fase

## Tarea 7: Documentación
- Este archivo como plan de la fase
- Resumen de Fase 4 en `docs/DEVELOPMENT.md`, marcar `✅ COMPLETED`
- Alinear `docs/ZPL.md` §9 con el tipo `LabelPosition`

---

## Decisión de diseño: sin encoder Code 128 en Rust
El prompt maestro (§38) lista "Code128" en Fase 4, pero ZPL `^BC` ya codifica Code 128 nativamente en la impresora (la app solo envía el valor ASCII). Implementar un encoder Rust sería sobreingeniería (regla §40). El `Barcode` value object ya valida el formato `Z` + 7 dígitos. Se mantiene `^BCN` + `^BY2`.

## Archivos que se tocan
| Archivo | Acción |
|---------|--------|
| `src/infrastructure/zpl/label_layout.rs` | +`LabelPosition` struct |
| `src/infrastructure/zpl/generator.rs` | firma `generate_batch` + tests |
| `src/domain/services/label_service.rs` | retorno `calculate_positions` + test |
| `src/application/use_cases/print_labels.rs` | usar `LabelService` |
| `src/application/use_cases/preview_label.rs` | `LabelPosition` |
| `docs/PHASE4_PLAN.md` | este plan |
| `docs/DEVELOPMENT.md` | resumen Fase 4 |
| `docs/ZPL.md` | alinear interfaz §9 |
