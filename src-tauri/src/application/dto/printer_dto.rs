use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterConfigDto {
    pub id: Option<String>,
    pub name: String,
    pub model: String,
    pub dpi: u32,
    pub label_width_mm: f64,
    pub label_height_mm: f64,
    pub columns: u32,
    pub connection_type: String,
    pub ip_address: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterDto {
    pub id: String,
    pub name: String,
    pub model: String,
    pub dpi: u32,
    pub label_width_mm: f64,
    pub label_height_mm: f64,
    pub columns: u32,
    pub connection_type: String,
    pub ip_address: String,
    pub port: u16,
    pub created_at: String,
    pub updated_at: String,
}
