use crate::errors::domain_error::DomainError;

#[async_trait::async_trait]
pub trait SequenceRepository: Send + Sync {
    async fn get_last_used_code(&self) -> Result<String, DomainError>;
    async fn update_last_used_code(&self, code: &str) -> Result<(), DomainError>;
}
