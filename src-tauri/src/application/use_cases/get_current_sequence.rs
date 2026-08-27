use std::sync::Arc;

use crate::application::dto::print_dto::SequenceInfoDto;
use crate::domain::services::sequence_service::SequenceService;
use crate::errors::application_error::ApplicationError;

pub struct GetCurrentSequence {
    sequence_service: Arc<SequenceService>,
}

impl GetCurrentSequence {
    pub fn new(sequence_service: Arc<SequenceService>) -> Self {
        Self { sequence_service }
    }

    pub async fn execute(&self) -> Result<SequenceInfoDto, ApplicationError> {
        let sequence = self
            .sequence_service
            .get_current_sequence()
            .await
            .map_err(ApplicationError::Domain)?;

        let next_code = {
            let mut seq = sequence.clone();
            seq.next()
        };

        Ok(SequenceInfoDto {
            last_used_code: sequence.last_used_code(),
            next_code,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::sequence::Sequence;
    use crate::domain::repositories::sequence_repository::SequenceRepository;
    use crate::errors::domain_error::DomainError;
    use std::sync::Mutex;

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
            let current = {
                let mut seq = Sequence::from_code(&self.get_last_used_code().await?)?;
                let (start, end, _) = seq.reserve_range(quantity)?;
                self.update_last_used_code(&seq.last_used_code()).await?;
                (start, end, seq.last_used_code())
            };
            Ok(current)
        }
    }

    #[tokio::test]
    async fn test_get_current_sequence_returns_last_and_next() {
        let repo = Arc::new(FakeSequenceRepository::new("Z0000005"));
        let service = Arc::new(SequenceService::new(repo));
        let use_case = GetCurrentSequence::new(service);

        let info = use_case.execute().await.unwrap();
        assert_eq!(info.last_used_code, "Z0000005");
        assert_eq!(info.next_code, "Z0000006");
    }

    #[tokio::test]
    async fn test_get_current_sequence_rollover_next() {
        let repo = Arc::new(FakeSequenceRepository::new("Z9999999"));
        let service = Arc::new(SequenceService::new(repo));
        let use_case = GetCurrentSequence::new(service);

        let info = use_case.execute().await.unwrap();
        assert_eq!(info.last_used_code, "Z9999999");
        assert_eq!(info.next_code, "Z0000001");
    }
}
