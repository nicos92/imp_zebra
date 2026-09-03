# Plan: Envío de etiquetas fila por fila

## Problema Actual

Actualmente `ZplGenerator::generate_batch()` genera **un solo string ZPL** con todas las etiquetas en un único `^XA...^XZ`. Esto causa problemas porque:

1. La impresora recibe un string gigante con todas las etiquetas
2. Puede haber problemas de memoria o rendimiento en la impresora
3. No es el comportamiento esperado por el usuario

### Ejemplo actual (4 etiquetas, 2 columnas):

```
^XA
^PW800
^LL800
... (4 etiquetas juntas)
^XZ
```

## Solución Propuesta

Enviar **cada fila por separado**, cada una con su propio `^XA...^XZ`:

```
^XA
^PW800
^LL400
... (2 etiquetas - fila 0)
^XZ
^XA
^PW800
^LL400
... (2 etiquetas - fila 1)
^XZ
```

## Archivos a Modificar

### 1. `src-tauri/src/infrastructure/zpl/generator.rs`

**Cambios:**
- Agregar método `generate_batch_by_rows()` que retorne `Vec<String>`
- Cada batch contiene una fila completa de etiquetas
- Cada batch tiene su propio `^XA...^XZ`

**Firma del nuevo método:**
```rust
pub fn generate_batch_by_rows(
    &self, 
    labels: &[(String, LabelPosition)], 
    timestamp: &str
) -> Vec<String>
```

**Lógica:**
1. Agrupar labels por fila (usando `LabelPosition.row`)
2. Para cada fila, calcular el alto total de esa fila
3. Generar un ZPL independiente con `^PW{ancho_total}` y `^LL{alto_1_fila}`
4. Retornar vector de strings ZPL

### 2. `src-tauri/src/application/use_cases/print_labels.rs`

**Cambios:**
- Modificar `execute()` para usar `generate_batch_by_rows()`
- Enviar cada batch secuencialmente a la impresora
- Mantener el manejo de errores y estado del job

**Flujo actual:**
```rust
let zpl = generator.generate_batch(&labels_with_positions, &timestamp);
self.transport.send(zpl.as_bytes()).await?;
```

**Flujo nuevo:**
```rust
let batches = generator.generate_batch_by_rows(&labels_with_positions, &timestamp);
for batch in batches {
    self.transport.send(batch.as_bytes()).await?;
}
```

## Tests

### Tests existentes (deben seguir pasando)
- `test_generate_single_label`
- `test_generate_two_labels`
- `test_generate_odd_quantity_no_phantom`
- `test_generate_empty_batch`
- `test_print_labels_happy_path`
- `test_print_labels_send_failure_marks_failed`
- `test_print_labels_missing_printer`

### Tests nuevos a agregar
- `test_generate_batch_by_rows_single_row`
- `test_generate_batch_by_rows_multiple_rows`
- `test_generate_batch_by_rows_odd_quantity`

## Consideraciones

1. **Rendimiento**: El envío secuencial puede ser más lento, pero más confiable
2. **Compatibilidad**: El método `generate_batch()` se mantiene para compatibilidad
3. **Configuración**: En el futuro se puede agregar configuración de etiquetas por batch

## Cronograma

1. [ ] Escribir plan (este documento)
2. [ ] Implementar `generate_batch_by_rows()` en generator.rs
3. [ ] Modificar `execute()` en print_labels.rs
4. [ ] Ejecutar tests existentes
5. [ ] Agregar tests nuevos
6. [ ] Verificar compilación