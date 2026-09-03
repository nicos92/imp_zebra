# Ajustes de Diseño de Etiqueta

## Objetivo
Alinear el diseño y tamaño de la etiqueta generada por la app con la configuración
real exportada desde **Zebra Designer** (`docs/ejemplo-etiqueta.prn`).

## Diseño final

```
┌──────────────────────────┐
│  26/08/2026 07:15:32     │  ← Título(fecha/hora), reducido y en negrita
│█████████████████████████ │  ← Code 128 barcode, alto 240 dots, sin texto
│█████████████████████████ │
│      Z0000001            │  ← Código humano en negrita
└──────────────────────────┘
```

## Parámetros de configuración de la etiqueta

> **Ubicación**: todos los parámetros se configuran en un solo archivo:
> **`src-tauri/src/infrastructure/zpl/label_layout.rs`** (función `LabelLayout::new`)

| Parámetro | Campo en código | Valor | Descripción |
|-----------|-----------------|-------|-------------|
| Ancho etiqueta | `label_width_dots` | desde config | Ancho por etiqueta (dots) |
| Alto etiqueta | `label_height_dots` | desde config | Alto por etiqueta (dots) |
| Columnas | `columns` | 2 | Etiquetas por fila |
| Margen X | `margin_x` | 50 | Margen horizontal |
| Margen Y | `margin_y` | 50 | Margen vertical |
| **Alto barcode** | `barcode_height` | **240** | 30 mm (igual Zebra Designer) |
| **Ratio barcode** | `barcode_ratio` | **3.0** | Proporción estrecha/ancha |
| **Alto fecha** | `title_font_size` | **16** | Alto del título fecha |
| **Ancho fecha** | `title_font_width` | **20** | Ancho > alto → negrita |
| **Alto código** | `code_font_size` | **22** | Alto del código bajo barcode |
| **Ancho código** | `code_font_width` | **28** | Ancho > alto → negrita |
| DPI | `dpi` | 203 | Resolución impresora |

### Parámetros fijos del barcode (en `generator.rs` `append_label`)
- **Ancho de módulo (barra estrecha)**: fijo en `3` dots = 15 mils
- **Orientación**: `^BCB` (bottom-up, igual Zebra Designer)
- **Texto de interpretación del barcode**: desactivado (`N,N`)
- **Formato**: `^BY3,3,240` + `^BCB,,N,N` (exacto Zebra Designer)

## Cómo simular negrita en ZPL
ZPL `^A@` no tiene parámetro de negrita directo. Se simula haciendo que el **ancho (w) sea mayor que el alto (h)**:
- Fecha → `^A@N,16,20` (w=20 > h=16)
- Código → `^A@N,22,28` (w=28 > h=22)

## Cómo cambiar un parámetro (ejemplo)

Barcode más alto → editar en `label_layout.rs`:
```rust
barcode_height: 300,   // cambiar 240 → 300
```

Código debajo más grande → editar:
```rust
code_font_size: 26,    // alto
code_font_width: 34,   // ancho (negrita si > alto)
```

## Verificación de espacio (etiqueta 392 dots de alto / 49mm)
```
fecha   y=50  (h16) → 66
barcode y=86  (h240) → 326
código  y=336 (h22) → 358  ✓ cabe en 392
```

## Configuración de impresora para `^PW751` exacto (2 columnas)
Para que la app genere `^PW751` (94mm fila) y `^LL392` (49mm):
- `label_width_mm ≈ 47` (→ 2×47mm ≈ 94mm = 751 dots)
- `label_height_mm = 49` (→ 392 dots)
- `columns = 2`

## Tests
- `generator.rs`: verifica `^BCB,,N,N`, `^BY3,3,240`, contenido y lotes
- `label_layout.rs`: verifica valores de fuente y barcode
- Todos los tests ZPL pasan (12/12)
