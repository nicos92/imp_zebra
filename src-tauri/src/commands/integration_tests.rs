use std::sync::Arc;

use tauri::test::mock_app;
use tauri::Manager;

use crate::application::dto::print_dto::PrintRequestDto;
use crate::application::dto::printer_dto::PrinterConfigDto;
use crate::commands::print_commands::{
    get_print_job, list_print_jobs, preview_label, print_labels,
};
use crate::commands::printer_commands::{
    get_configured_printer, get_current_sequence, get_printer_config, save_printer_config,
    test_printer_connection,
};
use crate::infrastructure::database::test_helpers::create_test_pool;
use crate::state::app_state::AppState;

fn printer_config(ip: &str, port: u16) -> PrinterConfigDto {
    PrinterConfigDto {
        id: None,
        name: "Test Printer".to_string(),
        model: "Zebra ZD421".to_string(),
        dpi: 203,
        label_width_mm: 50.0,
        label_height_mm: 50.0,
        columns: 2,
        connection_type: "tcp".to_string(),
        ip_address: ip.to_string(),
        port,
    }
}

fn print_request(printer_id: &str, quantity: u64) -> PrintRequestDto {
    PrintRequestDto {
        quantity,
        printer_id: printer_id.to_string(),
    }
}

/// Builds a mock Tauri app with `AppState` managed against a real in-memory SQLite pool.
async fn test_app() -> tauri::App<tauri::test::MockRuntime> {
    let app = mock_app();
    app.manage(AppState::new(Arc::new(create_test_pool().await)));
    app
}

async fn save_printer(app: &tauri::App<tauri::test::MockRuntime>, cfg: PrinterConfigDto) -> String {
    save_printer_config(app.state::<AppState>(), cfg)
        .await
        .expect("save_printer_config should succeed")
        .id
}

#[tokio::test]
async fn save_and_read_printer_config() {
    let app = test_app().await;
    let saved_id = save_printer(&app, printer_config("192.168.1.100", 9100)).await;

    let fetched = get_printer_config(app.state::<AppState>(), saved_id.clone())
        .await
        .unwrap()
        .expect("printer should exist");
    assert_eq!(fetched.id, saved_id);
    assert_eq!(fetched.ip_address, "192.168.1.100");
    assert_eq!(fetched.port, 9100);
    assert_eq!(fetched.dpi, 203);
    assert_eq!(fetched.connection_type, "tcp");

    let configured = get_configured_printer(app.state::<AppState>())
        .await
        .unwrap()
        .expect("configured printer should be returned");
    assert_eq!(configured.id, saved_id);
}

#[tokio::test]
async fn get_configured_printer_is_none_when_empty() {
    let app = test_app().await;
    let configured = get_configured_printer(app.state::<AppState>())
        .await
        .unwrap();
    assert!(configured.is_none());
}

#[tokio::test]
async fn test_printer_connection_against_local_listener() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let accepted = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        use tokio::io::AsyncReadExt;
        let mut buf = Vec::new();
        let _ = socket.read_to_end(&mut buf).await;
        buf
    });

    let app = test_app().await;
    let id = save_printer(&app, printer_config(&addr.ip().to_string(), addr.port())).await;

    let ok = test_printer_connection(app.state::<AppState>(), id)
        .await
        .unwrap();
    assert!(ok);

    accepted.await.unwrap();
}

#[tokio::test]
async fn test_printer_connection_refused_is_error() {
    let dead_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = dead_listener.local_addr().unwrap();
    drop(dead_listener);

    let app = test_app().await;
    let id = save_printer(&app, printer_config(&addr.ip().to_string(), addr.port())).await;

    let err = test_printer_connection(app.state::<AppState>(), id)
        .await
        .unwrap_err();
    let json = serde_json::to_value(&err).unwrap();
    assert_eq!(json["code"], "PRINTER_CONNECTION_FAILED");
}

#[tokio::test]
async fn get_current_sequence_starts_at_first_code() {
    let app = test_app().await;
    let info = get_current_sequence(app.state::<AppState>()).await.unwrap();
    assert_eq!(info.last_used_code, "Z0000000");
    assert_eq!(info.next_code, "Z0000001");
}

#[tokio::test]
async fn print_labels_sends_zpl_to_local_tcp_listener() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let accepted = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        use tokio::io::AsyncReadExt;
        let mut buf = Vec::new();
        let _ = socket.read_to_end(&mut buf).await;
        buf
    });

    let app = test_app().await;
    let id = save_printer(&app, printer_config(&addr.ip().to_string(), addr.port())).await;

    let result = print_labels(app.state::<AppState>(), print_request(&id, 4))
        .await
        .expect("print_labels should succeed");
    assert_eq!(result.start_code, "Z0000001");
    assert_eq!(result.end_code, "Z0000004");
    assert_eq!(result.quantity, 4);
    assert_eq!(result.status, "completed");

    let zpl_bytes = accepted.await.unwrap();
    let zpl = String::from_utf8(zpl_bytes).unwrap();
    assert!(zpl.starts_with("^XA"), "ZPL should start with ^XA: {zpl}");
    assert!(
        zpl.trim_end().ends_with("^XZ"),
        "ZPL should end with ^XZ: {zpl}"
    );
    assert!(zpl.contains("Z0000001"));
    assert!(zpl.contains("Z0000004"));
    assert!(!zpl.contains("Z0000005"), "ZPL must not over-reserve codes");
    assert!(zpl.contains("^BC"), "ZPL should contain a Code 128 barcode");

    let job = get_print_job(app.state::<AppState>(), result.job_id.clone())
        .await
        .unwrap()
        .expect("job should exist");
    assert_eq!(job.status, "completed");
    assert!(job.completed_at.is_some());

    let seq = get_current_sequence(app.state::<AppState>()).await.unwrap();
    assert_eq!(seq.next_code, "Z0000005");
}

#[tokio::test]
async fn print_labels_marks_job_failed_when_tcp_refused() {
    let dead_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = dead_listener.local_addr().unwrap();
    drop(dead_listener);

    let app = test_app().await;
    let id = save_printer(&app, printer_config(&addr.ip().to_string(), addr.port())).await;

    let err = print_labels(app.state::<AppState>(), print_request(&id, 4))
        .await
        .expect_err("print_labels should fail");
    let json = serde_json::to_value(&err).unwrap();
    assert_eq!(json["code"], "PRINT_JOB_FAILED");

    let jobs = list_print_jobs(app.state::<AppState>(), Some(50))
        .await
        .unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].status, "failed");
    assert!(jobs[0].completed_at.is_some());
}

#[tokio::test]
async fn print_labels_with_missing_printer_is_error() {
    let app = test_app().await;
    let err = print_labels(app.state::<AppState>(), print_request("missing", 4))
        .await
        .unwrap_err();
    let json = serde_json::to_value(&err).unwrap();
    assert_eq!(json["code"], "PRINTER_NOT_CONFIGURED");
}

#[tokio::test]
async fn preview_label_generates_zpl_for_next_code() {
    let app = test_app().await;
    let preview = preview_label(app.state::<AppState>(), 50.0, 50.0, 2, 203)
        .await
        .unwrap();
    assert_eq!(preview.code, "Z0000001");
    assert!(preview.zpl.starts_with("^XA"));
    assert!(preview.zpl.trim_end().ends_with("^XZ"));
    assert!(preview.zpl.contains("^BC"));
}

#[tokio::test]
async fn list_print_jobs_is_empty_initially_and_get_unknown_returns_none() {
    let app = test_app().await;
    let jobs = list_print_jobs(app.state::<AppState>(), None)
        .await
        .unwrap();
    assert!(jobs.is_empty());

    let job = get_print_job(app.state::<AppState>(), "no-such-job".to_string())
        .await
        .unwrap();
    assert!(job.is_none());
}
