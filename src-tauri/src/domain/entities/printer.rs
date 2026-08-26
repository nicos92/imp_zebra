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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::printer_config::{ConnectionType, PrinterConfig};

    fn valid_printer_config() -> PrinterConfig {
        PrinterConfig::new(
            "Test Printer",
            "Zebra ZD421",
            203,
            50.0,
            50.0,
            2,
            ConnectionType::Tcp,
            "192.168.1.100",
            9100,
        )
        .unwrap()
    }

    #[test]
    fn test_new_printer() {
        let printer = Printer::new("printer-1", &valid_printer_config());
        assert_eq!(printer.id, "printer-1");
        assert_eq!(printer.name, "Test Printer");
        assert_eq!(printer.model, "Zebra ZD421");
        assert_eq!(printer.dpi, 203);
        assert_eq!(printer.label_width_mm, 50.0);
        assert_eq!(printer.label_height_mm, 50.0);
        assert_eq!(printer.columns, 2);
        assert_eq!(printer.connection_type, ConnectionType::Tcp);
        assert_eq!(printer.ip_address, "192.168.1.100");
        assert_eq!(printer.port, 9100);
    }

    #[test]
    fn test_update_printer() {
        let mut printer = Printer::new("printer-1", &valid_printer_config());
        let new_config = PrinterConfig::new(
            "Updated Printer",
            "Zebra ZD421",
            300,
            100.0,
            150.0,
            1,
            ConnectionType::Tcp,
            "192.168.1.101",
            9100,
        )
        .unwrap();

        printer.update(&new_config);
        assert_eq!(printer.name, "Updated Printer");
        assert_eq!(printer.dpi, 300);
        assert_eq!(printer.label_width_mm, 100.0);
        assert_eq!(printer.label_height_mm, 150.0);
        assert_eq!(printer.columns, 1);
        assert_eq!(printer.ip_address, "192.168.1.101");
    }

    #[test]
    fn test_to_config() {
        let printer = Printer::new("printer-1", &valid_printer_config());
        let config = printer.to_config();
        assert_eq!(config.name, "Test Printer");
        assert_eq!(config.model, "Zebra ZD421");
        assert_eq!(config.dpi, 203);
        assert_eq!(config.label_width_mm, 50.0);
        assert_eq!(config.label_height_mm, 50.0);
        assert_eq!(config.columns, 2);
        assert_eq!(config.connection_type, ConnectionType::Tcp);
        assert_eq!(config.ip_address, "192.168.1.100");
        assert_eq!(config.port, 9100);
    }

    #[test]
    fn test_address() {
        let printer = Printer::new("printer-1", &valid_printer_config());
        assert_eq!(printer.address(), "192.168.1.100:9100");
    }
}
