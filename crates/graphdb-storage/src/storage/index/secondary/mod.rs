//! Secondary Index Module
//!
//! Property-based secondary indexes that support complex queries with MVCC.
//! These indexes are decoupled from the CSR structure and use BTreeMap for storage.

mod edge_index_manager;
mod generic_index_manager;
mod index_data_manager;
mod index_gc_manager;
mod index_updater;
mod key_codec;
mod vertex_index_manager;

pub use index_data_manager::{
    EdgeIndexOps, GcStats, IndexDataManagerImpl, IndexGcOps, VertexIndexOps,
};
pub use index_gc_manager::{IndexGcConfig, IndexGcManager};
