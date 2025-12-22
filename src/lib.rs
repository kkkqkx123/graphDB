//! GraphDB - A lightweight single-node graph database implemented in Rust
//!
//! This crate provides the core functionality for a graph database that runs
//! as a single executable for personal and small-scale applications.

pub mod api;
pub mod cache;
pub mod common;
pub mod config;
pub mod core;
pub mod expression;
pub mod graph;
pub mod query;
pub mod services;
pub mod stats;
pub mod storage;
pub mod utils;

// Re-export common types at the crate root for convenience
pub use crate::config::Config;
pub use crate::core::{
    DateTimeValue, DateValue, Direction, DurationValue, Edge, GeographyValue, NullType, Path, Step,
    Tag, TimeValue, Value, Vertex,
};
pub use crate::query::{ExecutionResult, ExecutorFactory, QueryError, QueryPipelineManager};
pub use crate::storage::{NativeStorage, StorageEngine, StorageError};

// Re-export commonly used types from submodules
pub use crate::common::{
    base::id::*, charset::*, fs::*, log::*, memory::*, network::*, process::*, thread::*, time::*,
};
pub use crate::expression::*;
pub use crate::graph::{
    index::*,
    transaction::*,
    utils::{generate_id, is_valid_id, EPIdGenerator, IdGenerator, INVALID_ID},
};
pub use crate::services::{algorithm::*, context::*, function::*, session::*, stats::*};

