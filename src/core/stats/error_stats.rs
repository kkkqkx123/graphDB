//! Error Statistics Module
//!
//! Provide functions for error type identification, error stage determination, and error statistics.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Number of variants in the ErrorType enum (used for determining the array size)
const ERROR_TYPE_COUNT: usize = 11;
/// Number of variants in the QueryPhase enumeration (used for determining the array size)
const QUERY_PHASE_COUNT: usize = 5;

/// Convert ErrorType to an array index.
pub fn error_type_to_index(error_type: ErrorType) -> usize {
    match error_type {
        ErrorType::ParseError => 0,
        ErrorType::ValidationError => 1,
        ErrorType::PlanningError => 2,
        ErrorType::OptimizationError => 3,
        ErrorType::ExecutionError => 4,
        ErrorType::StorageError => 5,
        ErrorType::TimeoutError => 6,
        ErrorType::MemoryLimitError => 7,
        ErrorType::PermissionError => 8,
        ErrorType::SessionError => 9,
        ErrorType::OtherError => 10,
    }
}

/// Convert the array index to an ErrorType.
pub fn index_to_error_type(index: usize) -> Option<ErrorType> {
    match index {
        0 => Some(ErrorType::ParseError),
        1 => Some(ErrorType::ValidationError),
        2 => Some(ErrorType::PlanningError),
        3 => Some(ErrorType::OptimizationError),
        4 => Some(ErrorType::ExecutionError),
        5 => Some(ErrorType::StorageError),
        6 => Some(ErrorType::TimeoutError),
        7 => Some(ErrorType::MemoryLimitError),
        8 => Some(ErrorType::PermissionError),
        9 => Some(ErrorType::SessionError),
        10 => Some(ErrorType::OtherError),
        _ => None,
    }
}

/// Convert QueryPhase to an array index.
pub fn query_phase_to_index(phase: QueryPhase) -> usize {
    match phase {
        QueryPhase::Parse => 0,
        QueryPhase::Validate => 1,
        QueryPhase::Plan => 2,
        QueryPhase::Optimize => 3,
        QueryPhase::Execute => 4,
    }
}

/// Convert the array index to a QueryPhase.
pub fn index_to_query_phase(index: usize) -> Option<QueryPhase> {
    match index {
        0 => Some(QueryPhase::Parse),
        1 => Some(QueryPhase::Validate),
        2 => Some(QueryPhase::Plan),
        3 => Some(QueryPhase::Optimize),
        4 => Some(QueryPhase::Execute),
        _ => None,
    }
}

/// Query Execution Phase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QueryPhase {
    Parse,
    Validate,
    Plan,
    Optimize,
    Execute,
}

impl std::fmt::Display for QueryPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryPhase::Parse => write!(f, "parse"),
            QueryPhase::Validate => write!(f, "validate"),
            QueryPhase::Plan => write!(f, "plan"),
            QueryPhase::Optimize => write!(f, "optimize"),
            QueryPhase::Execute => write!(f, "execute"),
        }
    }
}

/// Error type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorType {
    ParseError,
    ValidationError,
    PlanningError,
    OptimizationError,
    ExecutionError,
    StorageError,
    TimeoutError,
    MemoryLimitError,
    PermissionError,
    SessionError,
    OtherError,
}

impl std::fmt::Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorType::ParseError => write!(f, "parse_error"),
            ErrorType::ValidationError => write!(f, "validation_error"),
            ErrorType::PlanningError => write!(f, "planning_error"),
            ErrorType::OptimizationError => write!(f, "optimization_error"),
            ErrorType::ExecutionError => write!(f, "execution_error"),
            ErrorType::StorageError => write!(f, "storage_error"),
            ErrorType::TimeoutError => write!(f, "timeout_error"),
            ErrorType::MemoryLimitError => write!(f, "memory_limit_error"),
            ErrorType::PermissionError => write!(f, "permission_error"),
            ErrorType::SessionError => write!(f, "session_error"),
            ErrorType::OtherError => write!(f, "other_error"),
        }
    }
}

/// Extended error message
#[derive(Debug, Clone)]
pub struct ErrorInfo {
    pub error_type: ErrorType,
    pub error_phase: QueryPhase,
    pub error_message: String,
    pub error_details: Option<String>,
}

impl ErrorInfo {
    pub fn new(
        error_type: ErrorType,
        error_phase: QueryPhase,
        error_message: impl Into<String>,
    ) -> Self {
        Self {
            error_type,
            error_phase,
            error_message: error_message.into(),
            error_details: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.error_details = Some(details.into());
        self
    }
}

/// Error Statistics Summary
#[derive(Debug, Clone)]
pub struct ErrorSummary {
    pub total_errors: u64,
    pub errors_by_type: HashMap<ErrorType, u64>,
    pub errors_by_phase: HashMap<QueryPhase, u64>,
}

/// Error Statistics Manager
pub struct ErrorStatsManager {
    error_counts: [AtomicU64; ERROR_TYPE_COUNT],
    error_by_phase: [AtomicU64; QUERY_PHASE_COUNT],
}

impl ErrorStatsManager {
    pub fn new() -> Self {
        Self {
            error_counts: std::array::from_fn(|_| AtomicU64::new(0)),
            error_by_phase: std::array::from_fn(|_| AtomicU64::new(0)),
        }
    }

    pub fn record_error(&self, error_type: ErrorType, phase: QueryPhase) {
        let index = error_type_to_index(error_type);
        self.error_counts[index].fetch_add(1, Ordering::Relaxed);

        let phase_index = query_phase_to_index(phase);
        self.error_by_phase[phase_index].fetch_add(1, Ordering::Relaxed);

        log::warn!("查询错误: type={}, phase={}", error_type, phase);
    }

    pub fn get_error_count(&self, error_type: ErrorType) -> u64 {
        let index = error_type_to_index(error_type);
        self.error_counts[index].load(Ordering::Relaxed)
    }

    pub fn get_error_count_by_phase(&self, phase: QueryPhase) -> u64 {
        let index = query_phase_to_index(phase);
        self.error_by_phase[index].load(Ordering::Relaxed)
    }

    pub fn get_all_error_counts(&self) -> HashMap<ErrorType, u64> {
        let mut result = HashMap::new();
        for i in 0..ERROR_TYPE_COUNT {
            if let Some(error_type) = index_to_error_type(i) {
                let count = self.error_counts[i].load(Ordering::Relaxed);
                if count > 0 {
                    result.insert(error_type, count);
                }
            }
        }
        result
    }

    pub fn get_all_error_counts_by_phase(&self) -> HashMap<QueryPhase, u64> {
        let mut result = HashMap::new();
        for i in 0..QUERY_PHASE_COUNT {
            if let Some(phase) = index_to_query_phase(i) {
                let count = self.error_by_phase[i].load(Ordering::Relaxed);
                if count > 0 {
                    result.insert(phase, count);
                }
            }
        }
        result
    }

    pub fn reset_error_counts(&self) {
        for counter in &self.error_counts {
            counter.store(0, Ordering::Relaxed);
        }
        for counter in &self.error_by_phase {
            counter.store(0, Ordering::Relaxed);
        }
    }

    pub fn get_error_summary(&self) -> ErrorSummary {
        ErrorSummary {
            total_errors: self
                .error_counts
                .iter()
                .map(|c| c.load(Ordering::Relaxed))
                .sum(),
            errors_by_type: self.get_all_error_counts(),
            errors_by_phase: self.get_all_error_counts_by_phase(),
        }
    }
}

impl Default for ErrorStatsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_stats_recording() {
        let stats = ErrorStatsManager::new();

        stats.record_error(ErrorType::ParseError, QueryPhase::Parse);
        stats.record_error(ErrorType::ParseError, QueryPhase::Parse);
        stats.record_error(ErrorType::ExecutionError, QueryPhase::Execute);

        assert_eq!(stats.get_error_count(ErrorType::ParseError), 2);
        assert_eq!(stats.get_error_count(ErrorType::ExecutionError), 1);
        assert_eq!(stats.get_error_count(ErrorType::StorageError), 0);

        assert_eq!(stats.get_error_count_by_phase(QueryPhase::Parse), 2);
        assert_eq!(stats.get_error_count_by_phase(QueryPhase::Execute), 1);
        assert_eq!(stats.get_error_count_by_phase(QueryPhase::Validate), 0);
    }

    #[test]
    fn test_error_stats_summary() {
        let stats = ErrorStatsManager::new();

        stats.record_error(ErrorType::ParseError, QueryPhase::Parse);
        stats.record_error(ErrorType::ValidationError, QueryPhase::Validate);
        stats.record_error(ErrorType::ExecutionError, QueryPhase::Execute);

        let summary = stats.get_error_summary();
        assert_eq!(summary.total_errors, 3);
        assert_eq!(summary.errors_by_type.get(&ErrorType::ParseError), Some(&1));
        assert_eq!(summary.errors_by_phase.get(&QueryPhase::Parse), Some(&1));
    }

    #[test]
    fn test_error_stats_reset() {
        let stats = ErrorStatsManager::new();

        stats.record_error(ErrorType::ParseError, QueryPhase::Parse);
        assert_eq!(stats.get_error_count(ErrorType::ParseError), 1);

        stats.reset_error_counts();
        assert_eq!(stats.get_error_count(ErrorType::ParseError), 0);
    }
}
