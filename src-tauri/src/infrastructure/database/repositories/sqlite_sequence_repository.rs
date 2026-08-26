use sqlx::sqlite::SqlitePool;
use async_trait::async_trait;

use crate::domain::repositories::sequence_repository::SequenceRepository;
use crate::errors::domain_error::DomainError;

pub struct SqliteSequenceRepository {
    pool: SqlitePool,
}

impl SqliteSequenceRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SequenceRepository for SqliteSequenceRepository {
    async fn get_last_used_code(&self) -> Result<String, DomainError> {
        let row: (String,) = sqlx::query_as("SELECT last_used_code FROM sequence_state WHERE id = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(row.0)
    }

    async fn update_last_used_code(&self, code: &str) -> Result<(), DomainError> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE sequence_state SET last_used_code = ?, updated_at = ? WHERE id = 1")
            .bind(code)
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }
}
