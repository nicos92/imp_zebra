use crate::domain::entities::sequence::Sequence;
use crate::errors::domain_error::DomainError;
use crate::infrastructure::zpl::label_layout::LabelPosition;

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
    ) -> Vec<(String, LabelPosition)> {
        codes
            .iter()
            .enumerate()
            .map(|(i, code)| {
                let row = (i as u32) / columns;
                let col = (i as u32) % columns;
                (code.clone(), LabelPosition { row, column: col })
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
        use crate::infrastructure::zpl::label_layout::LabelPosition;
        let service = LabelService::new();
        let codes = vec![
            "Z0000001".to_string(),
            "Z0000002".to_string(),
            "Z0000003".to_string(),
            "Z0000004".to_string(),
        ];
        let positions = service.calculate_positions(&codes, 2);
        assert_eq!(
            positions,
            vec![
                ("Z0000001".to_string(), LabelPosition { row: 0, column: 0 }),
                ("Z0000002".to_string(), LabelPosition { row: 0, column: 1 }),
                ("Z0000003".to_string(), LabelPosition { row: 1, column: 0 }),
                ("Z0000004".to_string(), LabelPosition { row: 1, column: 1 }),
            ]
        );
    }
}
