pub mod api;
pub mod engine;
pub mod entity;
pub mod extend;
pub mod index;
pub mod iterator;
pub mod metadata;
pub mod monitoring;
pub mod operations;
pub mod schema;
pub mod shared_state;

#[cfg(test)]
pub mod test_mock;

// Re-export from api module
pub use api::{
    ColumnDef, EncodingFormat, FieldDef, FieldType, GeoShape, InsertEdgeInfo, InsertVertexInfo,
    StorageClient, StorageStats, UpdateInfo, UpdateOp, UpdateTarget,
};

// Re-export from engine module
pub use engine::{
    ByteKey, DefaultStorage, PlanContext, RedbStorage, RuntimeContext, StorageEnv,
    CURRENT_VERSIONS_TABLE, EDGES_TABLE, EDGE_DATA_TABLE, EDGE_TYPES_TABLE,
    EDGE_TYPE_ID_COUNTER_TABLE, EDGE_TYPE_NAME_INDEX_TABLE, INDEXES_TABLE, INDEX_COUNTER_TABLE,
    INDEX_DATA_TABLE, NODES_TABLE, PASSWORDS_TABLE, SCHEMA_CHANGES_TABLE, SCHEMA_VERSIONS_TABLE,
    SPACES_TABLE, SPACE_NAME_INDEX_TABLE, TAGS_TABLE, TAG_ID_COUNTER_TABLE, TAG_INDEXES_TABLE,
    TAG_NAME_INDEX_TABLE, VERTEX_DATA_TABLE,
};

// Re-export from entity module
pub use entity::{EdgeStorage, SyncStorage, UserStorage, VertexStorage};

// Re-export from extend module
pub use extend::FulltextStorage;

// Re-export from other modules
pub use index::*;
pub use iterator::*;
pub use metadata::*;
pub use operations::*;
pub use schema::Schema;
pub use shared_state::{StorageInner, StorageSharedState};

pub use crate::core::StorageError;
pub use crate::core::StorageResult;

#[cfg(test)]
pub use test_mock::*;
