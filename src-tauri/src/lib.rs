mod application;
mod commands;
mod domain;
mod errors;
mod infrastructure;
mod state;

use tauri::Manager;

pub use state::app_state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::block_on(async {
                let pool = infrastructure::database::connection::create_pool(&app_handle)
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
                app.manage(AppState::new(std::sync::Arc::new(pool)));
                Ok::<(), Box<dyn std::error::Error>>(())
            })
        })
        .invoke_handler(tauri::generate_handler![
            commands::printer_commands::get_printer_config,
            commands::printer_commands::save_printer_config,
            commands::printer_commands::get_current_sequence,
            commands::printer_commands::get_configured_printer,
            commands::print_commands::print_labels,
            commands::print_commands::preview_label,
            commands::print_commands::get_print_job,
            commands::print_commands::list_print_jobs,
            commands::print_commands::list_windows_printers,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
