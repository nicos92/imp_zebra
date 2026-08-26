use crate::errors::domain_error::DomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Barcode(String);

impl Barcode {
    pub fn new(code: &str) -> Result<Self, DomainError> {
        if !Self::is_valid(code) {
            return Err(DomainError::InvalidBarcode {
                code: code.to_string(),
            });
        }
        Ok(Self(code.to_string()))
    }

    pub fn value(&self) -> &str {
        &self.0
    }

    fn is_valid(code: &str) -> bool {
        if code.len() < 8 || code.len() > 20 {
            return false;
        }
        let prefix = &code[..1];
        if prefix != "Z" {
            return false;
        }
        code[1..].chars().all(|c| c.is_ascii_digit())
    }
}

impl std::fmt::Display for Barcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_barcode() {
        let barcode = Barcode::new("Z0000001").unwrap();
        assert_eq!(barcode.value(), "Z0000001");
    }

    #[test]
    fn test_invalid_prefix() {
        assert!(Barcode::new("A0000001").is_err());
    }

    #[test]
    fn test_invalid_chars() {
        assert!(Barcode::new("Z000000A").is_err());
    }

    #[test]
    fn test_too_short() {
        assert!(Barcode::new("Z1").is_err());
        assert!(Barcode::new("Z12").is_err());
        assert!(Barcode::new("Z123").is_err());
    }

    #[test]
    fn test_too_long() {
        assert!(Barcode::new("Z00000000000000000000").is_err());
    }
}
