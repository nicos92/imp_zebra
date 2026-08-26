# Database

## 1. Engine

SQLite via SQLx with async support and compile-time checked queries.

## 2. Schema

### 2.1 sequence_state

Stores the sequence counter as a single row.

```sql
CREATE TABLE IF NOT EXISTS sequence_state (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    last_used_code TEXT NOT NULL DEFAULT 'Z0000000',
    updated_at TEXT NOT NULL
);
```

| Column | Type | Constraint | Default | Description |
|--------|------|-----------|---------|-------------|
| id | INTEGER | PRIMARY KEY, CHECK(id=1) | - | Always 1 (single row) |
| last_used_code | TEXT | NOT NULL | 'Z0000000' | Last printed code |
| updated_at | TEXT | NOT NULL | - | ISO 8601 timestamp |

**Design decision:** Store `last_used_code` (not `next_code`). The next code is always `increment(last_used_code)`. This avoids off-by-one errors and makes the invariant simple to verify.

**Initial state:** `Z0000000` means the first call to `increment()` produces `Z0000001`.

### 2.2 printers

Stores printer configuration.

```sql
CREATE TABLE IF NOT EXISTS printers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    model TEXT NOT NULL,
    dpi INTEGER NOT NULL DEFAULT 203,
    label_width_mm REAL NOT NULL DEFAULT 50.0,
    label_height_mm REAL NOT NULL DEFAULT 50.0,
    columns INTEGER NOT NULL DEFAULT 2,
    connection_type TEXT NOT NULL DEFAULT 'tcp',
    ip_address TEXT NOT NULL,
    port INTEGER NOT NULL DEFAULT 9100,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

| Column | Type | Constraint | Default | Description |
|--------|------|-----------|---------|-------------|
| id | TEXT | PRIMARY KEY | - | UUID or nanoid |
| name | TEXT | NOT NULL | - | User-friendly name |
| model | TEXT | NOT NULL | - | Zebra model (ZT410, ZT411, etc.) |
| dpi | INTEGER | NOT NULL | 203 | Print resolution |
| label_width_mm | REAL | NOT NULL | 50.0 | Single label width in mm |
| label_height_mm | REAL | NOT NULL | 50.0 | Single label height in mm |
| columns | INTEGER | NOT NULL | 2 | Labels per row (2 for dual column) |
| connection_type | TEXT | NOT NULL | 'tcp' | 'tcp', 'usb', 'serial' (future) |
| ip_address | TEXT | NOT NULL | - | Printer IP address |
| port | INTEGER | NOT NULL | 9100 | TCP port |
| created_at | TEXT | NOT NULL | - | ISO 8601 timestamp |
| updated_at | TEXT | NOT NULL | - | ISO 8601 timestamp |

### 2.3 print_jobs

Stores print job history as ranges.

```sql
CREATE TABLE IF NOT EXISTS print_jobs (
    id TEXT PRIMARY KEY,
    printer_id TEXT NOT NULL,
    start_code TEXT NOT NULL,
    end_code TEXT NOT NULL,
    quantity INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TEXT NOT NULL,
    completed_at TEXT,
    FOREIGN KEY (printer_id) REFERENCES printers(id)
);
```

| Column | Type | Constraint | Default | Description |
|--------|------|-----------|---------|-------------|
| id | TEXT | PRIMARY KEY | - | UUID or nanoid |
| printer_id | TEXT | NOT NULL, FK | - | References printers.id |
| start_code | TEXT | NOT NULL | - | First code in range |
| end_code | TEXT | NOT NULL | - | Last code in range |
| quantity | INTEGER | NOT NULL | - | Number of labels |
| status | TEXT | NOT NULL | 'pending' | pending/printing/completed/failed |
| created_at | TEXT | NOT NULL | - | ISO 8601 timestamp |
| completed_at | TEXT | NULL | NULL | ISO 8601 timestamp when completed |

**Status values:**
- `pending`: Job created, sequence reserved, ZPL not yet sent
- `printing`: ZPL sent to TCP, awaiting transport confirmation
- `completed`: TCP send() succeeded (does not guarantee physical print)
- `failed`: TCP connection or send error

**Design decision:** Store ranges instead of one row per label. For a batch of 10,000 labels: 1 row vs 10,000 rows. Simplifies history queries and reduces DB size.

## 3. Indices

```sql
CREATE INDEX IF NOT EXISTS idx_print_jobs_printer_id ON print_jobs(printer_id);
CREATE INDEX IF NOT EXISTS idx_print_jobs_status ON print_jobs(status);
CREATE INDEX IF NOT EXISTS idx_print_jobs_created_at ON print_jobs(created_at);
```

## 4. Initial Seed

```sql
INSERT OR IGNORE INTO sequence_state (id, last_used_code, updated_at)
VALUES (1, 'Z0000000', '2026-01-01T00:00:00Z');
```

## 5. Migrations

Migration files are stored in `src-tauri/migrations/` and executed automatically at app startup via SQLx's migration runner.

```
src-tauri/migrations/
└── 001_initial.sql    (schema + seed)
```

## 6. Concurrent Access

SQLite is configured with:
- WAL mode (Write-Ahead Logging) for concurrent reads
- `busy_timeout = 5000` ms to handle write contention

Sequence reservation uses a transaction:
```sql
BEGIN;
-- Read current last_used_code
SELECT last_used_code FROM sequence_state WHERE id = 1;
-- Update to new value (calculated in Rust)
UPDATE sequence_state SET last_used_code = ?, updated_at = ? WHERE id = 1;
-- Insert print job
INSERT INTO print_jobs (...) VALUES (...);
COMMIT;
```

If two transactions compete for the same sequence, SQLite locks the second one until the first commits. The second transaction then reads the updated `last_used_code` and calculates the next range correctly.

## 7. Database Location

Tauri places the database in the platform-specific app data directory:
- Windows: `%APPDATA%/zebra-printer/zebra-printer.db`
- macOS: `~/Library/Application Support/zebra-printer/zebra-printer.db`
- Linux: `~/.local/share/zebra-printer/zebra-printer.db`
