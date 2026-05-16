//! Secondary Index Module
//!
//! Property-based secondary indexes that support complex queries with MVCC.
//! These indexes are decoupled from the CSR structure and use BTreeMap for storage.

mod edge_index_manager;
mod index_data_manager;
mod index_gc_manager;
mod index_updater;
mod key_codec;
mod vertex_index_manager;

pub use edge_index_manager::EdgeIndexManager;
pub use index_data_manager::{
    GcStats, IndexDataManagerImpl, IndexDataManager, IndexEntry, Timestamp,
    INVALID_TIMESTAMP, MAX_TIMESTAMP,
};
pub use index_gc_manager::{IndexGcConfig, IndexGcManager};
pub use index_updater::{IndexUndoEntry, IndexUndoLog, IndexUpdateContext, IndexUpdater};
pub use key_codec::{
    ByteKey, CompressionConfig, DeltaCompressor, DictionaryCompressor, IndexCompressor, KeyBuilder,
    KeyParser, PrefixCompressor,
    KEY_TYPE_EDGE_FORWARD, KEY_TYPE_EDGE_REVERSE, KEY_TYPE_VERTEX_FORWARD, KEY_TYPE_VERTEX_REVERSE,
};
pub use key_codec::key_types::{deserialize_value, serialize_value, SecondaryIndexKey};
pub use vertex_index_manager::VertexIndexManager;
