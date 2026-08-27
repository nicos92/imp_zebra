use std::sync::Arc;
use tauri::State;

use crate::application::dto::print_dto::SequenceInfoDto;
use crate::application::dto::printer_dto::{PrinterConfigDto, PrinterDto};
use crate::application::use_cases::configure_printer::ConfigurePrinter;
use crate::application::use_cases::get_configured_printer::GetConfiguredPrinter;
use crate::application::use_cases::get_current_sequence::GetCurrentSequence;
use crate::application::use_cases::get_printer_config::GetPrinterConfig;
use crate::application::use_cases::test_printer::TestPrinter;
use crate::domain::repositories::printer_repository::PrinterRepository;
use crate::domain::repositories::sequence_repository::SequenceRepository;
use crate::domain::services::sequence_service::SequenceService;
use crate::errors::application_error::ApplicationError;
use crate::infrastructure::printer::printer_transport::PrinterTransport;
use crate::infrastructure::printer::tcp_transport::TcpPrinterTransport;
use crate::state::app_state::AppState;

#[tauri::command]
pub async fn get_printer_config(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<PrinterDto>, ApplicationError> {
    let repo = Arc::new(
        crate::infrastructure::database::repositories::sqlite_printer_repository::SqlitePrinterRepository::new(
            (*state.db).clone(),
        ),
    );
    let use_case = GetPrinterConfig::new(repo);
    use_case.execute(&id).await
}

#[tauri::command]
pub async fn save_printer_config(
    state: State<'_, AppState>,
    config: PrinterConfigDto,
) -> Result<PrinterDto, ApplicationError> {
    let repo = Arc::new(
        crate::infrastructure::database::repositories::sqlite_printer_repository::SqlitePrinterRepository::new(
            (*state.db).clone(),
        ),
    );
    let use_case = ConfigurePrinter::new(repo);
    use_case.execute(config).await
}

#[tauri::command]
pub async fn test_printer_connection(
    state: State<'_, AppState>,
    printer_id: String,
) -> Result<bool, ApplicationError> {
    let repo: Arc<dyn PrinterRepository> = Arc::new(
        crate::infrastructure::database::repositories::sqlite_printer_repository::SqlitePrinterRepository::new(
            (*state.db).clone(),
        ),
    );

    let printer = repo
        .find_by_id(&printer_id)
        .await?
        .ok_or(ApplicationError::PrinterNotConfigured)?;

    let transport: Arc<dyn PrinterTransport> =
        Arc::new(TcpPrinterTransport::new(&printer.ip_address, printer.port));

    let use_case = TestPrinter::new(repo, transport);
    use_case.execute(&printer_id).await
}

#[tauri::command]
pub async fn get_current_sequence(
    state: State<'_, AppState>,
) -> Result<SequenceInfoDto, ApplicationError> {
    let sequence_repo: Arc<dyn SequenceRepository> = Arc::new(
        crate::infrastructure::database::repositories::sqlite_sequence_repository::SqliteSequenceRepository::new(
            (*state.db).clone(),
        ),
    );
    let sequence_service = Arc::new(SequenceService::new(sequence_repo));
    let use_case = GetCurrentSequence::new(sequence_service);
    use_case.execute().await
}

#[tauri::command]
pub async fn get_configured_printer(
    state: State<'_, AppState>,
) -> Result<Option<PrinterDto>, ApplicationError> {
    let repo: Arc<dyn PrinterRepository> = Arc::new(
        crate::infrastructure::database::repositories::sqlite_printer_repository::SqlitePrinterRepository::new(
            (*state.db).clone(),
        ),
    );
    let use_case = GetConfiguredPrinter::new(repo);
    use_case.execute().await
}
