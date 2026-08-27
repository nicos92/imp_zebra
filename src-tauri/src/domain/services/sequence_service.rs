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
        let (start, end, _new_last) = self.repository.reserve_range(quantity).await?;
        let codes = self.codes_for_range(&start, quantity)?;
        Ok((start, end, codes))
    }

    fn codes_for_range(&self, start: &str, quantity: u64) -> Result<Vec<String>, DomainError> {
        let mut sequence = Sequence::from_code(start)?;
        if quantity == 0 {
            return Err(DomainError::InvalidQuantity { value: 0 });
        }
        let mut codes = Vec::with_capacity(quantity as usize);
        for _ in 0..quantity {
            codes.push(sequence.next());
        }
        Ok(codes)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct FakeSequenceRepository {
        last_used: Arc<Mutex<String>>,
    }

    impl FakeSequenceRepository {
        fn new(initial_code: &str) -> Self {
            Self {
                last_used: Arc::new(Mutex::new(initial_code.to_string())),
            }
        }
    }

    #[async_trait::async_trait]
    impl SequenceRepository for FakeSequenceRepository {
        async fn get_last_used_code(&self) -> Result<String, DomainError> {
            Ok(self.last_used.lock().unwrap().clone())
        }

        async fn update_last_used_code(&self, code: &str) -> Result<(), DomainError> {
            *self.last_used.lock().unwrap() = code.to_string();
            Ok(())
        }

        async fn reserve_range(
            &self,
            quantity: u64,
        ) -> Result<(String, String, String), DomainError> {
            let current = self.get_last_used_code().await?;
            let mut seq = crate::domain::entities::sequence::Sequence::from_code(&current)?;
            let (start, end, _) = seq.reserve_range(quantity)?;
            self.update_last_used_code(&seq.last_used_code()).await?;
            Ok((start, end, seq.last_used_code()))
        }
    }

    #[tokio::test]
    async fn test_get_current_sequence() {
        let repo = Arc::new(FakeSequenceRepository::new("Z0000005"));
        let service = SequenceService::new(repo);

        let seq = service.get_current_sequence().await.unwrap();
        assert_eq!(seq.last_used_code(), "Z0000005");
    }

    #[tokio::test]
    async fn test_reserve_range() {
        let repo = Arc::new(FakeSequenceRepository::new("Z0000000"));
        let service = SequenceService::new(repo.clone());

        let (start, end, codes) = service.reserve_range(5).await.unwrap();
        assert_eq!(start, "Z0000001");
        assert_eq!(end, "Z0000005");
        assert_eq!(codes.len(), 5);

        let last_used = repo.get_last_used_code().await.unwrap();
        assert_eq!(last_used, "Z0000005");
    }

    #[tokio::test]
    async fn test_get_next_code() {
        let repo = Arc::new(FakeSequenceRepository::new("Z0000000"));
        let service = SequenceService::new(repo.clone());

        let code = service.get_next_code().await.unwrap();
        assert_eq!(code, "Z0000001");

        let last_used = repo.get_last_used_code().await.unwrap();
        assert_eq!(last_used, "Z0000001");
    }
}
