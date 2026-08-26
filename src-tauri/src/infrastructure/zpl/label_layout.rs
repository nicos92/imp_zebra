#[derive(Debug, Clone)]
pub struct LabelLayout {
    pub label_width_dots: u32,
    pub label_height_dots: u32,
    pub columns: u32,
    pub margin_x: u32,
    pub margin_y: u32,
    pub barcode_height: u32,
    pub title_font_size: u32,
    pub code_font_size: u32,
    pub dpi: u32,
}

impl LabelLayout {
    pub fn new(
        label_width_dots: u32,
        label_height_dots: u32,
        columns: u32,
        dpi: u32,
    ) -> Self {
        Self {
            label_width_dots,
            label_height_dots,
            columns,
            margin_x: 50,
            margin_y: 50,
            barcode_height: 100,
            title_font_size: 30,
            code_font_size: 25,
            dpi,
        }
    }

    pub fn from_printer_config(
        label_width_mm: f64,
        label_height_mm: f64,
        columns: u32,
        dpi: u32,
    ) -> Self {
        let width_dots = (label_width_mm / 25.4 * dpi as f64 + 0.5) as u32;
        let height_dots = (label_height_mm / 25.4 * dpi as f64 + 0.5) as u32;
        Self::new(width_dots, height_dots, columns, dpi)
    }

    pub fn total_width(&self) -> u32 {
        self.label_width_dots * self.columns
    }

    pub fn total_height(&self, rows: u32) -> u32 {
        self.label_height_dots * rows
    }

    pub fn label_x(&self, column: u32) -> u32 {
        self.margin_x + column * self.label_width_dots
    }

    pub fn label_y(&self, row: u32) -> u32 {
        self.margin_y + row * self.label_height_dots
    }

    pub fn title_position(&self, column: u32, row: u32) -> (u32, u32) {
        (self.label_x(column), self.label_y(row))
    }

    pub fn barcode_position(&self, column: u32, row: u32) -> (u32, u32) {
        (
            self.label_x(column),
            self.label_y(row) + self.title_font_size + 20,
        )
    }

    pub fn code_text_position(&self, column: u32, row: u32) -> (u32, u32) {
        (
            self.label_x(column),
            self.label_y(row) + self.title_font_size + 20 + self.barcode_height + 10,
        )
    }
}

impl Default for LabelLayout {
    fn default() -> Self {
        Self::new(400, 400, 2, 203)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_layout() {
        let layout = LabelLayout::default();
        assert_eq!(layout.label_width_dots, 400);
        assert_eq!(layout.label_height_dots, 400);
        assert_eq!(layout.columns, 2);
        assert_eq!(layout.dpi, 203);
    }

    #[test]
    fn test_total_width() {
        let layout = LabelLayout::default();
        assert_eq!(layout.total_width(), 800);
    }

    #[test]
    fn test_label_positions() {
        let layout = LabelLayout::default();
        assert_eq!(layout.label_x(0), 50);
        assert_eq!(layout.label_x(1), 450);
        assert_eq!(layout.label_y(0), 50);
        assert_eq!(layout.label_y(1), 450);
    }

    #[test]
    fn test_from_printer_config() {
        let layout = LabelLayout::from_printer_config(50.0, 50.0, 2, 203);
        assert_eq!(layout.label_width_dots, 400);
        assert_eq!(layout.label_height_dots, 400);
    }
}
