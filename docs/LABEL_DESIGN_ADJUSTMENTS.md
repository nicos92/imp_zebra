# Ajustes de Diseño de Etiqueta

## Objetivo
Mejorar la legibilidad y apariencia de la etiqueta impresa ajustando el código de barras, el título con fecha, y alineando el formato ZPL con el generado por **Zebra Designer**.

## Estado final del diseño

```
┌──────────────────────────┐
│  26/08/2026 07:15:32     │  ← Título (fecha/hora), más alto
│█████████████████████████ │  ← Code 128 barcode (ancho y alto)
│█████████████████████████ │  ← Sin texto numérico debajo
│                          │
└──────────────────────────┘
```

## Parámetros de configuración de la etiqueta

> **Ubicación**: todos los parámetros se configuran en un solo archivo:
> **`src-tauri/src/infrastructure/zpl/label_layout.rs`** (función `LabelLayout::new`)

| Parámetro | Campo en código | Valor actual | Descripción |
|-----------|-----------------|--------------|-------------|
| Ancho etiqueta | `label_width_dots` | 400 dots | Ancho de cada etiqueta individual (dato desde config) |
| Alto etiqueta | `label_height_dots` | 400 dots | Alto de cada etiqueta individual (dato desde config) |
| Columnas | `columns` | 2 | Etiquetas por fila |
| Margen X | `margin_x` | 50 dots | Margen horizontal |
| Margen Y | `margin_y` | 50 dots | Margen vertical |
| **Alto barcode** | `barcode_height` | **253 dots** | 3.163 cm (exacto Zebra Designer) |
| **Ratio barcode** | `barcode_ratio` | **3.0** | Proporción ancho barra estrecha/ancha |
| **Tamaño título** | `title_font_size` | **20 dots** | Alto del título con fecha (ancho fijo 10) |
| DPI impresora | `dpi` | 203 | Resolución de la impresora |

### Parámetros fijos del barcode (en `generator.rs` `append_label`)
- **Ancho de módulo (barra estrecha)**: fijo en `3` dots = 15 mils (igual Zebra Designer)
- **Orientación**: `^BCB` (bottom-up, igual Zebra Designer)
- **Textos numéricos del barcode**: desactivados (`N,N`)

## Formato ZPL final (por etiqueta)

```zpl
^FO50,50
^A@N,20,10              ← Título: 20 alto x 10 ancho
^FD26/08/2026 07:15:32
^FS

^FO50,90
^BY3,3,253              ← module=3, ratio=3, height=253
^BCB,,N,N               ← Code 128, bottom-up, sin texto
^FDZ0000001
^FS
```

## Cómo cambiar un parámetro (ejemplo)

Para hacer el barcode más alto, editar en `label_layout.rs` `LabelLayout::new`:

```rust
barcode_height: 300,   // cambiar 253 → 300 (dots)
```

Para cambiar el ancho de las barras, editar el literal `3` en `generator.rs` `append_label`:

```rust
// en la línea: ^BY3,{ratio},{height}
// cambiar el "3" por el nuevo ancho de módulo
```

## Verificación de espacio (etiqueta 400x400 dots / 203 dpi)

```
margin_y:     50
título:       y=50,  alto=20
barcode:      y=90,  alto=253
fin barcode:  y=343
restante:      57 dots (suficiente para el tacto de corte)
```

## Cambios realizados

### 1. Código de barras - ancho, alto y orientación (formato Zebra)
- **Altura**: 200 → **253 dots** (coincide con Zebra Designer: 3.163 cm)
- **Ancho de módulo**: `^BY3` → usa los 3 parámetros completos `^BY3,3,253`
- **Orientación**: `^BCN` → `^BCB` (bottom-up, igual Zebra Designer)
- **Sin texto numérico**: se eliminó el comando `^BC` que imprimía el código bajo el barcode (se usa nuestro propio diseño, que también fue removido)

### 2. Título con fecha - más alto
- **Tamaño de fuente**: 10×10 → **20×10 dots** (más alto, mismo ancho)

### 3. Eliminación de texto numérico duplicado
- Se removió `code_text_position()` y `code_font_size` (ya no se imprime el código como texto, ni el barcode lo imprime)

## Tests
- `generator.rs`: verifica `^BCB`, contenido y lotes
- `label_layout.rs`: posiciones y valores por defecto
- Todos pasan (12/12 de ZPL)
