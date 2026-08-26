use crate::domain::entities::print_job::{PrintJob, PrintJobStatus};
use crate::errors::domain_error::DomainError;

#[async_trait::async_trait]
pub trait PrintJobRepository: Send + Sync {
    async fn save(&self, job: &PrintJob) -> Result<(), DomainError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<PrintJob>, DomainError>;
    async fn update_status(
        &self,
        id: &str,
        status: PrintJobStatus,
        completed_at: Option<&str>,
    ) -> Result<(), DomainError>;
    async fn find_recent(&self, limit: i64) -> Result<Vec<PrintJob>, DomainError>;
}
