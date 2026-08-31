use async_trait::async_trait;
use sqlx::sqlite::SqlitePool;

use crate::domain::entities::sequence::Sequence;
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
        let row: (String,) =
            sqlx::query_as("SELECT last_used_code FROM sequence_state WHERE id = 1")
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

    async fn reserve_range(&self, quantity: u64) -> Result<(String, String, String), DomainError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        let row: (String,) =
            sqlx::query_as("SELECT last_used_code FROM sequence_state WHERE id = 1")
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| DomainError::Database(e.to_string()))?;

        let mut sequence =
            Sequence::from_code(&row.0).map_err(|e| DomainError::Database(e.to_string()))?;

        let (start, end, _codes) = sequence
            .reserve_range(quantity)
            .map_err(|e| DomainError::Database(e.to_string()))?;

        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE sequence_state SET last_used_code = ?, updated_at = ? WHERE id = 1")
            .bind(sequence.last_used_code())
            .bind(&now)
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok((start, end, sequence.last_used_code()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    async fn repo() -> SqliteSequenceRepository {
        let pool = crate::infrastructure::database::test_helpers::create_test_pool().await;
        SqliteSequenceRepository::new(pool)
    }

    #[tokio::test]
    async fn test_get_initial_code() {
        let repo = repo().await;
        let code = repo.get_last_used_code().await.unwrap();
        assert_eq!(code, "Z0000000");
    }

    #[tokio::test]
    async fn test_update_code() {
        let repo = repo().await;
        repo.update_last_used_code("Z0000050").await.unwrap();

        let code = repo.get_last_used_code().await.unwrap();
        assert_eq!(code, "Z0000050");
    }

    #[tokio::test]
    async fn test_reserve_range() {
        let repo = repo().await;
        let (start, end, new_last) = repo.reserve_range(5).await.unwrap();

        assert_eq!(start, "Z0000001");
        assert_eq!(end, "Z0000005");
        assert_eq!(new_last, "Z0000005");

        let code = repo.get_last_used_code().await.unwrap();
        assert_eq!(code, "Z0000005");
    }

    #[tokio::test]
    async fn test_reserve_range_sequential() {
        let repo = repo().await;
        repo.update_last_used_code("Z9999998").await.unwrap();

        let (start, end, new_last) = repo.reserve_range(5).await.unwrap();

        assert_eq!(start, "Z9999999");
        assert_eq!(end, "Z0000004");
        assert_eq!(new_last, "Z0000004");

        let code = repo.get_last_used_code().await.unwrap();
        assert_eq!(code, "Z0000004");
    }

    #[tokio::test]
    async fn test_concurrent_reserve_ranges_do_not_overlap() {
        let repo = Arc::new(repo().await);
        let tasks = 8;
        let per_task = 25u64;

        let mut handles = Vec::new();
        for _ in 0..tasks {
            let repo = Arc::clone(&repo);
            handles.push(tokio::spawn(async move {
                let (start, end, _new_last) = repo.reserve_range(per_task).await.unwrap();
                (start, end)
            }));
        }

        let mut ranges: Vec<(String, String)> = Vec::new();
        for handle in handles {
            ranges.push(handle.await.unwrap());
        }

        assert_eq!(ranges.len(), tasks);

        let mut values: Vec<(u64, u64)> = ranges
            .iter()
            .map(|(start, end)| {
                let s = Sequence::parse_code(start).unwrap();
                let e = Sequence::parse_code(end).unwrap();
                (s, e)
            })
            .collect();
        values.sort_unstable();

        let expected_total = tasks as u64 * per_task;
        let mut seen = 0u64;
        for (s, e) in values {
            assert_eq!(s, seen + 1, "ranges must be contiguous and non-overlapping");
            assert_eq!(e, s + per_task - 1);
            assert!(e <= Sequence::parse_code("Z9999999").unwrap());
            seen = e;
        }
        assert_eq!(seen, expected_total);
    }
}
