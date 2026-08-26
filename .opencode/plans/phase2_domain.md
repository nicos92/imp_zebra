# Phase 2: Domain Layer - Implementation Plan

## Goal
Complete the domain layer with comprehensive tests, fix architectural issues, and ensure all domain logic is properly validated.

## Current State Summary

### ✅ Fully Implemented and Tested
| Module | Tests |
|--------|-------|
| `Sequence` entity | 12 tests |
| `Barcode` value object | 5 tests |
| `PrinterConfig` value object | 8 tests |
| `LabelService` | 2 tests |
| `ZplGenerator` | 3 tests |
| `LabelLayout` | 4 tests |

### ⚠️ Implemented but Untested
| Module | Tests |
|--------|-------|
| `Printer` entity | 0 tests |
| `PrintJob` entity | 0 tests |
| `SequenceService` | 0 tests |

### ❌ Issues Found
1. `DomainError::Database` leaks infrastructure into domain
2. `PrintJob` has no state transition guards
3. `PrintJobStatus::from_str` returns `Option` (inconsistent with `ConnectionType::from_str`)
4. `Barcode` value object is defined but never used by any other type
5. No IP address format validation in `PrinterConfig`
6. `LabelService` and `SequenceService` have overlapping responsibilities

---

## Task 1: Add Printer Entity Tests

**File:** `src-tauri/src/domain/entities/printer.rs`

**Tests to add:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::printer_config::{ConnectionType, PrinterConfig};

    fn valid_printer_config() -> PrinterConfig {
        PrinterConfig::new(
            "Test Printer",
            "Zebra ZD421",
            203,
            50.0,
            50.0,
            2,
            ConnectionType::Tcp,
            "192.168.1.100",
            9100,
        )
        .unwrap()
    }

    #[test]
    fn test_new_printer() {
        let printer = Printer::new("printer-1", &valid_printer_config());
        assert_eq!(printer.id, "printer-1");
        assert_eq!(printer.name, "Test Printer");
        assert_eq!(printer.dpi, 203);
    }

    #[test]
    fn test_update_printer() {
        let mut printer = Printer::new("printer-1", &valid_printer_config());
        let new_config = PrinterConfig::new(
            "Updated Printer",
            "Zebra ZD421",
            300,
            100.0,
            150.0,
            1,
            ConnectionType::Tcp,
            "192.168.1.101",
            9100,
        )
        .unwrap();
        
        printer.update(&new_config);
        assert_eq!(printer.name, "Updated Printer");
        assert_eq!(printer.dpi, 300);
        assert_eq!(printer.label_width_mm, 100.0);
    }

    #[test]
    fn test_to_config() {
        let printer = Printer::new("printer-1", &valid_printer_config());
        let config = printer.to_config();
        assert_eq!(config.name, "Test Printer");
        assert_eq!(config.dpi, 203);
    }

    #[test]
    fn test_address() {
        let printer = Printer::new("printer-1", &valid_printer_config());
        assert_eq!(printer.address(), "192.168.1.100:9100");
    }
}
```

**Expected: 4 tests**

---

## Task 2: Add PrintJob Entity Tests

**File:** `src-tauri/src/domain/entities/print_job.rs`

**Tests to add:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn valid_print_job() -> PrintJob {
        PrintJob::new(
            "job-1",
            "printer-1",
            "Z0000001",
            "Z0000010",
            10,
        )
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
        job.start_printing();
        assert_eq!(job.status, PrintJobStatus::Printing);
    }

    #[test]
    fn test_complete() {
        let mut job = valid_print_job();
        job.start_printing();
        job.complete();
        assert_eq!(job.status, PrintJobStatus::Completed);
        assert!(job.completed_at.is_some());
    }

    #[test]
    fn test_fail() {
        let mut job = valid_print_job();
        job.start_printing();
        job.fail();
        assert_eq!(job.status, PrintJobStatus::Failed);
        assert!(job.completed_at.is_some());
    }

    #[test]
    fn test_is_terminal() {
        let mut job = valid_print_job();
        assert!(!job.is_terminal());

        job.start_printing();
        assert!(!job.is_terminal());

        job.complete();
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
        assert_eq!(PrintJobStatus::from_str("pending"), Some(PrintJobStatus::Pending));
        assert_eq!(PrintJobStatus::from_str("printing"), Some(PrintJobStatus::Printing));
        assert_eq!(PrintJobStatus::from_str("completed"), Some(PrintJobStatus::Completed));
        assert_eq!(PrintJobStatus::from_str("failed"), Some(PrintJobStatus::Failed));
        assert_eq!(PrintJobStatus::from_str("invalid"), None);
    }
}
```

**Expected: 8 tests**

---

## Task 3: Add SequenceService Tests

**File:** `src-tauri/src/domain/services/sequence_service.rs`

**Tests to add (using a fake repository):**
```rust
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
        
        // Verify persistence
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
```

**Expected: 3 tests**

---

## Task 4: Fix Architectural Issues

### 4a. Remove `DomainError::Database` variant
**File:** `src-tauri/src/errors/domain_error.rs`

**Change:** Remove the `Database(String)` variant. Database errors should only exist in `InfrastructureError`.

**Impact:** Need to update any code that converts `DomainError::Database` to `InfrastructureError`.

### 4b. Add State Transition Validation to PrintJob
**File:** `src-tauri/src/domain/entities/print_job.rs`

**Change:** Add guards to prevent invalid transitions:
```rust
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
```

**New Error Variant:**
```rust
#[error("Invalid state transition from {from} to {to}")]
InvalidStateTransition { from: String, to: String },
```

### 4c. Make `PrintJobStatus::from_str` return `Result`
**File:** `src-tauri/src/domain/entities/print_job.rs`

**Change:**
```rust
pub fn from_str(s: &str) -> Result<Self, DomainError> {
    match s {
        "pending" => Ok(Self::Pending),
        "printing" => Ok(Self::Printing),
        "completed" => Ok(Self::Completed),
        "failed" => Ok(Self::Failed),
        _ => Err(DomainError::InvalidPrintJobStatus(s.to_string())),
    }
}
```

**New Error Variant:**
```rust
#[error("Invalid print job status: {0}")]
InvalidPrintJobStatus(String),
```

### 4d. Add IP Address Format Validation to PrinterConfig
**File:** `src-tauri/src/domain/value_objects/printer_config.rs`

**Change:** Add basic IP format validation:
```rust
fn is_valid_ip(ip: &str) -> bool {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    parts.iter().all(|p| p.parse::<u8>().is_ok())
}
```

**Update validation:**
```rust
if !is_valid_ip(&ip_address) {
    return Err(DomainError::InvalidPrinterConfig {
        field: "ip_address".to_string(),
        message: "Invalid IP address format".to_string(),
    });
}
```

**New tests:**
```rust
#[test]
fn test_invalid_ip_address() {
    let result = PrinterConfig::new(
        "Test", "Zebra", 203, 50.0, 50.0, 2,
        ConnectionType::Tcp, "invalid-ip", 9100,
    );
    assert!(result.is_err());
}

#[test]
fn test_valid_ip_address() {
    let result = PrinterConfig::new(
        "Test", "Zebra", 203, 50.0, 50.0, 2,
        ConnectionType::Tcp, "192.168.1.100", 9100,
    );
    assert!(result.is_ok());
}
```

---

## Task 5: Integrate or Remove Barcode

**Decision:** Since `Barcode` is defined but never used, and `Sequence` already handles code validation, we should either:

**Option A (Recommended):** Keep `Barcode` as a standalone value object that can be used by the frontend or future features. Document its purpose.

**Option B:** Remove it to avoid dead code.

**Recommendation:** Keep it for now. It's a well-designed value object that may be useful for:
- Frontend validation
- Future features (e.g., manual barcode entry)
- Type safety in specific contexts

---

## Task 6: Resolve LabelService vs SequenceService Overlap

**Current State:**
- `LabelService`: Sync, no repo, generates codes from a start code
- `SequenceService`: Async, uses repo, manages sequence with persistence

**Resolution:** Keep both but clarify responsibilities:
- `LabelService`: Pure utility for code generation and position calculation (no I/O)
- `SequenceService`: Stateful service that manages sequence persistence

**No code changes needed** - the current design is actually correct:
- `LabelService` is used for preview/position calculation (no persistence needed)
- `SequenceService` is used for actual printing (persistence required)

---

## Expected Test Count After Phase 2

| Module | Before | After |
|--------|--------|-------|
| Sequence | 12 | 12 |
| Barcode | 5 | 5 |
| PrinterConfig | 8 | 10 (+2 IP tests) |
| LabelService | 2 | 2 |
| ZplGenerator | 3 | 3 |
| LabelLayout | 4 | 4 |
| Printer | 0 | 4 |
| PrintJob | 0 | 8 |
| SequenceService | 0 | 3 |
| **Total** | **34** | **51** |

---

## Execution Order

1. **Task 4d:** Add IP validation to PrinterConfig (+2 tests)
2. **Task 4a:** Remove `DomainError::Database` variant
3. **Task 4b:** Add state transition validation to PrintJob (+1 error variant)
4. **Task 4c:** Make `PrintJobStatus::from_str` return Result (+1 error variant)
5. **Task 1:** Add Printer entity tests (+4 tests)
6. **Task 2:** Add PrintJob entity tests (+8 tests, updated for new Result returns)
7. **Task 3:** Add SequenceService tests (+3 tests)
8. **Verify:** Run `cargo test` to ensure all 51 tests pass
9. **Update docs:** Add Phase 2 summary to `docs/DEVELOPMENT.md`

---

## Files to Modify

| File | Changes |
|------|---------|
| `src-tauri/src/errors/domain_error.rs` | Remove `Database`, add `InvalidStateTransition`, `InvalidPrintJobStatus` |
| `src-tauri/src/errors/application_error.rs` | Update if affected by DomainError changes |
| `src-tauri/src/domain/entities/printer.rs` | Add tests |
| `src-tauri/src/domain/entities/print_job.rs` | Add state guards, fix from_str, add tests |
| `src-tauri/src/domain/services/sequence_service.rs` | Add tests |
| `src-tauri/src/domain/value_objects/printer_config.rs` | Add IP validation, tests |
| `src-tauri/src/infrastructure/database/sqlite_print_job_repository.rs` | Update if affected by PrintJob changes |
| `src-tauri/src/application/use_cases/print_labels.rs` | Update if affected by PrintJob changes |
