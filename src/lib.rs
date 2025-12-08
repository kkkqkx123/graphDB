//! GraphDB - A lightweight single-node graph database implemented in Rust
//! 
//! This crate provides the core functionality for a graph database that runs
//! as a single executable for personal and small-scale applications.

pub mod core;
pub mod storage;
pub mod query;
pub mod api;
pub mod utils;
pub mod config;
pub mod common;
pub mod graph;
pub mod services;

// Re-export common types at the crate root for convenience
pub use crate::core::{Vertex, Edge, Value, Direction, Tag, Path, Step, NullType, DateValue, TimeValue, DateTimeValue, GeographyValue, DurationValue, error::{Status, StatusOr}};
pub use crate::storage::{StorageEngine, NativeStorage, StorageError};
pub use crate::query::{Query, QueryResult, QueryExecutor, QueryError};
pub use crate::config::Config;

// Re-export commonly used types from submodules
pub use crate::common::{base::id::*, time::*, memory::*, thread::*, process::*, network::*, fs::*, log::*, charset::*};
pub use crate::graph::{transaction::*, index::*, expression::*, utils::{IdGenerator, EPIdGenerator, INVALID_ID, generate_id, is_valid_id}};
pub use crate::services::{session::*, stats::*, function::*, algorithm::*, context::*};