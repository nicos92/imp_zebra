use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::timeout;

use async_trait::async_trait;

use crate::errors::infrastructure_error::InfrastructureError;
use crate::infrastructure::printer::printer_transport::PrinterTransport;

pub struct TcpPrinterTransport {
    address: String,
    connect_timeout: Duration,
    write_timeout: Duration,
}

impl TcpPrinterTransport {
    pub fn new(ip: &str, port: u16) -> Self {
        Self::new_with_timeouts(ip, port, Duration::from_secs(5), Duration::from_secs(30))
    }

    fn new_with_timeouts(
        ip: &str,
        port: u16,
        connect_timeout: Duration,
        write_timeout: Duration,
    ) -> Self {
        Self {
            address: format!("{}:{}", ip, port),
            connect_timeout,
            write_timeout,
        }
    }
}

#[async_trait]
impl PrinterTransport for TcpPrinterTransport {
    async fn send(&self, data: &[u8]) -> Result<(), InfrastructureError> {
        let mut stream = timeout(self.connect_timeout, TcpStream::connect(&self.address))
            .await
            .map_err(|_| InfrastructureError::PrinterTimeout)?
            .map_err(|e| {
                InfrastructureError::PrinterConnection(format!(
                    "Failed to connect to {}: {}",
                    self.address, e
                ))
            })?;

        stream
            .set_nodelay(true)
            .map_err(|e| InfrastructureError::PrinterConnection(e.to_string()))?;

        timeout(self.write_timeout, stream.write_all(data))
            .await
            .map_err(|_| InfrastructureError::PrinterTimeout)?
            .map_err(|e| {
                InfrastructureError::PrinterConnection(format!("Failed to send data: {}", e))
            })?;

        timeout(self.write_timeout, stream.flush())
            .await
            .map_err(|_| InfrastructureError::PrinterTimeout)?
            .map_err(|e| {
                InfrastructureError::PrinterConnection(format!("Failed to flush: {}", e))
            })?;

        Ok(())
    }

    async fn test_connection(&self) -> Result<(), InfrastructureError> {
        let stream = timeout(self.connect_timeout, TcpStream::connect(&self.address))
            .await
            .map_err(|_| InfrastructureError::PrinterTimeout)?
            .map_err(|e| {
                InfrastructureError::PrinterConnection(format!(
                    "Failed to connect to {}: {}",
                    self.address, e
                ))
            })?;

        drop(stream);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use std::time::Duration;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_send_to_listener() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let listen_task = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 6];
            use tokio::io::AsyncReadExt;
            let _ = socket.read_exact(&mut buf).await;
            buf
        });

        let transport = TcpPrinterTransport::new(&addr.ip().to_string(), addr.port());
        let result = transport.send(b"^XA^XZ").await;
        assert!(result.is_ok(), "send should succeed: {:?}", result);

        let received = listen_task.await.unwrap();
        assert_eq!(received, b"^XA^XZ".to_vec());
    }

    #[tokio::test]
    async fn test_connection_refused() {
        let addr: SocketAddr = "127.0.0.1:1".parse().unwrap();
        let transport = TcpPrinterTransport::new(&addr.ip().to_string(), addr.port());
        let result = transport.test_connection().await;
        assert!(matches!(
            result,
            Err(InfrastructureError::PrinterConnection(_))
        ));
    }

    #[tokio::test]
    async fn test_timeout_mapping() {
        let transport = TcpPrinterTransport::new_with_timeouts(
            "10.255.255.1",
            9100,
            Duration::from_millis(50),
            Duration::from_secs(1),
        );
        let result = transport.test_connection().await;
        assert!(matches!(result, Err(InfrastructureError::PrinterTimeout)));
    }
}
