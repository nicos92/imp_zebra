use std::sync::Arc;

use crate::application::dto::printer_dto::PrinterDto;
use crate::domain::repositories::printer_repository::PrinterRepository;
use crate::errors::application_error::ApplicationError;

pub struct GetConfiguredPrinter {
    repository: Arc<dyn PrinterRepository>,
}

impl GetConfiguredPrinter {
    pub fn new(repository: Arc<dyn PrinterRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<Option<PrinterDto>, ApplicationError> {
        let printers = self.repository.find_all().await?;
        Ok(printers.into_iter().next().map(|p| PrinterDto {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::printer::Printer;
    use crate::domain::value_objects::printer_config::{ConnectionType, PrinterConfig};
    use crate::errors::domain_error::DomainError;
    use std::sync::Mutex;

    pub struct FakePrinterRepository {
        printers: Mutex<Vec<Printer>>,
    }

    impl FakePrinterRepository {
        fn new(printers: Vec<Printer>) -> Self {
            Self {
                printers: Mutex::new(printers),
            }
        }
    }

    #[async_trait::async_trait]
    impl PrinterRepository for FakePrinterRepository {
        async fn find_by_id(&self, id: &str) -> Result<Option<Printer>, DomainError> {
            Ok(self
                .printers
                .lock()
                .unwrap()
                .iter()
                .find(|p| p.id == id)
                .cloned())
        }

        async fn find_all(&self) -> Result<Vec<Printer>, DomainError> {
            Ok(self.printers.lock().unwrap().clone())
        }

        async fn save(&self, _printer: &Printer) -> Result<(), DomainError> {
            Ok(())
        }

        async fn update(&self, _id: &str, _config: &PrinterConfig) -> Result<(), DomainError> {
            Ok(())
        }

        async fn delete(&self, _id: &str) -> Result<(), DomainError> {
            Ok(())
        }
    }

    fn make_printer(id: &str) -> Printer {
        let config = PrinterConfig::new(
            "Zebra ZT410",
            "ZT410",
            203,
            5.0,
            5.0,
            2,
            ConnectionType::Tcp,
            "192.168.1.50",
            9100,
        )
        .unwrap();
        Printer::new(id, &config)
    }

    #[tokio::test]
    async fn test_get_configured_printer_returns_first() {
        let repo = Arc::new(FakePrinterRepository::new(vec![
            make_printer("printer-1"),
            make_printer("printer-2"),
        ]));
        let use_case = GetConfiguredPrinter::new(repo);

        let result = use_case.execute().await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, "printer-1");
    }

    #[tokio::test]
    async fn test_get_configured_printer_none_when_empty() {
        let repo = Arc::new(FakePrinterRepository::new(Vec::new()));
        let use_case = GetConfiguredPrinter::new(repo);

        let result = use_case.execute().await.unwrap();
        assert!(result.is_none());
    }
}
