use std::sync::Arc;

use crate::application::dto::print_dto::PreviewLabelDto;
use crate::domain::services::sequence_service::SequenceService;
use crate::errors::application_error::ApplicationError;
use crate::infrastructure::zpl::generator::ZplGenerator;
use crate::infrastructure::zpl::label_layout::LabelLayout;

pub struct PreviewLabel {
    sequence_service: Arc<SequenceService>,
}

impl PreviewLabel {
    pub fn new(sequence_service: Arc<SequenceService>) -> Self {
        Self { sequence_service }
    }

    pub async fn execute(
        &self,
        label_width_mm: f64,
        label_height_mm: f64,
        columns: u32,
        dpi: u32,
    ) -> Result<PreviewLabelDto, ApplicationError> {
        let sequence = self.sequence_service.get_current_sequence().await?;
        let next_code = {
            let mut seq = sequence.clone();
            seq.next()
        };

        let layout = LabelLayout::from_printer_config(label_width_mm, label_height_mm, columns, dpi);
        let generator = ZplGenerator::new(layout);

        let timestamp = chrono::Local::now().format("%d/%m/%Y %H:%M:%S").to_string();
        let labels = vec![(next_code.clone(), 0, 0)];
        let zpl = generator.generate_batch(&labels, &timestamp);

        Ok(PreviewLabelDto {
            code: next_code,
            timestamp,
            zpl,
        })
    }
}
