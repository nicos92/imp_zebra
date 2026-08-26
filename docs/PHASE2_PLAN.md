# Phase 2: Domain Layer - Implementation Plan

## Goal
Complete the domain layer with comprehensive tests, fix architectural issues, and ensure all domain logic is properly validated.

## Current State Summary

### Fully Implemented and Tested
| Module | Tests |
|--------|-------|
| `Sequence` entity | 12 tests |
| `Barcode` value object | 5 tests |
| `PrinterConfig` value object | 8 tests |
| `LabelService` | 2 tests |
| `ZplGenerator` | 3 tests |
| `LabelLayout` | 4 tests |

### Implemented but Untested
| Module | Tests |
|--------|-------|
| `Printer` entity | 0 tests |
| `PrintJob` entity | 0 tests |
| `SequenceService` | 0 tests |

### Issues Found
1. `PrintJob` has no state transition guards
2. `PrintJobStatus::from_str` returns `Option` (inconsistent with `ConnectionType::from_str`)
3. No IP address format validation in `PrinterConfig`
4. `DomainError::Database` leaks infrastructure into domain (kept as pragmatic compromise - repository traits return `DomainError`)

---

## Task 1: Add Printer Entity Tests

**File:** `src-tauri/src/domain/entities/printer.rs`

Tests: new, update, to_config, address (4 tests)

---

## Task 2: Add PrintJob Entity Tests

**File:** `src-tauri/src/domain/entities/print_job.rs`

Tests: new, start_printing, complete, fail, is_terminal, status_as_str, status_from_str (7 tests)

---

## Task 3: Add SequenceService Tests

**File:** `src-tauri/src/domain/services/sequence_service.rs`

Tests with fake repository: get_current_sequence, reserve_range, get_next_code (3 tests)

---

## Task 4: Fix Architectural Issues

### 4a. Keep `DomainError::Database` (pragmatic compromise)
Repository traits return `DomainError`, so infrastructure must map errors through it.

### 4b. Add State Transition Validation to PrintJob
Methods return `Result<(), DomainError>` with `InvalidStateTransition` error.

### 4c. Make `PrintJobStatus::from_str` return `Result`
Returns `Result<Self, DomainError>` with `InvalidPrintJobStatus` error.

### 4d. Add IP Address Format Validation to PrinterConfig
Validates IPv4 format (4 octets, 0-255).

---

## Expected Test Count

| Module | Before | After |
|--------|--------|-------|
| Sequence | 12 | 12 |
| Barcode | 5 | 5 |
| PrinterConfig | 8 | 10 (+2 IP tests) |
| LabelService | 2 | 2 |
| ZplGenerator | 3 | 3 |
| LabelLayout | 4 | 4 |
| Printer | 0 | 4 |
| PrintJob | 0 | 7 |
| SequenceService | 0 | 3 |
| **Total** | **34** | **50** |

---

## Files to Modify

| File | Changes |
|------|---------|
| `src-tauri/src/errors/domain_error.rs` | Add `InvalidStateTransition`, `InvalidPrintJobStatus` |
| `src-tauri/src/domain/entities/printer.rs` | Add tests |
| `src-tauri/src/domain/entities/print_job.rs` | Add state guards, fix from_str, add tests |
| `src-tauri/src/domain/services/sequence_service.rs` | Add tests |
| `src-tauri/src/domain/value_objects/printer_config.rs` | Add IP validation, tests |
