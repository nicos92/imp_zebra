use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;

use crate::errors::infrastructure_error::InfrastructureError;

pub async fn create_pool(app_handle: &AppHandle) -> Result<SqlitePool, InfrastructureError> {
    let db_path = get_db_path(app_handle);

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            InfrastructureError::Database(format!("Failed to create DB directory: {}", e))
        })?;
    }

    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .map_err(|e| InfrastructureError::Database(format!("Failed to connect: {}", e)))?;

    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await
        .map_err(|e| InfrastructureError::Database(format!("Failed to set WAL: {}", e)))?;

    sqlx::query("PRAGMA busy_timeout=5000")
        .execute(&pool)
        .await
        .map_err(|e| InfrastructureError::Database(format!("Failed to set busy_timeout: {}", e)))?;

    sqlx::query("PRAGMA foreign_keys=ON")
        .execute(&pool)
        .await
        .map_err(|e| InfrastructureError::Database(format!("Failed to enable FK: {}", e)))?;

    crate::infrastructure::database::migrations::run_migrations(&pool).await?;

    Ok(pool)
}

fn get_db_path(app_handle: &AppHandle) -> PathBuf {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir");
    data_dir.join("zebra-printer.db")
}
