use std::sync::Arc;
use crate::domain::entities::sequence::Sequence;
use crate::domain::repositories::sequence_repository::SequenceRepository;
use crate::errors::domain_error::DomainError;

pub struct SequenceService {
    repository: Arc<dyn SequenceRepository>,
}

impl SequenceService {
    pub fn new(repository: Arc<dyn SequenceRepository>) -> Self {
        Self { repository }
    }

    pub async fn get_current_sequence(&self) -> Result<Sequence, DomainError> {
        let code = self.repository.get_last_used_code().await?;
        Sequence::from_code(&code)
    }

    pub async fn reserve_range(
        &self,
        quantity: u64,
    ) -> Result<(String, String, Vec<String>), DomainError> {
        let mut sequence = self.get_current_sequence().await?;
        let (start, end, codes) = sequence.reserve_range(quantity)?;
        self.repository
            .update_last_used_code(&sequence.last_used_code())
            .await?;
        Ok((start, end, codes))
    }

    pub async fn get_next_code(&self) -> Result<String, DomainError> {
        let mut sequence = self.get_current_sequence().await?;
        let code = sequence.next();
        self.repository
            .update_last_used_code(&sequence.last_used_code())
            .await?;
        Ok(code)
    }
}
