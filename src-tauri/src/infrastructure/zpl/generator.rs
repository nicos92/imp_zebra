use crate::infrastructure::zpl::label_layout::{LabelLayout, LabelPosition};

pub struct ZplGenerator {
    layout: LabelLayout,
}

impl ZplGenerator {
    pub fn new(layout: LabelLayout) -> Self {
        Self { layout }
    }

    pub fn generate_batch(&self, labels: &[(String, LabelPosition)], timestamp: &str) -> String {
        let mut zpl = String::with_capacity(1024);

        zpl.push_str("^XA\n");
        zpl.push_str(&format!("^PW{}\n", self.layout.total_width()));

        let rows = if labels.is_empty() {
            0
        } else {
            (labels.len() as u32 - 1) / self.layout.columns + 1
        };
        zpl.push_str(&format!("^LL{}\n", self.layout.total_height(rows)));

        for (code, pos) in labels {
            self.append_label(&mut zpl, code, timestamp, *pos);
        }

        zpl.push_str("^XZ\n");
        zpl
    }

    fn append_label(&self, zpl: &mut String, code: &str, timestamp: &str, pos: LabelPosition) {
        let (title_x, title_y) = self.layout.title_position(pos.column, pos.row);
        zpl.push_str(&format!(
            "^FO{},{}\n^A@N,{},{}\n^FD{}\n^FS\n",
            title_x, title_y, self.layout.title_font_size, self.layout.title_font_size, timestamp
        ));

        let (barcode_x, barcode_y) = self.layout.barcode_position(pos.column, pos.row);
        zpl.push_str(&format!(
            "^FO{},{}\n^BY2\n^BCN,{},Y,N\n^FD{}\n^FS\n",
            barcode_x, barcode_y, self.layout.barcode_height, code
        ));

        let (text_x, text_y) = self.layout.code_text_position(pos.column, pos.row);
        zpl.push_str(&format!(
            "^FO{},{}\n^A@N,{},{}\n^FD{}\n^FS\n",
            text_x, text_y, self.layout.code_font_size, self.layout.code_font_size, code
        ));
    }
}

impl Default for ZplGenerator {
    fn default() -> Self {
        Self::new(LabelLayout::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_single_label() {
        let generator = ZplGenerator::default();
        let labels = vec![("Z0000001".to_string(), LabelPosition { row: 0, column: 0 })];
        let zpl = generator.generate_batch(&labels, "26/08/2026 07:15:32");

        assert!(zpl.starts_with("^XA"));
        assert!(zpl.ends_with("^XZ\n"));
        assert!(zpl.contains("Z0000001"));
        assert!(zpl.contains("26/08/2026 07:15:32"));
        assert!(zpl.contains("^BCN"));
    }

    #[test]
    fn test_generate_two_labels() {
        let generator = ZplGenerator::default();
        let labels = vec![
            ("Z0000001".to_string(), LabelPosition { row: 0, column: 0 }),
            ("Z0000002".to_string(), LabelPosition { row: 0, column: 1 }),
        ];
        let zpl = generator.generate_batch(&labels, "26/08/2026 07:15:32");

        assert!(zpl.contains("Z0000001"));
        assert!(zpl.contains("Z0000002"));
    }

    #[test]
    fn test_generate_odd_quantity_no_phantom() {
        let generator = ZplGenerator::default();
        let labels = vec![
            ("Z0000001".to_string(), LabelPosition { row: 0, column: 0 }),
            ("Z0000002".to_string(), LabelPosition { row: 0, column: 1 }),
            ("Z0000003".to_string(), LabelPosition { row: 1, column: 0 }),
        ];
        let zpl = generator.generate_batch(&labels, "26/08/2026 07:15:32");

        assert!(zpl.contains("Z0000001"));
        assert!(zpl.contains("Z0000002"));
        assert!(zpl.contains("Z0000003"));
        assert!(!zpl.contains("Z0000004"));
        assert!(zpl.contains("^LL800"));
    }

    #[test]
    fn test_generate_empty_batch() {
        let generator = ZplGenerator::default();
        let labels = vec![];
        let zpl = generator.generate_batch(&labels, "26/08/2026 07:15:32");

        assert!(zpl.starts_with("^XA"));
        assert!(zpl.ends_with("^XZ\n"));
        assert!(zpl.contains("^LL0"));
    }
}
