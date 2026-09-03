# Plan: Configuración de etiqueta según Zebra Designer (ejemplo-etiqueta.prn)

## Objetivo
Alinear el tamaño y el diseño de la etiqueta generada por la app con la configuración
real exportada desde Zebra Designer (`docs/ejemplo-etiqueta.prn`).

## Referencia de tamaño (ejemplo-etiqueta.prn)
```zpl
^PW751      ← ancho de fila completa = 94 mm (2 etiquetas de ~47mm)
^LL392      ← alto de etiqueta = 49 mm
^BY3,3,240  ← barcode: module=3, ratio=3.0, alto=240 dots (30mm)
^BCB,,N,N   ← Code 128, orientación bottom-up, sin interpretación de texto
```

## Decisiones de diseño
1. **Barcode**: alto = 240 dots, `^BY3,3,240`, `^BCB,,N,N`
2. **Negrita** (ZPL no tiene parámetro directo): se simula con ancho > alto (w > h)
3. **Fecha**: más chica que el código y en negrita

## Valores de fuente propuestos
| Texto   | Alto (h) | Ancho (w) | Nota                          |
|---------|----------|-----------|-------------------------------|
| Fecha   | 16       | 20        | más chica que código, negrita |
| Código  | 22       | 28        | negrita (w > h)               |

## Cambios por archivo

### src-tauri/src/infrastructure/zpl/label_layout.rs
- `barcode_height`: 253 → 240
- Añadir `title_font_width` (fecha) y `code_font_width` (código) para negrita
- `title_font_size` (fecha alto): 20 → 16
- `code_font_size` (código alto): 25 → 22
- Recalcular `barcode_position` con el nuevo `title_font_size`

### src-tauri/src/infrastructure/zpl/generator.rs
- Barcode: `^BY3\n^BCN,{h},Y,N` → `^BY3,3,{h}\n^BCB,,N,N`
- Fecha: `^A@N,{h},{h}` → `^A@N,16,20` (reducida + negrita)
- Código texto: `^A@N,{h},{h}` → `^A@N,22,28` (negrita)

### ZPL resultante por etiqueta
```zpl
; Fecha (reducida + negrita)
^FO{x},{y}^A@N,16,20^FD{timestamp}^FS

; Barcode (Zebra) sin interpretación
^BY3,3,240
^BCB,,N,N
^FD{code}^FS

; Código humano debajo (negrita)
^A@N,22,28
^FD{code}^FS
```

## Configuración de impresora para ^PW751 exacto (2 columnas)
- `label_width_mm ≈ 47`  (→ 2×47mm ≈ 94mm = 751 dots)
- `label_height_mm = 49` (→ 392 dots)
- `columns = 2`

## Verificación de espacio (alto etiqueta 392 dots)
```
fecha   y=50  (h16) → 66
barcode y=86  (h240) → 326
código  y=336 (h22) → 358  ✓ cabe en 392
```

## Tests
- `generator.rs`: test `test_generate_single_label` ya espera `^BCB`; al generar `^BCB` pasará.
- Ejecutar `cargo test --lib` para verificar.
