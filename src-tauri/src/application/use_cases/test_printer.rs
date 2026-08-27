use std::sync::Arc;

use crate::domain::repositories::printer_repository::PrinterRepository;
use crate::errors::application_error::ApplicationError;
use crate::infrastructure::printer::printer_transport::PrinterTransport;
use crate::infrastructure::printer::tcp_transport::TcpPrinterTransport;

pub struct TestPrinter {
    printer_repository: Arc<dyn PrinterRepository>,
}

impl TestPrinter {
    pub fn new(printer_repository: Arc<dyn PrinterRepository>) -> Self {
        Self { printer_repository }
    }

    pub async fn execute(&self, printer_id: &str) -> Result<bool, ApplicationError> {
        let printer = self
            .printer_repository
            .find_by_id(printer_id)
            .await?
            .ok_or(ApplicationError::PrinterNotConfigured)?;

        let transport = TcpPrinterTransport::new(&printer.ip_address, printer.port);
        transport
            .test_connection()
            .await
            .map_err(|e| ApplicationError::Infrastructure(e))?;

        Ok(true)
    }
}
