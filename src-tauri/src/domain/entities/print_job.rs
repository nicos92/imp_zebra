use crate::errors::domain_error::DomainError;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrintJobStatus {
    Pending,
    Printing,
    Completed,
    Failed,
}

impl PrintJobStatus {
    pub fn as_str(&self) -> &str {
        match self {
            PrintJobStatus::Pending => "pending",
            PrintJobStatus::Printing => "printing",
            PrintJobStatus::Completed => "completed",
            PrintJobStatus::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, DomainError> {
        match s {
            "pending" => Ok(PrintJobStatus::Pending),
            "printing" => Ok(PrintJobStatus::Printing),
            "completed" => Ok(PrintJobStatus::Completed),
            "failed" => Ok(PrintJobStatus::Failed),
            _ => Err(DomainError::InvalidPrintJobStatus(s.to_string())),
        }
    }
}

impl std::fmt::Display for PrintJobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct PrintJob {
    pub id: String,
    pub printer_id: String,
    pub start_code: String,
    pub end_code: String,
    pub quantity: u64,
    pub status: PrintJobStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl PrintJob {
    pub fn new(
        id: &str,
        printer_id: &str,
        start_code: &str,
        end_code: &str,
        quantity: u64,
    ) -> Self {
        Self {
            id: id.to_string(),
            printer_id: printer_id.to_string(),
            start_code: start_code.to_string(),
            end_code: end_code.to_string(),
            quantity,
            status: PrintJobStatus::Pending,
            created_at: Utc::now(),
            completed_at: None,
        }
    }

    pub fn start_printing(&mut self) -> Result<(), DomainError> {
        if self.status != PrintJobStatus::Pending {
            return Err(DomainError::InvalidStateTransition {
                from: self.status.as_str().to_string(),
                to: "printing".to_string(),
            });
        }
        self.status = PrintJobStatus::Printing;
        Ok(())
    }

    pub fn complete(&mut self) -> Result<(), DomainError> {
        if self.status != PrintJobStatus::Printing {
            return Err(DomainError::InvalidStateTransition {
                from: self.status.as_str().to_string(),
                to: "completed".to_string(),
            });
        }
        self.status = PrintJobStatus::Completed;
        self.completed_at = Some(Utc::now());
        Ok(())
    }

    pub fn fail(&mut self) -> Result<(), DomainError> {
        if self.status != PrintJobStatus::Printing {
            return Err(DomainError::InvalidStateTransition {
                from: self.status.as_str().to_string(),
                to: "failed".to_string(),
            });
        }
        self.status = PrintJobStatus::Failed;
        self.completed_at = Some(Utc::now());
        Ok(())
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            PrintJobStatus::Completed | PrintJobStatus::Failed
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_print_job() -> PrintJob {
        PrintJob::new("job-1", "printer-1", "Z0000001", "Z0000010", 10)
    }

    #[test]
    fn test_new_print_job() {
        let job = valid_print_job();
        assert_eq!(job.id, "job-1");
        assert_eq!(job.printer_id, "printer-1");
        assert_eq!(job.start_code, "Z0000001");
        assert_eq!(job.end_code, "Z0000010");
        assert_eq!(job.quantity, 10);
        assert_eq!(job.status, PrintJobStatus::Pending);
        assert!(job.completed_at.is_none());
    }

    #[test]
    fn test_start_printing() {
        let mut job = valid_print_job();
        job.start_printing().unwrap();
        assert_eq!(job.status, PrintJobStatus::Printing);
    }

    #[test]
    fn test_start_printing_from_invalid_state() {
        let mut job = valid_print_job();
        job.start_printing().unwrap();
        job.complete().unwrap();

        let result = job.start_printing();
        assert!(result.is_err());
    }

    #[test]
    fn test_complete() {
        let mut job = valid_print_job();
        job.start_printing().unwrap();
        job.complete().unwrap();
        assert_eq!(job.status, PrintJobStatus::Completed);
        assert!(job.completed_at.is_some());
    }

    #[test]
    fn test_complete_from_invalid_state() {
        let mut job = valid_print_job();

        let result = job.complete();
        assert!(result.is_err());
    }

    #[test]
    fn test_fail() {
        let mut job = valid_print_job();
        job.start_printing().unwrap();
        job.fail().unwrap();
        assert_eq!(job.status, PrintJobStatus::Failed);
        assert!(job.completed_at.is_some());
    }

    #[test]
    fn test_fail_from_invalid_state() {
        let mut job = valid_print_job();

        let result = job.fail();
        assert!(result.is_err());
    }

    #[test]
    fn test_is_terminal() {
        let mut job = valid_print_job();
        assert!(!job.is_terminal());

        job.start_printing().unwrap();
        assert!(!job.is_terminal());

        job.complete().unwrap();
        assert!(job.is_terminal());
    }

    #[test]
    fn test_status_as_str() {
        assert_eq!(PrintJobStatus::Pending.as_str(), "pending");
        assert_eq!(PrintJobStatus::Printing.as_str(), "printing");
        assert_eq!(PrintJobStatus::Completed.as_str(), "completed");
        assert_eq!(PrintJobStatus::Failed.as_str(), "failed");
    }

    #[test]
    fn test_status_from_str() {
        assert_eq!(
            PrintJobStatus::from_str("pending").unwrap(),
            PrintJobStatus::Pending
        );
        assert_eq!(
            PrintJobStatus::from_str("printing").unwrap(),
            PrintJobStatus::Printing
        );
        assert_eq!(
            PrintJobStatus::from_str("completed").unwrap(),
            PrintJobStatus::Completed
        );
        assert_eq!(
            PrintJobStatus::from_str("failed").unwrap(),
            PrintJobStatus::Failed
        );
        assert!(PrintJobStatus::from_str("invalid").is_err());
    }
}
