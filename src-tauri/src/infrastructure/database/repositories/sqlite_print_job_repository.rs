use sqlx::sqlite::SqlitePool;
use async_trait::async_trait;

use crate::domain::entities::print_job::{PrintJob, PrintJobStatus};
use crate::domain::repositories::print_job_repository::PrintJobRepository;
use crate::errors::domain_error::DomainError;

pub struct SqlitePrintJobRepository {
    pool: SqlitePool,
}

impl SqlitePrintJobRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PrintJobRepository for SqlitePrintJobRepository {
    async fn save(&self, job: &PrintJob) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO print_jobs (id, printer_id, start_code, end_code, quantity, status, created_at, completed_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&job.id)
        .bind(&job.printer_id)
        .bind(&job.start_code)
        .bind(&job.end_code)
        .bind(job.quantity as i64)
        .bind(job.status.as_str())
        .bind(job.created_at.to_rfc3339())
        .bind(job.completed_at.map(|dt| dt.to_rfc3339()))
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<PrintJob>, DomainError> {
        let row: Option<(String, String, String, String, i64, String, String, Option<String>)> =
            sqlx::query_as(
                "SELECT id, printer_id, start_code, end_code, quantity, status, created_at, completed_at FROM print_jobs WHERE id = ?"
            )
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(row.map(|r| PrintJob {
            id: r.0,
            printer_id: r.1,
            start_code: r.2,
            end_code: r.3,
            quantity: r.4 as u64,
            status: PrintJobStatus::from_str(&r.5).unwrap_or(PrintJobStatus::Pending),
            created_at: chrono::DateTime::parse_from_rfc3339(&r.6)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            completed_at: r.7.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
        }))
    }

    async fn update_status(
        &self,
        id: &str,
        status: PrintJobStatus,
        completed_at: Option<&str>,
    ) -> Result<(), DomainError> {
        sqlx::query("UPDATE print_jobs SET status = ?, completed_at = ? WHERE id = ?")
            .bind(status.as_str())
            .bind(completed_at)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn find_recent(&self, limit: i64) -> Result<Vec<PrintJob>, DomainError> {
        let rows: Vec<(String, String, String, String, i64, String, String, Option<String>)> =
            sqlx::query_as(
                "SELECT id, printer_id, start_code, end_code, quantity, status, created_at, completed_at FROM print_jobs ORDER BY created_at DESC LIMIT ?"
            )
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| PrintJob {
                id: r.0,
                printer_id: r.1,
                start_code: r.2,
                end_code: r.3,
                quantity: r.4 as u64,
                status: PrintJobStatus::from_str(&r.5).unwrap_or(PrintJobStatus::Pending),
                created_at: chrono::DateTime::parse_from_rfc3339(&r.6)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
                completed_at: r.7.and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(&s)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .ok()
                }),
            })
            .collect())
    }
}
