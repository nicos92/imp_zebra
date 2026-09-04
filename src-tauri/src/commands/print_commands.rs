use std::sync::Arc;
use tauri::{Manager, State};

use crate::application::dto::print_dto::{
    PreviewLabelDto, PrintJobDto, PrintRequestDto, PrintResultDto,
};
use crate::application::dto::printer_dto::WindowsPrinterInfoDto;
use crate::application::use_cases::list_print_jobs::ListPrintJobs;
use crate::application::use_cases::preview_label::PreviewLabel;
use crate::application::use_cases::print_labels::PrintLabels;
use crate::domain::repositories::print_job_repository::PrintJobRepository;
use crate::domain::repositories::printer_repository::PrinterRepository;
use crate::domain::repositories::sequence_repository::SequenceRepository;
use crate::domain::services::sequence_service::SequenceService;
use crate::errors::application_error::ApplicationError;
use crate::infrastructure::printer::printer_transport::PrinterTransport;
use crate::infrastructure::printer::windows_printer::{self, WindowsPrintTransport};
use crate::state::app_state::AppState;

#[tauri::command]
pub async fn print_labels(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    request: PrintRequestDto,
) -> Result<PrintResultDto, ApplicationError> {
    let db = (*state.db).clone();

    let sequence_repo: Arc<dyn SequenceRepository> = Arc::new(
        crate::infrastructure::database::repositories::sqlite_sequence_repository::SqliteSequenceRepository::new(db.clone()),
    );
    let printer_repo: Arc<dyn PrinterRepository> = Arc::new(
        crate::infrastructure::database::repositories::sqlite_printer_repository::SqlitePrinterRepository::new(db.clone()),
    );
    let job_repo: Arc<dyn PrintJobRepository> = Arc::new(
        crate::infrastructure::database::repositories::sqlite_print_job_repository::SqlitePrintJobRepository::new(db),
    );

    let printer_repo_ref: Arc<dyn PrinterRepository> = printer_repo.clone();
    let _printer = printer_repo_ref
        .find_by_id(&request.printer_id)
        .await?
        .ok_or(ApplicationError::PrinterNotConfigured)?;

    let hwnd = app
        .get_webview_window("main")
        .and_then(|w| w.hwnd().ok())
        .map(|h| h.0 as isize);

    let printer_name = tokio::task::spawn_blocking(move || {
        windows_printer::show_print_dialog(hwnd)
    })
    .await
    .map_err(|e| ApplicationError::PrintJobFailed(format!("Dialog thread error: {}", e)))?
    .ok_or(ApplicationError::PrintCancelled)?;

    let transport: Arc<dyn PrinterTransport> =
        Arc::new(WindowsPrintTransport::new(&printer_name));

    let sequence_service = Arc::new(SequenceService::new(sequence_repo));

    let use_case = PrintLabels::new(sequence_service, printer_repo, job_repo, transport);
    use_case.execute(request).await
}

#[tauri::command]
pub async fn list_windows_printers() -> Result<Vec<WindowsPrinterInfoDto>, ApplicationError> {
    let printers = tokio::task::spawn_blocking(|| windows_printer::list_installed_printers())
        .await
        .map_err(|e| ApplicationError::PrintJobFailed(format!("Thread error: {}", e)))?
        .map_err(|e| ApplicationError::PrintJobFailed(e.to_string()))?;

    Ok(printers
        .into_iter()
        .map(|p| WindowsPrinterInfoDto {
            name: p.name,
            driver_name: p.driver_name,
        })
        .collect())
}

#[tauri::command]
pub async fn preview_label(
    state: State<'_, AppState>,
    label_width_mm: f64,
    label_height_mm: f64,
    columns: u32,
    dpi: u32,
) -> Result<PreviewLabelDto, ApplicationError> {
    let sequence_repo: Arc<dyn SequenceRepository> = Arc::new(
        crate::infrastructure::database::repositories::sqlite_sequence_repository::SqliteSequenceRepository::new(
            (*state.db).clone(),
        ),
    );
    let sequence_service = Arc::new(SequenceService::new(sequence_repo));

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
    Ok(job.map(PrintJobDto::from))
}

#[tauri::command]
pub async fn list_print_jobs(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> Result<Vec<PrintJobDto>, ApplicationError> {
    let repo: Arc<dyn PrintJobRepository> = Arc::new(
        crate::infrastructure::database::repositories::sqlite_print_job_repository::SqlitePrintJobRepository::new(
            (*state.db).clone(),
        ),
    );
    let use_case = ListPrintJobs::new(repo);
    use_case.execute(limit).await
}
