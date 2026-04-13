//! Type of execution result
//!
//! Defines the data structure of the actuator's execution result, supporting multiple result types.

use crate::core::error::DBError;
use crate::query::data_set::DataSet;

/// Type of execution result
///
/// Uniformly represents the execution results of all actuators and supports multiple data formats.
#[derive(Debug, Clone)]
pub enum ExecutionResult {
    /// Successful execution returns structured dataset (primary result type)
    DataSet(DataSet),
    /// Successful execution, no data returned
    Empty,
    /// Successful execution, no data returned (alias)
    Success,
    /// implementation error
    Error(String),
}

impl ExecutionResult {
    /// Get the count of elements in the result
    pub fn count(&self) -> usize {
        match self {
            ExecutionResult::DataSet(ds) => ds.row_count(),
            ExecutionResult::Success => 0,
            ExecutionResult::Empty => 0,
            ExecutionResult::Error(_) => 0,
        }
    }

    /// Creating an ExecutionResult from a DataSet
    pub fn from_data_set(data: DataSet) -> Self {
        ExecutionResult::DataSet(data)
    }

    /// Convert to DataSet
    pub fn to_data_set(&self) -> Option<&DataSet> {
        match self {
            ExecutionResult::DataSet(ds) => Some(ds),
            _ => None,
        }
    }
}

/// Result type alias
pub type DBResult<T> = Result<T, DBError>;

/// Support for traits that are converted to execution results
pub trait IntoExecutionResult {
    fn into_execution_result(self) -> ExecutionResult;
}

impl IntoExecutionResult for DataSet {
    fn into_execution_result(self) -> ExecutionResult {
        ExecutionResult::DataSet(self)
    }
}

impl IntoExecutionResult for () {
    fn into_execution_result(self) -> ExecutionResult {
        ExecutionResult::Success
    }
}
