use tauri::State;

use crate::application::dto::print_dto::{
    PreviewLabelDto, PrintJobDto, PrintRequestDto, PrintResultDto,
};
use crate::application::use_cases::preview_label::PreviewLabel;
use crate::application::use_cases::print_labels::PrintLabels;
use crate::domain::repositories::print_job_repository::PrintJobRepository;
use crate::domain::services::sequence_service::SequenceService;
use crate::errors::application_error::ApplicationError;
use crate::state::app_state::AppState;

#[tauri::command]
pub async fn print_labels(
    state: State<'_, AppState>,
    request: PrintRequestDto,
) -> Result<PrintResultDto, ApplicationError> {
    let db = (*state.db).clone();

    let sequence_repo = std::sync::Arc::new(
        crate::infrastructure::database::repositories::sqlite_sequence_repository::SqliteSequenceRepository::new(db.clone()),
    );
    let printer_repo = std::sync::Arc::new(
        crate::infrastructure::database::repositories::sqlite_printer_repository::SqlitePrinterRepository::new(db.clone()),
    );
    let job_repo = std::sync::Arc::new(
        crate::infrastructure::database::repositories::sqlite_print_job_repository::SqlitePrintJobRepository::new(db),
    );

    let sequence_service = std::sync::Arc::new(SequenceService::new(sequence_repo));

    let use_case = PrintLabels::new(sequence_service, printer_repo, job_repo);
    use_case.execute(request).await
}

#[tauri::command]
pub async fn preview_label(
    state: State<'_, AppState>,
    label_width_mm: f64,
    label_height_mm: f64,
    columns: u32,
    dpi: u32,
) -> Result<PreviewLabelDto, ApplicationError> {
    let sequence_repo = std::sync::Arc::new(
        crate::infrastructure::database::repositories::sqlite_sequence_repository::SqliteSequenceRepository::new(
            (*state.db).clone(),
        ),
    );
    let sequence_service = std::sync::Arc::new(SequenceService::new(sequence_repo));

    let use_case = PreviewLabel::new(sequence_service);
    use_case
        .execute(label_width_mm, label_height_mm, columns, dpi)
        .await
}

#[tauri::command]
pub async fn get_print_job(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<Option<PrintJobDto>, ApplicationError> {
    let repo = crate::infrastructure::database::repositories::sqlite_print_job_repository::SqlitePrintJobRepository::new(
        (*state.db).clone(),
    );
    let job = repo.find_by_id(&job_id).await?;
    Ok(job.map(|j| PrintJobDto {
        id: j.id,
        printer_id: j.printer_id,
        start_code: j.start_code,
        end_code: j.end_code,
        quantity: j.quantity,
        status: j.status.to_string(),
        created_at: j.created_at.to_rfc3339(),
        completed_at: j.completed_at.map(|dt| dt.to_rfc3339()),
    }))
}
