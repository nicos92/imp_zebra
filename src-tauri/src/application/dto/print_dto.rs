use serde::{Deserialize, Serialize};

use crate::domain::entities::print_job::PrintJob;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintRequestDto {
    pub quantity: u64,
    pub printer_id: String,
}

impl From<PrintJob> for PrintJobDto {
    fn from(job: PrintJob) -> Self {
        Self {
            id: job.id,
            printer_id: job.printer_id,
            start_code: job.start_code,
            end_code: job.end_code,
            quantity: job.quantity,
            status: job.status.to_string(),
            created_at: job.created_at.to_rfc3339(),
            completed_at: job.completed_at.map(|dt| dt.to_rfc3339()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintJobDto {
    pub id: String,
    pub printer_id: String,
    pub start_code: String,
    pub end_code: String,
    pub quantity: u64,
    pub status: String,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintResultDto {
    pub job_id: String,
    pub start_code: String,
    pub end_code: String,
    pub quantity: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceInfoDto {
    pub last_used_code: String,
    pub next_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewLabelDto {
    pub code: String,
    pub timestamp: String,
    pub zpl: String,
}
