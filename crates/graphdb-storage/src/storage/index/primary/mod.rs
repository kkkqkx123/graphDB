//! Primary Index Module
//!
//! CSR-aware primary indexes that are tightly coupled with the storage structure.
//! These indexes provide fast access to data by internal IDs and are automatically maintained.

mod degree_index;
mod edge_id_index;
mod primary_index_manager;

pub use degree_index::DegreeIndex;
pub use edge_id_index::EdgeIdIndex;
