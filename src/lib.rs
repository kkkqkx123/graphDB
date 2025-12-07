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
pub mod expression;
pub mod graph;
pub mod context;
pub mod network;

// Re-export common types at the crate root for convenience
pub use crate::core::{Vertex, Edge, Value, Direction, Tag, Path, Step, NullType, DateValue, TimeValue, DateTimeValue, GeographyValue, DurationValue, error::{GraphDBError, GraphDBResult}};
pub use crate::storage::{StorageEngine, NativeStorage, StorageError};
pub use crate::query::{Query, QueryResult, QueryExecutor, QueryError};
pub use crate::config::Config;