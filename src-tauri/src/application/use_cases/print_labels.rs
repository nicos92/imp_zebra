use std::sync::Arc;

use crate::application::dto::print_dto::{PrintRequestDto, PrintResultDto};
use crate::domain::entities::print_job::PrintJob;
use crate::domain::repositories::print_job_repository::PrintJobRepository;
use crate::domain::repositories::printer_repository::PrinterRepository;
use crate::domain::services::label_service::LabelService;
use crate::domain::services::sequence_service::SequenceService;
use crate::errors::application_error::ApplicationError;
use crate::infrastructure::printer::printer_transport::PrinterTransport;
use crate::infrastructure::zpl::generator::ZplGenerator;
use crate::infrastructure::zpl::label_layout::LabelLayout;
use tracing::{event, instrument, Level};

pub struct PrintLabels {
    sequence_service: Arc<SequenceService>,
    printer_repository: Arc<dyn PrinterRepository>,
    print_job_repository: Arc<dyn PrintJobRepository>,
    transport: Arc<dyn PrinterTransport>,
}

impl PrintLabels {
    pub fn new(
        sequence_service: Arc<SequenceService>,
        printer_repository: Arc<dyn PrinterRepository>,
        print_job_repository: Arc<dyn PrintJobRepository>,
        transport: Arc<dyn PrinterTransport>,
    ) -> Self {
        Self {
            sequence_service,
            printer_repository,
            print_job_repository,
            transport,
        }
    }

    #[instrument(skip_all, fields(printer_id = %request.printer_id, quantity = request.quantity))]
    pub async fn execute(
        &self,
        request: PrintRequestDto,
    ) -> Result<PrintResultDto, ApplicationError> {
        let printer = self
            .printer_repository
            .find_by_id(&request.printer_id)
            .await?
            .ok_or(ApplicationError::PrinterNotConfigured)?;

        let (start, end, codes) = self
            .sequence_service
            .reserve_range(request.quantity)
            .await
            .map_err(ApplicationError::Domain)?;

        let job_id = uuid::Uuid::new_v4().to_string();
        let mut job = PrintJob::new(&job_id, &request.printer_id, &start, &end, request.quantity);

        self.print_job_repository.save(&job).await?;
        event!(
            Level::INFO,
            job_id = %job_id,
            start_code = %start,
            end_code = %end,
            "print job created"
        );

        let layout = LabelLayout::from_printer_config(
            printer.label_width_mm,
            printer.label_height_mm,
            printer.columns,
            printer.dpi,
        );
        let generator = ZplGenerator::new(layout);

        let timestamp = chrono::Local::now().format("%d/%m/%Y %H:%M:%S").to_string();

        let labels_with_positions =
            LabelService::new().calculate_positions(&codes, printer.columns);

        let zpl = generator.generate_batch(&labels_with_positions, &timestamp);

        job.start_printing().map_err(ApplicationError::Domain)?;
        self.print_job_repository
            .update_status(&job_id, job.status.clone(), None)
            .await?;
        event!(
            Level::INFO,
            job_id = %job_id,
            "print job started"
        );

        match self.transport.send(zpl.as_bytes()).await {
            Ok(()) => {
                job.complete().map_err(ApplicationError::Domain)?;
                self.print_job_repository
                    .update_status(
                        &job_id,
                        job.status.clone(),
                        Some(&chrono::Utc::now().to_rfc3339()),
                    )
                    .await?;
                event!(
                    Level::INFO,
                    job_id = %job_id,
                    start_code = %start,
                    end_code = %end,
                    "print job completed"
                );
            }
            Err(e) => {
                job.fail().map_err(ApplicationError::Domain)?;
                self.print_job_repository
                    .update_status(
                        &job_id,
                        job.status.clone(),
                        Some(&chrono::Utc::now().to_rfc3339()),
                    )
                    .await?;
                event!(
                    Level::ERROR,
                    job_id = %job_id,
                    error = %e,
                    "print job failed"
                );
                return Err(ApplicationError::PrintJobFailed(e.to_string()));
            }
        }

        Ok(PrintResultDto {
            job_id,
            start_code: start,
            end_code: end,
            quantity: request.quantity,
            status: job.status.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::print_job::{PrintJob, PrintJobStatus};
    use crate::domain::entities::printer::Printer;
    use crate::domain::entities::sequence::Sequence;
    use crate::domain::repositories::print_job_repository::PrintJobRepository;
    use crate::domain::repositories::sequence_repository::SequenceRepository;
    use crate::domain::value_objects::printer_config::{ConnectionType, PrinterConfig};
    use crate::errors::domain_error::DomainError;
    use crate::errors::infrastructure_error::InfrastructureError;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct InMemoryPrinterRepository {
        printers: Mutex<HashMap<String, Printer>>,
    }

    impl InMemoryPrinterRepository {
        fn new(printers: Vec<Printer>) -> Self {
            let map = printers
                .into_iter()
                .map(|p| (p.id.clone(), p))
                .collect::<HashMap<_, _>>();
            Self {
                printers: Mutex::new(map),
            }
        }
    }

    #[async_trait]
    impl PrinterRepository for InMemoryPrinterRepository {
        async fn find_by_id(&self, id: &str) -> Result<Option<Printer>, DomainError> {
            Ok(self.printers.lock().unwrap().get(id).cloned())
        }

        async fn find_all(&self) -> Result<Vec<Printer>, DomainError> {
            Ok(self.printers.lock().unwrap().values().cloned().collect())
        }

        async fn save(&self, printer: &Printer) -> Result<(), DomainError> {
            self.printers
                .lock()
                .unwrap()
                .insert(printer.id.clone(), printer.clone());
            Ok(())
        }

        async fn update(&self, id: &str, config: &PrinterConfig) -> Result<(), DomainError> {
            if let Some(printer) = self.printers.lock().unwrap().get_mut(id) {
                printer.update(config);
            }
            Ok(())
        }

        async fn delete(&self, id: &str) -> Result<(), DomainError> {
            self.printers.lock().unwrap().remove(id);
            Ok(())
        }
    }

    struct InMemorySequenceRepository {
        last_used: Mutex<String>,
    }

    impl InMemorySequenceRepository {
        fn new(initial_code: &str) -> Self {
            Self {
                last_used: Mutex::new(initial_code.to_string()),
            }
        }
    }

    #[async_trait]
    impl SequenceRepository for InMemorySequenceRepository {
        async fn get_last_used_code(&self) -> Result<String, DomainError> {
            Ok(self.last_used.lock().unwrap().clone())
        }

        async fn update_last_used_code(&self, code: &str) -> Result<(), DomainError> {
            *self.last_used.lock().unwrap() = code.to_string();
            Ok(())
        }

        async fn reserve_range(
            &self,
            quantity: u64,
        ) -> Result<(String, String, String), DomainError> {
            let mut seq = Sequence::from_code(&self.get_last_used_code().await?)?;
            let (start, end, _) = seq.reserve_range(quantity)?;
            let new_last = seq.last_used_code();
            *self.last_used.lock().unwrap() = new_last.clone();
            Ok((start, end, new_last))
        }
    }

    struct InMemoryPrintJobRepository {
        jobs: Mutex<HashMap<String, PrintJob>>,
    }

    impl InMemoryPrintJobRepository {
        fn new() -> Self {
            Self {
                jobs: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl PrintJobRepository for InMemoryPrintJobRepository {
        async fn save(&self, job: &PrintJob) -> Result<(), DomainError> {
            self.jobs
                .lock()
                .unwrap()
                .insert(job.id.clone(), job.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: &str) -> Result<Option<PrintJob>, DomainError> {
            Ok(self.jobs.lock().unwrap().get(id).cloned())
        }

        async fn update_status(
            &self,
            id: &str,
            status: PrintJobStatus,
            completed_at: Option<&str>,
        ) -> Result<(), DomainError> {
            if let Some(job) = self.jobs.lock().unwrap().get_mut(id) {
                job.status = status;
                if let Some(ts) = completed_at {
                    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
                        job.completed_at = Some(dt.with_timezone(&chrono::Utc));
                    }
                }
            }
            Ok(())
        }

        async fn find_recent(&self, _limit: i64) -> Result<Vec<PrintJob>, DomainError> {
            Ok(self.jobs.lock().unwrap().values().cloned().collect())
        }
    }

    struct FakePrinterTransport {
        fail: bool,
        sent: Arc<Mutex<Option<Vec<u8>>>>,
    }

    #[async_trait]
    impl PrinterTransport for FakePrinterTransport {
        async fn send(&self, data: &[u8]) -> Result<(), InfrastructureError> {
            if self.fail {
                return Err(InfrastructureError::PrinterConnection(
                    "connection reset".to_string(),
                ));
            }
            *self.sent.lock().unwrap() = Some(data.to_vec());
            Ok(())
        }

        async fn test_connection(&self) -> Result<(), InfrastructureError> {
            Ok(())
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

    fn request(printer_id: &str, quantity: u64) -> PrintRequestDto {
        PrintRequestDto {
            quantity,
            printer_id: printer_id.to_string(),
        }
    }

    #[tokio::test]
    async fn test_print_labels_happy_path() {
        let printer_repo: Arc<dyn PrinterRepository> =
            Arc::new(InMemoryPrinterRepository::new(vec![valid_printer()]));
        let seq_repo: Arc<dyn SequenceRepository> =
            Arc::new(InMemorySequenceRepository::new("Z0000000"));
        let job_repo: Arc<dyn PrintJobRepository> = Arc::new(InMemoryPrintJobRepository::new());
        let sent: Arc<Mutex<Option<Vec<u8>>>> = Arc::new(Mutex::new(None));
        let transport: Arc<dyn PrinterTransport> = Arc::new(FakePrinterTransport {
            fail: false,
            sent: sent.clone(),
        });

        let seq_service = Arc::new(SequenceService::new(seq_repo));
        let use_case = PrintLabels::new(seq_service, printer_repo, job_repo.clone(), transport);

        let result = use_case.execute(request("printer-1", 4)).await.unwrap();
        assert_eq!(result.start_code, "Z0000001");
        assert_eq!(result.end_code, "Z0000004");
        assert_eq!(result.quantity, 4);
        assert_eq!(result.status, "completed");

        let job = job_repo.find_by_id(&result.job_id).await.unwrap().unwrap();
        assert_eq!(job.status, PrintJobStatus::Completed);
        assert!(job.completed_at.is_some());

        assert!(!sent.lock().unwrap().as_ref().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_print_labels_send_failure_marks_failed() {
        let printer_repo: Arc<dyn PrinterRepository> =
            Arc::new(InMemoryPrinterRepository::new(vec![valid_printer()]));
        let seq_repo: Arc<dyn SequenceRepository> =
            Arc::new(InMemorySequenceRepository::new("Z0000000"));
        let job_repo: Arc<dyn PrintJobRepository> = Arc::new(InMemoryPrintJobRepository::new());
        let sent: Arc<Mutex<Option<Vec<u8>>>> = Arc::new(Mutex::new(None));
        let transport: Arc<dyn PrinterTransport> = Arc::new(FakePrinterTransport {
            fail: true,
            sent: sent.clone(),
        });

        let seq_service = Arc::new(SequenceService::new(seq_repo));
        let use_case = PrintLabels::new(seq_service, printer_repo, job_repo.clone(), transport);

        let result = use_case.execute(request("printer-1", 4)).await;
        assert!(matches!(result, Err(ApplicationError::PrintJobFailed(_))));

        let jobs = job_repo.find_recent(10).await.unwrap();
        let job = jobs.iter().find(|j| j.printer_id == "printer-1").unwrap();
        assert_eq!(job.status, PrintJobStatus::Failed);
        assert!(job.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_print_labels_missing_printer() {
        let printer_repo: Arc<dyn PrinterRepository> =
            Arc::new(InMemoryPrinterRepository::new(vec![]));
        let seq_repo: Arc<dyn SequenceRepository> =
            Arc::new(InMemorySequenceRepository::new("Z0000000"));
        let job_repo: Arc<dyn PrintJobRepository> = Arc::new(InMemoryPrintJobRepository::new());
        let sent: Arc<Mutex<Option<Vec<u8>>>> = Arc::new(Mutex::new(None));
        let transport: Arc<dyn PrinterTransport> = Arc::new(FakePrinterTransport {
            fail: false,
            sent: sent.clone(),
        });

        let seq_service = Arc::new(SequenceService::new(seq_repo));
        let use_case = PrintLabels::new(seq_service, printer_repo, job_repo, transport);

        let result = use_case.execute(request("missing", 4)).await;
        assert!(matches!(
            result,
            Err(ApplicationError::PrinterNotConfigured)
        ));
    }
}
