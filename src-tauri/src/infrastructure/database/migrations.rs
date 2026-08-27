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
}
