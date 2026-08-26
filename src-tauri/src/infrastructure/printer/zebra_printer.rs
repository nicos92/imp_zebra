use std::sync::Arc;

use crate::domain::entities::printer::Printer;
use crate::errors::infrastructure_error::InfrastructureError;
use crate::infrastructure::printer::tcp_transport::TcpTransport;

pub struct ZebraPrinter {
    printer: Printer,
    transport: Arc<TcpTransport>,
}

impl ZebraPrinter {
    pub fn new(printer: Printer) -> Self {
        let transport = Arc::new(TcpTransport::new(&printer.ip_address, printer.port));
        Self { printer, transport }
    }

    pub async fn send_zpl(&self, zpl: &str) -> Result<(), InfrastructureError> {
        self.transport.send(zpl.as_bytes()).await
    }

    pub async fn test_connection(&self) -> Result<(), InfrastructureError> {
        self.transport.test_connection().await
    }

    pub fn printer(&self) -> &Printer {
        &self.printer
    }

    pub fn address(&self) -> String {
        self.printer.address()
    }
}
