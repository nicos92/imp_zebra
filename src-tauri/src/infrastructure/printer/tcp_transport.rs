use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::errors::infrastructure_error::InfrastructureError;

pub struct TcpTransport {
    address: String,
    connect_timeout: Duration,
    write_timeout: Duration,
}

impl TcpTransport {
    pub fn new(ip: &str, port: u16) -> Self {
        Self {
            address: format!("{}:{}", ip, port),
            connect_timeout: Duration::from_secs(5),
            write_timeout: Duration::from_secs(30),
        }
    }

    pub async fn send(&self, data: &[u8]) -> Result<(), InfrastructureError> {
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

    pub async fn test_connection(&self) -> Result<(), InfrastructureError> {
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
