use std::sync::Arc;

use crate::domain::repositories::printer_repository::PrinterRepository;
use crate::errors::application_error::ApplicationError;
use crate::infrastructure::printer::printer_transport::PrinterTransport;

pub struct TestPrinter {
    printer_repository: Arc<dyn PrinterRepository>,
    transport: Arc<dyn PrinterTransport>,
}

impl TestPrinter {
    pub fn new(
        printer_repository: Arc<dyn PrinterRepository>,
        transport: Arc<dyn PrinterTransport>,
    ) -> Self {
        Self {
            printer_repository,
            transport,
        }
    }

    pub async fn execute(&self, printer_id: &str) -> Result<bool, ApplicationError> {
        self.printer_repository
            .find_by_id(printer_id)
            .await?
            .ok_or(ApplicationError::PrinterNotConfigured)?;

        self.transport.test_connection().await?;

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::printer::Printer;
    use crate::domain::value_objects::printer_config::{ConnectionType, PrinterConfig};
    use crate::errors::domain_error::DomainError;
    use crate::errors::infrastructure_error::InfrastructureError;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct InMemoryPrinterRepository {
        printers: Mutex<HashMap<String, Printer>>,
    }

    impl InMemoryPrinterRepository {
        fn new(printers: Vec<Printer>) -> Self {
            let map = printers
                .into_iter()
                .map(|p| (p.id.clone(), p))
                .collect::<HashMap<_, _>>();
            Self {
                printers: Mutex::new(map),
            }
        }
    }

    #[async_trait]
    impl PrinterRepository for InMemoryPrinterRepository {
        async fn find_by_id(&self, id: &str) -> Result<Option<Printer>, DomainError> {
            Ok(self.printers.lock().unwrap().get(id).cloned())
        }

        async fn find_all(&self) -> Result<Vec<Printer>, DomainError> {
            Ok(self.printers.lock().unwrap().values().cloned().collect())
        }

        async fn save(&self, printer: &Printer) -> Result<(), DomainError> {
            self.printers
                .lock()
                .unwrap()
                .insert(printer.id.clone(), printer.clone());
            Ok(())
        }

        async fn update(&self, id: &str, config: &PrinterConfig) -> Result<(), DomainError> {
            if let Some(printer) = self.printers.lock().unwrap().get_mut(id) {
                printer.update(config);
            }
            Ok(())
        }

        async fn delete(&self, id: &str) -> Result<(), DomainError> {
            self.printers.lock().unwrap().remove(id);
            Ok(())
        }
    }

    struct FakePrinterTransport {
        fail: bool,
    }

    #[async_trait]
    impl PrinterTransport for FakePrinterTransport {
        async fn send(&self, _data: &[u8]) -> Result<(), InfrastructureError> {
            Ok(())
        }

        async fn test_connection(&self) -> Result<(), InfrastructureError> {
            if self.fail {
                Err(InfrastructureError::PrinterConnection(
                    "connection reset".to_string(),
                ))
            } else {
                Ok(())
            }
        }
    }

    fn valid_printer() -> Printer {
        let config = PrinterConfig::new(
            "Test Printer",
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
        Printer::new("printer-1", &config)
    }

    #[tokio::test]
    async fn test_connection_ok() {
        let repo: Arc<dyn PrinterRepository> =
            Arc::new(InMemoryPrinterRepository::new(vec![valid_printer()]));
        let transport: Arc<dyn PrinterTransport> = Arc::new(FakePrinterTransport { fail: false });
        let use_case = TestPrinter::new(repo, transport);

        let result = use_case.execute("printer-1").await;
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_connection_fails() {
        let repo: Arc<dyn PrinterRepository> =
            Arc::new(InMemoryPrinterRepository::new(vec![valid_printer()]));
        let transport: Arc<dyn PrinterTransport> = Arc::new(FakePrinterTransport { fail: true });
        let use_case = TestPrinter::new(repo, transport);

        let result = use_case.execute("printer-1").await;
        assert!(matches!(
            result,
            Err(ApplicationError::Infrastructure(
                InfrastructureError::PrinterConnection(_)
            ))
        ));
    }

    #[tokio::test]
    async fn test_printer_not_configured() {
        let repo: Arc<dyn PrinterRepository> = Arc::new(InMemoryPrinterRepository::new(vec![]));
        let transport: Arc<dyn PrinterTransport> = Arc::new(FakePrinterTransport { fail: false });
        let use_case = TestPrinter::new(repo, transport);

        let result = use_case.execute("missing").await;
        assert!(matches!(
            result,
            Err(ApplicationError::PrinterNotConfigured)
        ));
    }
}
