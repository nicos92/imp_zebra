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
            status: PrintJobStatus::from_str(&r.5).ok().unwrap_or(PrintJobStatus::Pending),
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
                status: PrintJobStatus::from_str(&r.5).ok().unwrap_or(PrintJobStatus::Pending),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repositories::printer_repository::PrinterRepository;
    use crate::infrastructure::database::repositories::sqlite_printer_repository::SqlitePrinterRepository;

    fn valid_job(id: &str, printer_id: &str, start: &str, end: &str, quantity: u64) -> PrintJob {
        PrintJob::new(id, printer_id, start, end, quantity)
    }

    async fn create_printer(pool: &sqlx::sqlite::SqlitePool, id: &str) {
        let config = crate::domain::value_objects::printer_config::PrinterConfig::new(
            "Test Printer",
            "Zebra ZD421",
            203,
            50.0,
            50.0,
            2,
            crate::domain::value_objects::printer_config::ConnectionType::Tcp,
            "192.168.1.100",
            9100,
        )
        .unwrap();
        let printer = crate::domain::entities::printer::Printer::new(id, &config);
        SqlitePrinterRepository::new(pool.clone())
            .save(&printer)
            .await
            .unwrap();
    }

    async fn repo() -> SqlitePrintJobRepository {
        let pool = crate::infrastructure::database::test_helpers::create_test_pool().await;
        create_printer(&pool, "printer-1").await;
        SqlitePrintJobRepository::new(pool)
    }

    #[tokio::test]
    async fn test_save_and_find_by_id() {
        let repo = repo().await;
        let job = valid_job("job-1", "printer-1", "Z0000001", "Z0000010", 10);

        repo.save(&job).await.unwrap();

        let found = repo.find_by_id("job-1").await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, "job-1");
        assert_eq!(found.printer_id, "printer-1");
        assert_eq!(found.start_code, "Z0000001");
        assert_eq!(found.end_code, "Z0000010");
        assert_eq!(found.quantity, 10);
        assert_eq!(found.status, PrintJobStatus::Pending);
    }

    #[tokio::test]
    async fn test_update_status() {
        let repo = repo().await;
        let job = valid_job("job-1", "printer-1", "Z0000001", "Z0000010", 10);
        repo.save(&job).await.unwrap();

        let completed_at = Some("2026-08-27T12:00:00Z");
        repo.update_status("job-1", PrintJobStatus::Completed, completed_at)
            .await
            .unwrap();

        let found = repo.find_by_id("job-1").await.unwrap().unwrap();
        assert_eq!(found.status, PrintJobStatus::Completed);
        assert!(found.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_find_recent_orders_by_created_at() {
        let repo = repo().await;

        let job1 = valid_job("job-1", "printer-1", "Z0000001", "Z0000003", 3);
        let job2 = valid_job("job-2", "printer-1", "Z0000004", "Z0000006", 3);
        let job3 = valid_job("job-3", "printer-1", "Z0000007", "Z0000009", 3);

        repo.save(&job1).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        repo.save(&job2).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        repo.save(&job3).await.unwrap();

        let recent = repo.find_recent(2).await.unwrap();
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].id, "job-3");
        assert_eq!(recent[1].id, "job-2");
    }

    #[tokio::test]
    async fn test_find_recent_empty() {
        let repo = repo().await;
        let recent = repo.find_recent(10).await.unwrap();
        assert!(recent.is_empty());
    }
}
