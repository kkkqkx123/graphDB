//! Type of execution result
//!
//! Defines the data structure of the actuator's execution result, supporting multiple result types.

use crate::core::error::DBError;
use crate::core::query_result::Result as CoreResult;

/// Type of execution result
///
/// Uniformly represents the execution results of all actuators and supports multiple data formats.
#[derive(Debug, Clone)]
pub enum ExecutionResult {
    /// Successful execution returns a list of generic values
    Values(Vec<crate::core::Value>),
    /// Successful execution, return vertex data
    Vertices(Vec<crate::core::Vertex>),
    /// Successful execution, return side data
    Edges(Vec<crate::core::Edge>),
    /// Successful execution returns structured dataset
    DataSet(crate::core::DataSet),
    /// Successful execution, return internal Result object
    Result(CoreResult),
    /// Successful execution, no data returned
    Empty,
    /// Successful execution, no data returned (alias)
    Success,
    /// implementation error
    Error(String),
    /// Return Count
    Count(usize),
    /// Return path
    Paths(Vec<crate::core::vertex_edge_path::Path>),
}

impl ExecutionResult {
    /// Get the count of elements in the result
    pub fn count(&self) -> usize {
        match self {
            ExecutionResult::Values(v) => v.len(),
            ExecutionResult::Vertices(v) => v.len(),
            ExecutionResult::Edges(v) => v.len(),
            ExecutionResult::DataSet(ds) => ds.rows.len(),
            ExecutionResult::Result(r) => r.row_count(),
            ExecutionResult::Count(c) => *c,
            ExecutionResult::Success => 0,
            ExecutionResult::Empty => 0,
            ExecutionResult::Error(_) => 0,
            ExecutionResult::Paths(p) => p.len(),
        }
    }

    /// Creating an ExecutionResult from a CoreResult
    pub fn from_result(result: CoreResult) -> Self {
        ExecutionResult::Result(result)
    }

    /// Convert to CoreResult
    pub fn to_result(&self) -> Option<CoreResult> {
        match self {
            ExecutionResult::Result(r) => Some(r.clone()),
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

impl IntoExecutionResult for Vec<crate::core::Value> {
    fn into_execution_result(self) -> ExecutionResult {
        ExecutionResult::Values(self)
    }
}

impl IntoExecutionResult for Vec<crate::core::Vertex> {
    fn into_execution_result(self) -> ExecutionResult {
        ExecutionResult::Vertices(self)
    }
}

impl IntoExecutionResult for Vec<crate::core::Edge> {
    fn into_execution_result(self) -> ExecutionResult {
        ExecutionResult::Edges(self)
    }
}

impl IntoExecutionResult for crate::core::DataSet {
    fn into_execution_result(self) -> ExecutionResult {
        ExecutionResult::DataSet(self)
    }
}

impl IntoExecutionResult for () {
    fn into_execution_result(self) -> ExecutionResult {
        ExecutionResult::Success
    }
}

impl IntoExecutionResult for usize {
    fn into_execution_result(self) -> ExecutionResult {
        ExecutionResult::Count(self)
    }
}
