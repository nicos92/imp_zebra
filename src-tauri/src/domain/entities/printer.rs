use chrono::{DateTime, Utc};
use crate::domain::value_objects::printer_config::{ConnectionType, PrinterConfig};

#[derive(Debug, Clone)]
pub struct Printer {
    pub id: String,
    pub name: String,
    pub model: String,
    pub dpi: u32,
    pub label_width_mm: f64,
    pub label_height_mm: f64,
    pub columns: u32,
    pub connection_type: ConnectionType,
    pub ip_address: String,
    pub port: u16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Printer {
    pub fn new(id: &str, config: &PrinterConfig) -> Self {
        let now = Utc::now();
        Self {
            id: id.to_string(),
            name: config.name.clone(),
            model: config.model.clone(),
            dpi: config.dpi,
            label_width_mm: config.label_width_mm,
            label_height_mm: config.label_height_mm,
            columns: config.columns,
            connection_type: config.connection_type.clone(),
            ip_address: config.ip_address.clone(),
            port: config.port,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update(&mut self, config: &PrinterConfig) {
        self.name = config.name.clone();
        self.model = config.model.clone();
        self.dpi = config.dpi;
        self.label_width_mm = config.label_width_mm;
        self.label_height_mm = config.label_height_mm;
        self.columns = config.columns;
        self.connection_type = config.connection_type.clone();
        self.ip_address = config.ip_address.clone();
        self.port = config.port;
        self.updated_at = Utc::now();
    }

    pub fn to_config(&self) -> PrinterConfig {
        PrinterConfig::new(
            &self.name,
            &self.model,
            self.dpi,
            self.label_width_mm,
            self.label_height_mm,
            self.columns,
            self.connection_type.clone(),
            &self.ip_address,
            self.port,
        )
        .expect("Printer entity contains invalid config")
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.ip_address, self.port)
    }
}
