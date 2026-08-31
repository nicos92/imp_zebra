use sqlx::sqlite::SqlitePool;

use crate::errors::infrastructure_error::InfrastructureError;

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), InfrastructureError> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| InfrastructureError::Database(format!("Migration failed: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_migrations_run_on_empty_db() {
        let pool = crate::infrastructure::database::test_helpers::create_test_pool().await;

        let tables: Vec<String> =
            sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
                .fetch_all(&pool)
                .await
                .expect("Failed to query tables");

        assert!(tables.contains(&"print_jobs".to_string()));
        assert!(tables.contains(&"printers".to_string()));
        assert!(tables.contains(&"sequence_state".to_string()));

        let seed: (String,) =
            sqlx::query_as("SELECT last_used_code FROM sequence_state WHERE id = 1")
                .fetch_one(&pool)
                .await
                .expect("Failed to fetch seed");

        assert_eq!(seed.0, "Z0000000");
    }

    #[tokio::test]
    async fn test_terminal_job_requires_completed_at() {
        let pool = crate::infrastructure::database::test_helpers::create_test_pool().await;

        let printer_id = "printer-check-1";
        let _ = sqlx::query(
            "INSERT INTO printers (id, name, model, dpi, label_width_mm, label_height_mm, columns, connection_type, ip_address, port, created_at, updated_at) VALUES (?, 'P', 'M', 203, 50.0, 30.0, 2, 'tcp', '192.168.1.1', 9100, '2026-08-01T00:00:00Z', '2026-08-01T00:00:00Z')",
        )
        .bind(printer_id)
        .execute(&pool)
        .await
        .expect("printer seed");

        let insert_terminal_no_ts = sqlx::query(
            "INSERT INTO print_jobs (id, printer_id, start_code, end_code, quantity, status, created_at, completed_at) VALUES ('j1', ?, 'Z0000001', 'Z0000002', 2, 'completed', '2026-08-01T00:00:00Z', NULL)",
        )
        .bind(printer_id)
        .execute(&pool)
        .await;

        assert!(
            insert_terminal_no_ts.is_err(),
            "terminal job without completed_at must be rejected"
        );

        let insert_pending_no_ts = sqlx::query(
            "INSERT INTO print_jobs (id, printer_id, start_code, end_code, quantity, status, created_at, completed_at) VALUES ('j2', ?, 'Z0000003', 'Z0000004', 2, 'pending', '2026-08-01T00:00:00Z', NULL)",
        )
        .bind(printer_id)
        .execute(&pool)
        .await;

        assert!(
            insert_pending_no_ts.is_ok(),
            "pending job may have NULL completed_at"
        );

        let insert_terminal_with_ts = sqlx::query(
            "INSERT INTO print_jobs (id, printer_id, start_code, end_code, quantity, status, created_at, completed_at) VALUES ('j3', ?, 'Z0000005', 'Z0000006', 2, 'completed', '2026-08-01T00:00:00Z', '2026-08-01T00:00:01Z')",
        )
        .bind(printer_id)
        .execute(&pool)
        .await;

        assert!(
            insert_terminal_with_ts.is_ok(),
            "terminal job with completed_at must be accepted"
        );
    }
}
