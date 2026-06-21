//! VertexTable module organization
//!
//! Split into logical components:
//! - core: CRUD operations and queries
//! - persistence: File I/O, serialization
//! - optimizer: Compaction and optimization
//! - schema: Schema management

pub mod core;
pub mod optimizer;
pub mod persistence;
pub mod schema;

pub use core::{VertexTable, VertexTableConfig, VertexIterator};
