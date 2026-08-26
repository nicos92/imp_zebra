use thiserror::Error;

#[derive(Error, Debug)]
pub enum InfrastructureError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Printer connection failed: {0}")]
    PrinterConnection(String),

    #[error("Printer timeout")]
    PrinterTimeout,

    #[error("Printer unavailable")]
    PrinterUnavailable,

    #[error("ZPL generation failed: {0}")]
    ZplGeneration(String),
}
