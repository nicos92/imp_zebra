use std::sync::Arc;

use crate::application::dto::print_dto::{PrintRequestDto, PrintResultDto};
use crate::domain::entities::print_job::PrintJob;
use crate::domain::repositories::print_job_repository::PrintJobRepository;
use crate::domain::repositories::printer_repository::PrinterRepository;
use crate::domain::services::label_service::LabelService;
use crate::domain::services::sequence_service::SequenceService;
use crate::errors::application_error::ApplicationError;
use crate::infrastructure::printer::zebra_printer::ZebraPrinter;
use crate::infrastructure::zpl::generator::ZplGenerator;
use crate::infrastructure::zpl::label_layout::LabelLayout;

pub struct PrintLabels {
    sequence_service: Arc<SequenceService>,
    printer_repository: Arc<dyn PrinterRepository>,
    print_job_repository: Arc<dyn PrintJobRepository>,
}

impl PrintLabels {
    pub fn new(
        sequence_service: Arc<SequenceService>,
        printer_repository: Arc<dyn PrinterRepository>,
        print_job_repository: Arc<dyn PrintJobRepository>,
    ) -> Self {
        Self {
            sequence_service,
            printer_repository,
            print_job_repository,
        }
    }

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

        let zebra = ZebraPrinter::new(printer);
        job.start_printing();
        self.print_job_repository
            .update_status(&job_id, job.status.clone(), None)
            .await?;

        match zebra.send_zpl(&zpl).await {
            Ok(()) => {
                job.complete();
                self.print_job_repository
                    .update_status(
                        &job_id,
                        job.status.clone(),
                        Some(&chrono::Utc::now().to_rfc3339()),
                    )
                    .await?;
            }
            Err(e) => {
                job.fail();
                self.print_job_repository
                    .update_status(
                        &job_id,
                        job.status.clone(),
                        Some(&chrono::Utc::now().to_rfc3339()),
                    )
                    .await?;
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
