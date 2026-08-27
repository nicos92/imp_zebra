# ZPL

## 1. Overview

The application generates ZPL II (Zebra Programming Language) commands. All ZPL is generated exclusively in the Rust backend by `ZplGenerator`.

## 2. Commands Used

| Command | Function | Example |
|---------|----------|---------|
| `^XA` | Start format | `^XA` |
| `^XZ` | End format | `^XZ` |
| `^PW` | Print width (dots) | `^PW800` |
| `^LL` | Label length (dots) | `^LL400` |
| `^LS` | X origin offset | `^LS0` |
| `^LT` | Y origin offset | `^LT0` |
| `^FO` | Field origin (x,y) | `^FO50,50` |
| `^FD` | Field data | `^FDZ0000001` |
| `^FS` | Field separator | `^FS` |
| `^A@` | Scalable font | `^A@N,30,30` |
| `^BC` | Code 128 barcode | `^BCN,100,Y,N` |
| `^BY` | Barcode module width | `^BY2` |
| `^PQ` | Print quantity | `^PQ1` |

## 3. Layout Configuration

### LabelLayout struct

```rust
pub struct LabelLayout {
    pub label_width_dots: u32,   // 400 for 5cm at 203 DPI
    pub label_height_dots: u32,  // 400 for 5cm at 203 DPI
    pub columns: u32,            // 2
    pub margin_x: u32,           // 50 dots
    pub margin_y: u32,           // 50 dots
    pub barcode_height: u32,     // 100 dots
    pub title_font_size: u32,    // 30 dots
    pub code_font_size: u32,     // 25 dots
    pub dpi: u32,                // 203
}
```

### Default values (203 DPI, 5x5 cm)

| Parameter | Value | Notes |
|-----------|-------|-------|
| label_width_dots | 400 | 5cm = 1.9685in × 203 ≈ 400 |
| label_height_dots | 400 | Same as width |
| columns | 2 | Two labels per row |
| margin_x | 50 | ~6mm margin |
| margin_y | 50 | ~6mm margin |
| barcode_height | 100 | ~12mm |
| title_font_size | 30 | ~3.7mm height |
| code_font_size | 25 | ~3.1mm height |

**Note:** These are initial approximations. Real calibration may require adjustments per printer. All values are configurable.

## 4. Dot Calculations

```
5 cm = 1.9685 inches
1.9685 × 203 DPI = 400 dots

10 cm (two labels) = 800 dots
```

For 300 DPI:
```
5 cm = 1.9685 inches
1.9685 × 300 DPI = 591 dots
```

## 5. Two-Column Layout

For a 10cm-wide roll with two 5cm labels per row:

```
┌──────────────────────────────────────────┐
│                  10 cm (800 dots)        │
├──────────────────────┬───────────────────┤
│  5cm (400 dots)      │  5cm (400 dots)  │
│                      │                   │
│  Left column         │  Right column     │
│  X offset: 0         │  X offset: 400   │
│                      │                   │
├──────────────────────┼───────────────────┤
│  Left column         │  Right column     │
│  Y offset: +400     │  Y offset: +400   │
│                      │                   │
└──────────────────────┴───────────────────┘
```

### Position assignment

- Label N (odd) → Left column: X = 0
- Label N (even) → Right column: X = label_width_dots
- Row index = (N - 1) / 2
- Y offset = margin_y + row_index * label_height_dots

## 6. Example ZPL Output

For 4 labels (Z0000001 to Z0000004) with timestamp "26/08/2026 07:15:32":

```zpl
^XA

^PW800
^LL800

; --- Row 1, Left (Z0000001) ---
^FO50,50
^A@N,30,30
^FD26/08/2026 07:15:32
^FS

^FO50,100
^BY2
^BCN,100,Y,N
^FDZ0000001
^FS

^FO50,220
^A@N,25,25
^FDZ0000001
^FS

; --- Row 1, Right (Z0000002) ---
^FO450,50
^A@N,30,30
^FD26/08/2026 07:15:32
^FS

^FO450,100
^BY2
^BCN,100,Y,N
^FDZ0000002
^FS

^FO450,220
^A@N,25,25
^FDZ0000002
^FS

; --- Row 2, Left (Z0000003) ---
^FO50,450
^A@N,30,30
^FD26/08/2026 07:15:32
^FS

^FO50,500
^BY2
^BCN,100,Y,N
^FDZ0000003
^FS

^FO50,620
^A@N,25,25
^FDZ0000003
^FS

; --- Row 2, Right (Z0000004) ---
^FO450,450
^A@N,30,30
^FD26/08/2026 07:15:32
^FS

^FO450,500
^BY2
^BCN,100,Y,N
^FDZ0000004
^FS

^FO450,620
^A@N,25,25
^FDZ0000004
^FS

^XZ
```

## 7. Label Content

Each label contains:

```
┌──────────────────────┐
│  26/08/2026 07:15:32 │  ← Title (print timestamp)
│                      │
│      █████████       │  ← Code 128 barcode
│      █████████       │
│      █████████       │
│                      │
│       Z0000001       │  ← Human-readable code
└──────────────────────┘
```

### Timestamp format
- Format: `DD/MM/YYYY HH:MM:SS`
- Timezone: Local system timezone (obtained in Rust via `chrono::Local`)
- Generated at print time, not stored

### Barcode
- Type: Code 128 (`^BCN`)
- Orientation: Normal (N)
- Module width: 2 (`^BY2`)
- Height: 100 dots (configurable)

## 8. Odd Quantity Handling

For quantity = 3 (Z0000001, Z0000002, Z0000003):

```
Row 1: Z0000001 (left) | Z0000002 (right)
Row 2: Z0000003 (left) | [empty - no ZPL generated]
```

The generator skips the second position entirely when there is no label to print. No phantom ZPL for non-existent labels.

## 9. Generator Interface

```rust
pub struct ZplGenerator {
    layout: LabelLayout,
}

impl ZplGenerator {
    pub fn new(layout: LabelLayout) -> Self;

    pub fn generate_batch(
        &self,
        labels: &[(String, LabelPosition)],  // (code, position)
        timestamp: &str,                      // formatted datetime
    ) -> String;                              // complete ZPL string
}

pub struct LabelPosition {
    pub row: u32,     // 0-based row index
    pub column: u32,  // 0 = left, 1 = right
}
```

## 10. Validation

The ZPL generator does NOT validate printer compatibility. It produces standard ZPL II that works on Zebra printers supporting:
- Code 128
- Scalable fonts (`^A@`)
- Standard field commands

If a printer does not support a command, it will be silently ignored by the printer (Zebra's default behavior).
