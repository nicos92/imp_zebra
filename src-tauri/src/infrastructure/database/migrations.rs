use sqlx::sqlite::SqlitePool;

use crate::errors::infrastructure_error::InfrastructureError;

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), InfrastructureError> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sequence_state (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            last_used_code TEXT NOT NULL DEFAULT 'Z0000000',
            updated_at TEXT NOT NULL
        );

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

        CREATE INDEX IF NOT EXISTS idx_print_jobs_printer_id ON print_jobs(printer_id);
        CREATE INDEX IF NOT EXISTS idx_print_jobs_status ON print_jobs(status);
        CREATE INDEX IF NOT EXISTS idx_print_jobs_created_at ON print_jobs(created_at);

        INSERT OR IGNORE INTO sequence_state (id, last_used_code, updated_at)
        VALUES (1, 'Z0000000', '2026-01-01T00:00:00Z');
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| InfrastructureError::Database(format!("Migration failed: {}", e)))?;

    Ok(())
}
