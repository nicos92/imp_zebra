use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Invalid barcode: {code}")]
    InvalidBarcode { code: String },

    #[error("Sequence overflow: value {value} exceeds maximum")]
    SequenceOverflow { value: u64 },

    #[error("Invalid quantity: {value}")]
    InvalidQuantity { value: u64 },

    #[error("Quantity too large: {value} exceeds limit of {max}")]
    QuantityTooLarge { value: u64, max: u64 },

    #[error("Invalid printer config: {field} - {message}")]
    InvalidPrinterConfig { field: String, message: String },

    #[error("Printer not configured")]
    PrinterNotConfigured,

    #[error("Database error: {0}")]
    Database(String),

    #[error("Invalid state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },

    #[error("Invalid print job status: {0}")]
    InvalidPrintJobStatus(String),
}
