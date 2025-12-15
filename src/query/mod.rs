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
pub mod query_converter;
pub mod query_executor;
pub mod types;

// Re-export commonly used types for convenience
pub use query_converter::QueryConverter;
pub use query_executor::QueryExecutor;
pub use types::{Condition, Query, QueryError, QueryResult};
