use crate::errors::domain_error::DomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionType {
    Tcp,
    Usb,
    Serial,
}

impl ConnectionType {
    pub fn as_str(&self) -> &str {
        match self {
            ConnectionType::Tcp => "tcp",
            ConnectionType::Usb => "usb",
            ConnectionType::Serial => "serial",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, DomainError> {
        match s.to_lowercase().as_str() {
            "tcp" => Ok(ConnectionType::Tcp),
            "usb" => Ok(ConnectionType::Usb),
            "serial" => Ok(ConnectionType::Serial),
            _ => Err(DomainError::InvalidPrinterConfig {
                field: "connection_type".to_string(),
                message: format!("Unknown connection type: {}", s),
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PrinterConfig {
    pub name: String,
    pub model: String,
    pub dpi: u32,
    pub label_width_mm: f64,
    pub label_height_mm: f64,
    pub columns: u32,
    pub connection_type: ConnectionType,
    pub ip_address: String,
    pub port: u16,
}

impl PrinterConfig {
    pub fn new(
        name: &str,
        model: &str,
        dpi: u32,
        label_width_mm: f64,
        label_height_mm: f64,
        columns: u32,
        connection_type: ConnectionType,
        ip_address: &str,
        port: u16,
    ) -> Result<Self, DomainError> {
        Self::validate(
            dpi,
            label_width_mm,
            label_height_mm,
            columns,
            ip_address,
            port,
        )?;

        Ok(Self {
            name: name.to_string(),
            model: model.to_string(),
            dpi,
            label_width_mm,
            label_height_mm,
            columns,
            connection_type,
            ip_address: ip_address.to_string(),
            port,
        })
    }

    fn validate(
        dpi: u32,
        label_width_mm: f64,
        label_height_mm: f64,
        columns: u32,
        ip_address: &str,
        port: u16,
    ) -> Result<(), DomainError> {
        if dpi == 0 || dpi > 600 {
            return Err(DomainError::InvalidPrinterConfig {
                field: "dpi".to_string(),
                message: format!("DPI must be between 1 and 600, got {}", dpi),
            });
        }

        if label_width_mm <= 0.0 || label_width_mm > 200.0 {
            return Err(DomainError::InvalidPrinterConfig {
                field: "label_width_mm".to_string(),
                message: format!(
                    "Label width must be between 0 and 200mm, got {}",
                    label_width_mm
                ),
            });
        }

        if label_height_mm <= 0.0 || label_height_mm > 200.0 {
            return Err(DomainError::InvalidPrinterConfig {
                field: "label_height_mm".to_string(),
                message: format!(
                    "Label height must be between 0 and 200mm, got {}",
                    label_height_mm
                ),
            });
        }

        if columns == 0 || columns > 4 {
            return Err(DomainError::InvalidPrinterConfig {
                field: "columns".to_string(),
                message: format!("Columns must be between 1 and 4, got {}", columns),
            });
        }

        if ip_address.is_empty() {
            return Err(DomainError::InvalidPrinterConfig {
                field: "ip_address".to_string(),
                message: "IP address cannot be empty".to_string(),
            });
        }

        if !is_valid_ip(ip_address) {
            return Err(DomainError::InvalidPrinterConfig {
                field: "ip_address".to_string(),
                message: format!("Invalid IP address format: {}", ip_address),
            });
        }

        if port == 0 {
            return Err(DomainError::InvalidPrinterConfig {
                field: "port".to_string(),
                message: "Port cannot be 0".to_string(),
            });
        }

        Ok(())
    }

    pub fn label_width_dots(&self) -> u32 {
        let inches = self.label_width_mm / 25.4;
        (inches * self.dpi as f64 + 0.5) as u32
    }

    pub fn label_height_dots(&self) -> u32 {
        let inches = self.label_height_mm / 25.4;
        (inches * self.dpi as f64 + 0.5) as u32
    }
}

fn is_valid_ip(ip: &str) -> bool {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    parts.iter().all(|p| p.parse::<u8>().is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_config() -> PrinterConfig {
        PrinterConfig::new(
            "Test Printer",
            "ZT410",
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
    fn test_valid_config() {
        let config = valid_config();
        assert_eq!(config.name, "Test Printer");
        assert_eq!(config.dpi, 203);
    }

    #[test]
    fn test_invalid_dpi() {
        assert!(PrinterConfig::new(
            "Test",
            "ZT410",
            0,
            50.0,
            50.0,
            2,
            ConnectionType::Tcp,
            "192.168.1.100",
            9100
        )
        .is_err());
    }

    #[test]
    fn test_invalid_width() {
        assert!(PrinterConfig::new(
            "Test",
            "ZT410",
            203,
            0.0,
            50.0,
            2,
            ConnectionType::Tcp,
            "192.168.1.100",
            9100
        )
        .is_err());
    }

    #[test]
    fn test_invalid_columns() {
        assert!(PrinterConfig::new(
            "Test",
            "ZT410",
            203,
            50.0,
            50.0,
            0,
            ConnectionType::Tcp,
            "192.168.1.100",
            9100
        )
        .is_err());
    }

    #[test]
    fn test_invalid_port() {
        assert!(PrinterConfig::new(
            "Test",
            "ZT410",
            203,
            50.0,
            50.0,
            2,
            ConnectionType::Tcp,
            "192.168.1.100",
            0
        )
        .is_err());
    }

    #[test]
    fn test_label_dimensions_in_dots() {
        let config = valid_config();
        assert_eq!(config.label_width_dots(), 400);
        assert_eq!(config.label_height_dots(), 400);
    }

    #[test]
    fn test_connection_type_str() {
        assert_eq!(ConnectionType::Tcp.as_str(), "tcp");
        assert_eq!(ConnectionType::Usb.as_str(), "usb");
        assert_eq!(ConnectionType::Serial.as_str(), "serial");
    }

    #[test]
    fn test_connection_type_from_str() {
        assert_eq!(
            ConnectionType::from_str("tcp").unwrap(),
            ConnectionType::Tcp
        );
        assert_eq!(
            ConnectionType::from_str("USB").unwrap(),
            ConnectionType::Usb
        );
        assert!(ConnectionType::from_str("invalid").is_err());
    }

    #[test]
    fn test_invalid_ip_address() {
        let result = PrinterConfig::new(
            "Test",
            "Zebra",
            203,
            50.0,
            50.0,
            2,
            ConnectionType::Tcp,
            "invalid-ip",
            9100,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_ip_address() {
        let result = PrinterConfig::new(
            "Test",
            "Zebra",
            203,
            50.0,
            50.0,
            2,
            ConnectionType::Tcp,
            "192.168.1.100",
            9100,
        );
        assert!(result.is_ok());
    }
}
