# Printing

## 1. Flow

```
User inputs quantity (e.g. 100)
        │
        ▼
Vue → invoke("print_labels", { quantity: 100 })
        │
        ▼
PrintLabelsUseCase.execute(quantity)
        │
        ├─ 1. Read last_used_code from sequence_state
        │
        ├─ 2. Calculate range:
        │     start = increment(last_used_code, 1)
        │     end   = increment(last_used_code, quantity)
        │     (handles Z9999999 → Z0000001 rollover)
        │
        ├─ 3. Transaction:
        │     UPDATE sequence_state SET last_used_code = end
        │     INSERT INTO print_jobs (start_code, end_code, quantity, status='pending')
        │
        ├─ 4. Generate ZPL II for the range (2 columns)
        │
        ├─ 5. Send ZPL to printer via TCP:9100
        │     ├─ Success → status = 'completed', completed_at = now()
        │     └─ Error   → status = 'failed'
        │
        └─ 6. Return PrintJobResult to frontend
```

## 2. Print Job States

```
pending ──► printing ──► completed
                │
                └──► failed
```

| State | Meaning |
|-------|---------|
| `pending` | Job created, sequence reserved, ZPL not yet sent |
| `printing` | ZPL sent to TCP socket, awaiting OS-level send confirmation |
| `completed` | TCP send() returned OK (does NOT guarantee physical print) |
| `failed` | TCP connection error, timeout, or send failure |

## 3. Consistency Guarantees

### What IS guaranteed:
- Sequence numbers are never duplicated across concurrent requests
- If the app crashes after reserving sequence but before sending ZPL, the job remains `pending`/`printing` (reviewable in history)
- Sequence numbers are never skipped unnecessarily

### What is NOT guaranteed:
- "Exactly once printing": TCP send() success ≠ physical print. The Zebra may have paper jam, low ink, etc. after receiving data.
- Automatic rollback of sequence on print failure. Once reserved, numbers are "lost" intentionally (better to lose numbers than duplicate them).

## 4. Edge Cases

### 4.1 App closes after sending ZPL

```
App sends ZPL via TCP
    ↓
Zebra receives and starts printing
    ↓
App closes before marking "completed"
    ↓
print_job stays in "pending" or "printing"
```

**Resolution:** On next startup, the app can detect jobs in `pending`/`printing` state and show them in history. No automatic re-send.

### 4.2 State saved, print fails physically

```
App marks "completed"
    ↓
Zebra has paper jam
    ↓
Label not physically printed
```

**Resolution:** Known limitation. User must verify physical printing. The app can display a warning: "Verify that printing completed successfully."

### 4.3 Rollover across Z9999999

```
last_used_code: Z9999998
quantity: 3

Range:
  Z9999999 (increment 1)
  Z0000001 (increment 2, wraps)
  Z0000002 (increment 3)
```

Handled by the `Sequence::increment_range()` method. Tested explicitly.

### 4.4 Odd quantities with 2-column layout

```
quantity: 3

Row 1: Z0000001 (left) | Z0000002 (right)
Row 2: Z0000003 (left) | [empty]
```

The ZPL generator skips the second position when there is no label to print.

## 5. Concurrency

### Problem
Two simultaneous requests must not receive overlapping ranges.

```
Request A: 100 labels
Request B: 200 labels

WRONG: A gets Z0000001-Z0000100, B also gets Z0000001-Z0000200
RIGHT: A gets Z0000001-Z0000100, B gets Z0000101-Z0000300
```

### Solution

SQLite transactions with WAL mode:

```sql
-- Transaction A
BEGIN;
SELECT last_used_code FROM sequence_state WHERE id = 1;  -- reads Z0000000
-- Rust calculates: Z0000001 to Z0000100
UPDATE sequence_state SET last_used_code = 'Z0000100' WHERE id = 1;
INSERT INTO print_jobs (...) VALUES ('Z0000001', 'Z0000100', 100, 'pending');
COMMIT;

-- Transaction B (waits for A to commit)
BEGIN;
SELECT last_used_code FROM sequence_state WHERE id = 1;  -- reads Z0000100
-- Rust calculates: Z0000101 to Z0000300
UPDATE sequence_state SET last_used_code = 'Z0000300' WHERE id = 1;
INSERT INTO print_jobs (...) VALUES ('Z0000101', 'Z0000300', 200, 'pending');
COMMIT;
```

SQLx with `busy_timeout=5000` handles the lock contention automatically.

## 6. ZPL Generation

The ZPL is generated entirely in Rust by `ZplGenerator`. The generator receives:
- A list of (code, position) pairs
- A `LabelLayout` configuration

And produces a ZPL II string ready to send to the printer.

See [ZPL.md](./ZPL.md) for ZPL command details.

## 7. Transport

The printer transport is abstracted behind the `PrinterTransport` trait in `infrastructure/printer/printer_transport.rs`:

```rust
#[async_trait]
pub trait PrinterTransport: Send + Sync {
    async fn send(&self, data: &[u8]) -> Result<(), InfrastructureError>;
    async fn test_connection(&self) -> Result<(), InfrastructureError>;
}
```

The TCP implementation `TcpPrinterTransport` connects to `ip:port` (default 9100) and writes raw bytes. No protocol overhead, no handshake — just raw TCP socket to the Zebra printer.

**Timeout:** Configurable via `new(ip, port)` (5s connect, 30s write) or `new_with_timeouts(...)` for tests. Errors map to `InfrastructureError::PrinterConnection` (connect/write failures) and `InfrastructureError::PrinterTimeout` (timeouts).

`ZebraPrinter` depends on `Arc<dyn PrinterTransport>`, injecting `TcpPrinterTransport` by default. Tests use a fake transport, so no physical Zebra is required.

## 8. Validation

Backend validates before printing:
- `quantity > 0`
- `quantity <= MAX_QUANTITY` (configurable, default 10000)
- Printer is configured (exists in DB)
- Printer IP and port are valid
- Connection succeeds (during `test_printer_connection`)
