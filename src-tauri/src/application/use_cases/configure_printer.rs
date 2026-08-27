use std::sync::Arc;

use crate::application::dto::printer_dto::{PrinterConfigDto, PrinterDto};
use crate::domain::entities::printer::Printer;
use crate::domain::repositories::printer_repository::PrinterRepository;
use crate::domain::value_objects::printer_config::{ConnectionType, PrinterConfig};
use crate::errors::application_error::ApplicationError;

pub struct ConfigurePrinter {
    repository: Arc<dyn PrinterRepository>,
}

impl ConfigurePrinter {
    pub fn new(repository: Arc<dyn PrinterRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, dto: PrinterConfigDto) -> Result<PrinterDto, ApplicationError> {
        let connection_type =
            ConnectionType::from_str(&dto.connection_type).map_err(ApplicationError::Domain)?;

        let config = PrinterConfig::new(
            &dto.name,
            &dto.model,
            dto.dpi,
            dto.label_width_mm,
            dto.label_height_mm,
            dto.columns,
            connection_type,
            &dto.ip_address,
            dto.port,
        )
        .map_err(ApplicationError::Domain)?;

        let id = dto.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let existing = self.repository.find_by_id(&id).await?;

        if let Some(mut printer) = existing {
            printer.update(&config);
            self.repository.update(&id, &config).await?;
            Ok(self.to_dto(&printer))
        } else {
            let printer = Printer::new(&id, &config);
            self.repository.save(&printer).await?;
            Ok(self.to_dto(&printer))
        }
    }

    fn to_dto(&self, printer: &Printer) -> PrinterDto {
        PrinterDto {
            id: printer.id.clone(),
            name: printer.name.clone(),
            model: printer.model.clone(),
            dpi: printer.dpi,
            label_width_mm: printer.label_width_mm,
            label_height_mm: printer.label_height_mm,
            columns: printer.columns,
            connection_type: printer.connection_type.as_str().to_string(),
            ip_address: printer.ip_address.clone(),
            port: printer.port,
            created_at: printer.created_at.to_rfc3339(),
            updated_at: printer.updated_at.to_rfc3339(),
        }
    }
}
