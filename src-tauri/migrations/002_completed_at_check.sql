-- Enforce that terminal print jobs (completed/failed) always carry a completed_at.
-- SQLite cannot add NOT NULL / CHECK via ALTER, so rebuild print_jobs and backfill
-- any legacy terminal rows that were missing a completion timestamp.

CREATE TABLE print_jobs_new (
    id TEXT PRIMARY KEY,
    printer_id TEXT NOT NULL,
    start_code TEXT NOT NULL,
    end_code TEXT NOT NULL,
    quantity INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TEXT NOT NULL,
    completed_at TEXT,
    CONSTRAINT terminal_requires_completed_at CHECK (
        status NOT IN ('completed', 'failed') OR completed_at IS NOT NULL
    ),
    FOREIGN KEY (printer_id) REFERENCES printers(id)
);

INSERT INTO print_jobs_new (id, printer_id, start_code, end_code, quantity, status, created_at, completed_at)
SELECT id,
       printer_id,
       start_code,
       end_code,
       quantity,
       status,
       created_at,
       CASE
           WHEN status IN ('completed', 'failed') THEN COALESCE(completed_at, created_at)
           ELSE completed_at
       END
FROM print_jobs;

DROP TABLE print_jobs;

ALTER TABLE print_jobs_new RENAME TO print_jobs;

CREATE INDEX idx_print_jobs_printer_id ON print_jobs(printer_id);
CREATE INDEX idx_print_jobs_status ON print_jobs(status);
CREATE INDEX idx_print_jobs_created_at ON print_jobs(created_at);
