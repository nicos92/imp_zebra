use serde::Serialize;
use thiserror::Error;

use crate::errors::domain_error::DomainError;
use crate::errors::infrastructure_error::InfrastructureError;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    Infrastructure(#[from] InfrastructureError),

    #[error("Printer not configured")]
    PrinterNotConfigured,

    #[error("Print job failed: {0}")]
    PrintJobFailed(String),
}

impl Serialize for ApplicationError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let (code, message) = match self {
            ApplicationError::Domain(e) => ("DOMAIN_ERROR", e.to_string()),
            ApplicationError::Infrastructure(e) => match e {
                InfrastructureError::Database(_) => ("DATABASE_ERROR", e.to_string()),
                InfrastructureError::PrinterConnection(_) => {
                    ("PRINTER_CONNECTION_FAILED", e.to_string())
                }
                InfrastructureError::PrinterTimeout => ("PRINTER_TIMEOUT", e.to_string()),
                InfrastructureError::PrinterUnavailable => {
                    ("PRINTER_UNAVAILABLE", e.to_string())
                }
                InfrastructureError::ZplGeneration(_) => ("ZPL_GENERATION_FAILED", e.to_string()),
            },
            ApplicationError::PrinterNotConfigured => {
                ("PRINTER_NOT_CONFIGURED", self.to_string())
            }
            ApplicationError::PrintJobFailed(_) => ("PRINT_JOB_FAILED", self.to_string()),
        };

        let mut map = std::collections::HashMap::new();
        map.insert("code", code);
        map.insert("message", &message);
        map.serialize(serializer)
    }
}
