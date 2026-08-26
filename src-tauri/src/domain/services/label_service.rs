use crate::domain::entities::sequence::Sequence;
use crate::errors::domain_error::DomainError;

pub struct LabelService;

impl LabelService {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_codes(&self, start: &str, quantity: u64) -> Result<Vec<String>, DomainError> {
        let mut sequence = Sequence::from_code(start)?;
        sequence.next_n(quantity)
    }

    pub fn calculate_positions(
        &self,
        codes: &[String],
        columns: u32,
    ) -> Vec<(String, u32, u32)> {
        codes
            .iter()
            .enumerate()
            .map(|(i, code)| {
                let row = (i as u32) / columns;
                let col = (i as u32) % columns;
                (code.clone(), row, col)
            })
            .collect()
    }
}

impl Default for LabelService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_codes() {
        let service = LabelService::new();
        let codes = service.generate_codes("Z0000000", 3).unwrap();
        assert_eq!(codes, vec!["Z0000001", "Z0000002", "Z0000003"]);
    }

    #[test]
    fn test_calculate_positions() {
        let service = LabelService::new();
        let codes = vec![
            "Z0000001".to_string(),
            "Z0000002".to_string(),
            "Z0000003".to_string(),
            "Z0000004".to_string(),
        ];
        let positions = service.calculate_positions(&codes, 2);
        assert_eq!(positions, vec![
            ("Z0000001".to_string(), 0, 0),
            ("Z0000002".to_string(), 0, 1),
            ("Z0000003".to_string(), 1, 0),
            ("Z0000004".to_string(), 1, 1),
        ]);
    }
}
