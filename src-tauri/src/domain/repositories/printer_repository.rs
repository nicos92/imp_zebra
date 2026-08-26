use crate::domain::entities::printer::Printer;
use crate::domain::value_objects::printer_config::PrinterConfig;
use crate::errors::domain_error::DomainError;

#[async_trait::async_trait]
pub trait PrinterRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Option<Printer>, DomainError>;
    async fn find_all(&self) -> Result<Vec<Printer>, DomainError>;
    async fn save(&self, printer: &Printer) -> Result<(), DomainError>;
    async fn update(&self, id: &str, config: &PrinterConfig) -> Result<(), DomainError>;
    async fn delete(&self, id: &str) -> Result<(), DomainError>;
}
