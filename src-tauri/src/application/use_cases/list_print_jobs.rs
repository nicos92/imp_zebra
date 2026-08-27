use std::sync::Arc;

use crate::application::dto::print_dto::PrintJobDto;
use crate::domain::repositories::print_job_repository::PrintJobRepository;
use crate::errors::application_error::ApplicationError;

pub struct ListPrintJobs {
    repository: Arc<dyn PrintJobRepository>,
}

const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 500;

impl ListPrintJobs {
    pub fn new(repository: Arc<dyn PrintJobRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, limit: Option<i64>) -> Result<Vec<PrintJobDto>, ApplicationError> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
        let jobs = self.repository.find_recent(limit).await?;
        Ok(jobs.into_iter().map(PrintJobDto::from).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::print_job::{PrintJob, PrintJobStatus};
    use crate::errors::domain_error::DomainError;
    use std::sync::Mutex;

    struct FakePrintJobRepository {
        jobs: Mutex<Vec<PrintJob>>,
        limit_captured: Mutex<Option<i64>>,
    }

    impl FakePrintJobRepository {
        fn new(jobs: Vec<PrintJob>) -> Self {
            Self {
                jobs: Mutex::new(jobs),
                limit_captured: Mutex::new(None),
            }
        }
    }

    #[async_trait::async_trait]
    impl PrintJobRepository for FakePrintJobRepository {
        async fn save(&self, _job: &PrintJob) -> Result<(), DomainError> {
            Ok(())
        }

        async fn find_by_id(&self, _id: &str) -> Result<Option<PrintJob>, DomainError> {
            Ok(None)
        }

        async fn update_status(
            &self,
            _id: &str,
            _status: PrintJobStatus,
            _completed_at: Option<&str>,
        ) -> Result<(), DomainError> {
            Ok(())
        }

        async fn find_recent(&self, limit: i64) -> Result<Vec<PrintJob>, DomainError> {
            *self.limit_captured.lock().unwrap() = Some(limit);
            Ok(self.jobs.lock().unwrap().clone())
        }
    }

    fn make_job(id: &str) -> PrintJob {
        PrintJob::new(id, "printer-1", "Z0000001", "Z0000010", 10)
    }

    #[tokio::test]
    async fn test_list_print_jobs_maps_entities() {
        let repo = Arc::new(FakePrintJobRepository::new(vec![make_job("job-1")]));
        let use_case = ListPrintJobs::new(repo);

        let jobs = use_case.execute(Some(10)).await.unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].id, "job-1");
        assert_eq!(jobs[0].printer_id, "printer-1");
        assert_eq!(jobs[0].start_code, "Z0000001");
        assert_eq!(jobs[0].end_code, "Z0000010");
        assert_eq!(jobs[0].quantity, 10);
        assert_eq!(jobs[0].status, "pending");
    }

    #[tokio::test]
    async fn test_list_print_jobs_empty() {
        let repo = Arc::new(FakePrintJobRepository::new(Vec::new()));
        let use_case = ListPrintJobs::new(repo);

        let jobs = use_case.execute(Some(10)).await.unwrap();
        assert!(jobs.is_empty());
    }

    #[tokio::test]
    async fn test_list_print_jobs_default_limit_when_none() {
        let repo = Arc::new(FakePrintJobRepository::new(Vec::new()));
        let use_case = ListPrintJobs::new(Arc::clone(&repo) as Arc<dyn PrintJobRepository>);

        use_case.execute(None).await.unwrap();
        assert_eq!(*repo.limit_captured.lock().unwrap(), Some(DEFAULT_LIMIT));
    }
}
