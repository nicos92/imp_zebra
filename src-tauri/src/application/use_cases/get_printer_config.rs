use std::sync::Arc;

use crate::application::dto::printer_dto::PrinterDto;
use crate::domain::repositories::printer_repository::PrinterRepository;
use crate::errors::application_error::ApplicationError;

pub struct GetPrinterConfig {
    repository: Arc<dyn PrinterRepository>,
}

impl GetPrinterConfig {
    pub fn new(repository: Arc<dyn PrinterRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, id: &str) -> Result<Option<PrinterDto>, ApplicationError> {
        let printer = self.repository.find_by_id(id).await?;
        Ok(printer.map(|p| PrinterDto {
            id: p.id,
            name: p.name,
            model: p.model,
            dpi: p.dpi,
            label_width_mm: p.label_width_mm,
            label_height_mm: p.label_height_mm,
            columns: p.columns,
            connection_type: p.connection_type.as_str().to_string(),
            ip_address: p.ip_address,
            port: p.port,
            created_at: p.created_at.to_rfc3339(),
            updated_at: p.updated_at.to_rfc3339(),
        }))
    }

    pub async fn get_all(&self) -> Result<Vec<PrinterDto>, ApplicationError> {
        let printers = self.repository.find_all().await?;
        Ok(printers
            .into_iter()
            .map(|p| PrinterDto {
                id: p.id,
                name: p.name,
                model: p.model,
                dpi: p.dpi,
                label_width_mm: p.label_width_mm,
                label_height_mm: p.label_height_mm,
                columns: p.columns,
                connection_type: p.connection_type.as_str().to_string(),
                ip_address: p.ip_address,
                port: p.port,
                created_at: p.created_at.to_rfc3339(),
                updated_at: p.updated_at.to_rfc3339(),
            })
            .collect())
    }
}
