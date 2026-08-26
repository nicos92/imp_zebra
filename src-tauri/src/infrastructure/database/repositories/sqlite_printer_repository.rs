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
