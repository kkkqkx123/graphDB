pub mod edge_storage;
pub mod event_storage;
pub mod fulltext_storage;
pub mod index;
pub mod iterator;
pub mod metadata;
pub mod monitoring;
pub mod operations;
pub mod redb_storage;
pub mod redb_types;
pub mod runtime_context;
pub mod schema;
pub mod shared_state;
pub mod storage_client;
pub mod types;
pub mod user_storage;
pub mod vertex_storage;

#[cfg(test)]
pub mod test_mock;

pub use edge_storage::EdgeStorage;
pub use event_storage::SyncStorage;
pub use fulltext_storage::FulltextStorage;
pub use index::*;
pub use iterator::*;
pub use metadata::*;
pub use operations::*;
pub use redb_storage::DefaultStorage;
pub use redb_storage::*;
pub use storage_client::*;
pub use user_storage::UserStorage;
pub use vertex_storage::VertexStorage;

pub use crate::core::StorageError;
pub use crate::core::StorageResult;

#[cfg(test)]
pub use test_mock::*;

// Types related to data export encoding
pub use schema::Schema;
pub use types::{ColumnDef, FieldDef, FieldType};
pub use types::{InsertEdgeInfo, InsertVertexInfo, UpdateInfo, UpdateOp, UpdateTarget};

// Export the runtime context type.
pub use runtime_context::{PlanContext, RuntimeContext, StorageEnv};
