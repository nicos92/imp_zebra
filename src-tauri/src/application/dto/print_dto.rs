use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintRequestDto {
    pub quantity: u64,
    pub printer_id: String,
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
