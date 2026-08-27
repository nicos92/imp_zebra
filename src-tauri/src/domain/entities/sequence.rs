use crate::errors::domain_error::DomainError;

const MAX_CODE: u64 = 9_999_999;
const CODE_LENGTH: usize = 7;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sequence {
    last_used: u64,
}

impl Sequence {
    pub fn new(last_used: u64) -> Result<Self, DomainError> {
        if last_used > MAX_CODE {
            return Err(DomainError::SequenceOverflow { value: last_used });
        }
        Ok(Self { last_used })
    }

    pub fn from_code(code: &str) -> Result<Self, DomainError> {
        let number = Self::parse_code(code)?;
        Ok(Self { last_used: number })
    }

    pub fn next(&mut self) -> String {
        self.last_used = self.increment_value(self.last_used);
        Sequence::to_code(self.last_used)
    }

    pub fn next_n(&mut self, n: u64) -> Result<Vec<String>, DomainError> {
        if n == 0 {
            return Err(DomainError::InvalidQuantity { value: 0 });
        }

        let mut codes = Vec::with_capacity(n as usize);
        for _ in 0..n {
            codes.push(self.next());
        }
        Ok(codes)
    }

    pub fn reserve_range(
        &mut self,
        quantity: u64,
    ) -> Result<(String, String, Vec<String>), DomainError> {
        if quantity == 0 {
            return Err(DomainError::InvalidQuantity { value: 0 });
        }

        let start = self.increment_value(self.last_used);
        let codes = self.generate_codes(start, quantity)?;
        let end = codes.last().unwrap().clone();
        self.last_used = Sequence::parse_code(&end)?;

        Ok((Sequence::to_code(start), end, codes))
    }

    pub fn last_used_code(&self) -> String {
        Sequence::to_code(self.last_used)
    }

    pub fn last_used_value(&self) -> u64 {
        self.last_used
    }

    fn increment_value(&self, current: u64) -> u64 {
        if current >= MAX_CODE {
            1
        } else {
            current + 1
        }
    }

    fn generate_codes(&self, start: u64, quantity: u64) -> Result<Vec<String>, DomainError> {
        let mut codes = Vec::with_capacity(quantity as usize);
        let mut current = start;

        for _ in 0..quantity {
            codes.push(Sequence::to_code(current));
            current = self.increment_value(current);
        }

        Ok(codes)
    }

    pub fn to_code(value: u64) -> String {
        format!("Z{:0>7}", value)
    }

    pub fn parse_code(code: &str) -> Result<u64, DomainError> {
        if code.len() != CODE_LENGTH + 1 {
            return Err(DomainError::InvalidBarcode {
                code: code.to_string(),
            });
        }

        if !code.starts_with('Z') {
            return Err(DomainError::InvalidBarcode {
                code: code.to_string(),
            });
        }

        let number_str = &code[1..];
        number_str
            .parse::<u64>()
            .map_err(|_| DomainError::InvalidBarcode {
                code: code.to_string(),
            })
    }
}

impl std::fmt::Display for Sequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.last_used_code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_sequence() {
        let seq = Sequence::new(0).unwrap();
        assert_eq!(seq.last_used_code(), "Z0000000");
    }

    #[test]
    fn test_new_sequence_overflow() {
        assert!(Sequence::new(MAX_CODE + 1).is_err());
    }

    #[test]
    fn test_from_code() {
        let seq = Sequence::from_code("Z0000001").unwrap();
        assert_eq!(seq.last_used_value(), 1);
    }

    #[test]
    fn test_next() {
        let mut seq = Sequence::new(0).unwrap();
        assert_eq!(seq.next(), "Z0000001");
        assert_eq!(seq.next(), "Z0000002");
    }

    #[test]
    fn test_next_rollover() {
        let mut seq = Sequence::new(MAX_CODE).unwrap();
        assert_eq!(seq.next(), "Z0000001");
    }

    #[test]
    fn test_next_n() {
        let mut seq = Sequence::new(0).unwrap();
        let codes = seq.next_n(3).unwrap();
        assert_eq!(codes, vec!["Z0000001", "Z0000002", "Z0000003"]);
    }

    #[test]
    fn test_next_n_invalid() {
        let mut seq = Sequence::new(0).unwrap();
        assert!(seq.next_n(0).is_err());
    }

    #[test]
    fn test_reserve_range() {
        let mut seq = Sequence::new(0).unwrap();
        let (start, end, codes) = seq.reserve_range(5).unwrap();
        assert_eq!(start, "Z0000001");
        assert_eq!(end, "Z0000005");
        assert_eq!(codes.len(), 5);
        assert_eq!(seq.last_used_code(), "Z0000005");
    }

    #[test]
    fn test_reserve_range_rollover() {
        let mut seq = Sequence::new(MAX_CODE - 2).unwrap();
        let (start, end, codes) = seq.reserve_range(5).unwrap();
        assert_eq!(start, "Z9999998");
        assert_eq!(end, "Z0000003");
        assert_eq!(codes.len(), 5);
        assert_eq!(
            codes,
            vec!["Z9999998", "Z9999999", "Z0000001", "Z0000002", "Z0000003"]
        );
    }

    #[test]
    fn test_to_code() {
        assert_eq!(Sequence::to_code(0), "Z0000000");
        assert_eq!(Sequence::to_code(1), "Z0000001");
        assert_eq!(Sequence::to_code(9999999), "Z9999999");
    }

    #[test]
    fn test_parse_code() {
        assert_eq!(Sequence::parse_code("Z0000001").unwrap(), 1);
        assert_eq!(Sequence::parse_code("Z9999999").unwrap(), 9999999);
        assert!(Sequence::parse_code("A0000001").is_err());
        assert!(Sequence::parse_code("Z000000").is_err());
    }

    #[test]
    fn test_display() {
        let seq = Sequence::new(42).unwrap();
        assert_eq!(format!("{}", seq), "Z0000042");
    }
}
