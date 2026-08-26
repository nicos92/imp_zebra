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

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(PrintJobStatus::Pending),
            "printing" => Some(PrintJobStatus::Printing),
            "completed" => Some(PrintJobStatus::Completed),
            "failed" => Some(PrintJobStatus::Failed),
            _ => None,
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
    pub fn new(id: &str, printer_id: &str, start_code: &str, end_code: &str, quantity: u64) -> Self {
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

    pub fn start_printing(&mut self) {
        self.status = PrintJobStatus::Printing;
    }

    pub fn complete(&mut self) {
        self.status = PrintJobStatus::Completed;
        self.completed_at = Some(Utc::now());
    }

    pub fn fail(&mut self) {
        self.status = PrintJobStatus::Failed;
        self.completed_at = Some(Utc::now());
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            PrintJobStatus::Completed | PrintJobStatus::Failed
        )
    }
}
