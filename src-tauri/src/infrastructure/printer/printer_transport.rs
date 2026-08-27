use async_trait::async_trait;

use crate::errors::infrastructure_error::InfrastructureError;

#[async_trait]
pub trait PrinterTransport: Send + Sync {
    async fn send(&self, data: &[u8]) -> Result<(), InfrastructureError>;

    async fn test_connection(&self) -> Result<(), InfrastructureError>;
}
