// Query module for the graph database
//
// This module provides the complete query processing pipeline including:
// - Parsing query strings into AST
// - Planning and optimizing execution plans
// - Executing queries against the storage engine
// - Managing query contexts and validation

// Sub-modules
pub mod context;
pub mod core;
pub mod executor;
pub mod optimizer;
pub mod parser;
pub mod planner;
pub mod query_pipeline_manager;
pub mod validator;
pub mod visitor;
// Re-export error types from core module
pub use crate::core::{DBResult, QueryError};
// Re-export execution result from executor module
pub use executor::traits::ExecutionResult;
// Re-export QueryPipelineManager
pub use query_pipeline_manager::QueryPipelineManager;
