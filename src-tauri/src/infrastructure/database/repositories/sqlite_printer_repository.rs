use sqlx::sqlite::SqlitePool;
use async_trait::async_trait;

use crate::domain::entities::printer::Printer;
use crate::domain::repositories::printer_repository::PrinterRepository;
use crate::domain::value_objects::printer_config::{ConnectionType, PrinterConfig};
use crate::errors::domain_error::DomainError;

pub struct SqlitePrinterRepository {
    pool: SqlitePool,
}

impl SqlitePrinterRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PrinterRepository for SqlitePrinterRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Printer>, DomainError> {
        let row: Option<(String, String, String, i64, f64, f64, i64, String, String, i64, String, String)> =
            sqlx::query_as(
                "SELECT id, name, model, dpi, label_width_mm, label_height_mm, columns, connection_type, ip_address, port, created_at, updated_at FROM printers WHERE id = ?"
            )
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(row.map(|r| Printer {
            id: r.0,
            name: r.1,
            model: r.2,
            dpi: r.3 as u32,
            label_width_mm: r.4,
            label_height_mm: r.5,
            columns: r.6 as u32,
            connection_type: ConnectionType::from_str(&r.7).unwrap_or(ConnectionType::Tcp),
            ip_address: r.8,
            port: r.9 as u16,
            created_at: chrono::DateTime::parse_from_rfc3339(&r.10)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            updated_at: chrono::DateTime::parse_from_rfc3339(&r.11)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
        }))
    }

    async fn find_all(&self) -> Result<Vec<Printer>, DomainError> {
        let rows: Vec<(String, String, String, i64, f64, f64, i64, String, String, i64, String, String)> =
            sqlx::query_as(
                "SELECT id, name, model, dpi, label_width_mm, label_height_mm, columns, connection_type, ip_address, port, created_at, updated_at FROM printers"
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| Printer {
                id: r.0,
                name: r.1,
                model: r.2,
                dpi: r.3 as u32,
                label_width_mm: r.4,
                label_height_mm: r.5,
                columns: r.6 as u32,
                connection_type: ConnectionType::from_str(&r.7).unwrap_or(ConnectionType::Tcp),
                ip_address: r.8,
                port: r.9 as u16,
                created_at: chrono::DateTime::parse_from_rfc3339(&r.10)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
                updated_at: chrono::DateTime::parse_from_rfc3339(&r.11)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
            })
            .collect())
    }

    async fn save(&self, printer: &Printer) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO printers (id, name, model, dpi, label_width_mm, label_height_mm, columns, connection_type, ip_address, port, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&printer.id)
        .bind(&printer.name)
        .bind(&printer.model)
        .bind(printer.dpi as i64)
        .bind(printer.label_width_mm)
        .bind(printer.label_height_mm)
        .bind(printer.columns as i64)
        .bind(printer.connection_type.as_str())
        .bind(&printer.ip_address)
        .bind(printer.port as i64)
        .bind(printer.created_at.to_rfc3339())
        .bind(printer.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn update(&self, id: &str, config: &PrinterConfig) -> Result<(), DomainError> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE printers SET name = ?, model = ?, dpi = ?, label_width_mm = ?, label_height_mm = ?, columns = ?, connection_type = ?, ip_address = ?, port = ?, updated_at = ? WHERE id = ?"
        )
        .bind(&config.name)
        .bind(&config.model)
        .bind(config.dpi as i64)
        .bind(config.label_width_mm)
        .bind(config.label_height_mm)
        .bind(config.columns as i64)
        .bind(config.connection_type.as_str())
        .bind(&config.ip_address)
        .bind(config.port as i64)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM printers WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_printer(id: &str, name: &str) -> Printer {
        let config = crate::domain::value_objects::printer_config::PrinterConfig::new(
            name,
            "Zebra ZD421",
            203,
            50.0,
            50.0,
            2,
            ConnectionType::Tcp,
            "192.168.1.100",
            9100,
        )
        .unwrap();
        Printer::new(id, &config)
    }

    async fn repo() -> SqlitePrinterRepository {
        let pool = crate::infrastructure::database::test_helpers::create_test_pool().await;
        SqlitePrinterRepository::new(pool)
    }

    #[tokio::test]
    async fn test_save_and_find_by_id() {
        let repo = repo().await;
        let printer = valid_printer("printer-1", "Test Printer");

        repo.save(&printer).await.unwrap();

        let found = repo.find_by_id("printer-1").await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, "printer-1");
        assert_eq!(found.name, "Test Printer");
        assert_eq!(found.model, "Zebra ZD421");
        assert_eq!(found.dpi, 203);
        assert_eq!(found.ip_address, "192.168.1.100");
        assert_eq!(found.port, 9100);
        assert_eq!(found.columns, 2);
    }

    #[tokio::test]
    async fn test_find_all() {
        let repo = repo().await;
        let p1 = valid_printer("printer-1", "Printer One");
        let p2 = valid_printer("printer-2", "Printer Two");

        repo.save(&p1).await.unwrap();
        repo.save(&p2).await.unwrap();

        let all = repo.find_all().await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_update() {
        let repo = repo().await;
        let printer = valid_printer("printer-1", "Test Printer");
        repo.save(&printer).await.unwrap();

        let new_config = crate::domain::value_objects::printer_config::PrinterConfig::new(
            "Updated Printer",
            "Zebra ZD421",
            300,
            100.0,
            150.0,
            1,
            ConnectionType::Tcp,
            "192.168.1.101",
            9100,
        )
        .unwrap();

        repo.update("printer-1", &new_config).await.unwrap();

        let found = repo.find_by_id("printer-1").await.unwrap().unwrap();
        assert_eq!(found.name, "Updated Printer");
        assert_eq!(found.dpi, 300);
        assert_eq!(found.label_width_mm, 100.0);
        assert_eq!(found.columns, 1);
        assert_eq!(found.ip_address, "192.168.1.101");
    }

    #[tokio::test]
    async fn test_delete() {
        let repo = repo().await;
        let printer = valid_printer("printer-1", "Test Printer");
        repo.save(&printer).await.unwrap();

        repo.delete("printer-1").await.unwrap();

        let found = repo.find_by_id("printer-1").await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_find_nonexistent() {
        let repo = repo().await;
        let found = repo.find_by_id("nonexistent").await.unwrap();
        assert!(found.is_none());
    }
}
