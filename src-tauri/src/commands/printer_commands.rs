use tauri::State;

use crate::application::dto::printer_dto::{PrinterConfigDto, PrinterDto};
use crate::application::use_cases::configure_printer::ConfigurePrinter;
use crate::application::use_cases::get_printer_config::GetPrinterConfig;
use crate::application::use_cases::test_printer::TestPrinter;
use crate::errors::application_error::ApplicationError;
use crate::state::app_state::AppState;

#[tauri::command]
pub async fn get_printer_config(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<PrinterDto>, ApplicationError> {
    let repo = std::sync::Arc::new(
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
    let repo = std::sync::Arc::new(
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
    let repo = std::sync::Arc::new(
        crate::infrastructure::database::repositories::sqlite_printer_repository::SqlitePrinterRepository::new(
            (*state.db).clone(),
        ),
    );
    let use_case = TestPrinter::new(repo);
    use_case.execute(&printer_id).await
}
