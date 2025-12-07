//! GraphDB - A lightweight single-node graph database implemented in Rust
//! 
//! This crate provides the core functionality for a graph database that runs
//! as a single executable for personal and small-scale applications.

pub mod core;
pub mod storage;
pub mod query;
pub mod transaction;
pub mod index;
pub mod api;
pub mod utils;
pub mod config;

// Re-export common types at the crate root for convenience
pub use crate::core::{Node, Edge, Value, Direction};
pub use crate::storage::{StorageEngine, NativeStorage, StorageError};
pub use crate::query::{Query, QueryResult, QueryExecutor, QueryError};
pub use crate::config::Config;