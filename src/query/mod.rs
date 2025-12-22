// Query module for the graph database
//
// This module provides the complete query processing pipeline including:
// - Parsing query strings into AST
// - Planning and optimizing execution plans
// - Executing queries against the storage engine
// - Managing query contexts and validation

// Sub-modules
pub mod context;
pub mod executor;
pub mod optimizer;
pub mod parser;
pub mod planner;
pub mod scheduler;
pub mod validator;
pub mod visitor;

// Module-specific implementations
// executor_factory和query_pipeline_manager已迁移到Core模块
// pub mod executor_factory;
// pub mod query_pipeline_manager;

// Re-export commonly used types for convenience
// pub use executor_factory::ExecutorFactory;
// pub use query_pipeline_manager::QueryPipelineManager;
// Re-export error types from core module
pub use crate::core::{DBResult, QueryError};
// Re-export execution result from executor module
pub use executor::traits::ExecutionResult;
