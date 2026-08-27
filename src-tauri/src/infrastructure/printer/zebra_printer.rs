use std::sync::Arc;

use crate::domain::entities::printer::Printer;
use crate::errors::infrastructure_error::InfrastructureError;
use crate::infrastructure::printer::printer_transport::PrinterTransport;
use crate::infrastructure::printer::tcp_transport::TcpPrinterTransport;

pub struct ZebraPrinter {
    printer: Printer,
    transport: Arc<dyn PrinterTransport>,
}

impl ZebraPrinter {
    pub fn new(printer: Printer) -> Self {
        let transport: Arc<dyn PrinterTransport> =
            Arc::new(TcpPrinterTransport::new(&printer.ip_address, printer.port));
        Self { printer, transport }
    }

    fn with_transport(printer: Printer, transport: Arc<dyn PrinterTransport>) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::printer::Printer;
    use crate::domain::value_objects::printer_config::{ConnectionType, PrinterConfig};
    use async_trait::async_trait;

    struct FakePrinterTransport {
        fail: bool,
    }

    #[async_trait]
    impl PrinterTransport for FakePrinterTransport {
        async fn send(&self, _data: &[u8]) -> Result<(), InfrastructureError> {
            if self.fail {
                Err(InfrastructureError::PrinterConnection(
                    "connection reset".to_string(),
                ))
            } else {
                Ok(())
            }
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
    async fn test_send_zpl_ok() {
        let printer = valid_printer();
        let transport: Arc<dyn PrinterTransport> = Arc::new(FakePrinterTransport { fail: false });
        let zebra = ZebraPrinter::with_transport(printer, transport);
        assert!(zebra.send_zpl("^XA^XZ").await.is_ok());
    }

    #[tokio::test]
    async fn test_send_zpl_error() {
        let printer = valid_printer();
        let transport: Arc<dyn PrinterTransport> = Arc::new(FakePrinterTransport { fail: true });
        let zebra = ZebraPrinter::with_transport(printer, transport);
        let result = zebra.send_zpl("^XA^XZ").await;
        assert!(matches!(
            result,
            Err(InfrastructureError::PrinterConnection(_))
        ));
    }

    #[tokio::test]
    async fn test_test_connection_ok() {
        let printer = valid_printer();
        let transport: Arc<dyn PrinterTransport> = Arc::new(FakePrinterTransport { fail: false });
        let zebra = ZebraPrinter::with_transport(printer, transport);
        assert!(zebra.test_connection().await.is_ok());
    }
}
