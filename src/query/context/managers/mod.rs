//! 管理器接口模块
//!
//! 包含所有管理器接口的定义，用于管理数据库的各种资源

pub mod r#impl;
pub mod index_manager;
pub mod meta_client;
pub mod retry;
pub mod schema_manager;
pub mod schema_traits;
pub mod storage_client;
pub mod transaction;
pub mod types;

// 重新导出所有公共类型和trait
pub use crate::core::error::{ErrorCategory, ManagerError};
pub use index_manager::{Index, IndexManager, IndexStatus, IndexType, IndexStats, IndexOptimization};
pub use meta_client::MetaClient;
pub use r#impl::*;
pub use retry::{retry_with_backoff, retry_with_strategy, RetryConfig, RetryStrategy};
pub use schema_manager::SchemaManager;
pub use schema_traits::{
    SchemaReader, SchemaWriter, SchemaVersionControl, SchemaPersistence, SchemaImportExport,
    SchemaManagerBuilder,
};
pub use storage_client::{
    DelTags, EdgeKey, ExecResponse, NewEdge, NewTag, NewVertex, StorageClient, StorageOperation,
    StorageResponse, UpdateResponse, UpdatedProp,
};
pub use transaction::{
    IsolationLevel, Transaction, TransactionId, TransactionManager, TransactionOperation,
    TransactionState,
};
pub use types::{
    CharsetInfo, ClusterInfo, EdgeTypeDef, EdgeTypeDefWithId, FieldDef, MetadataVersion,
    PropertyDef, PropertyType, Schema, SchemaChange, SchemaChangeType, SchemaExportConfig,
    SchemaHistory, SchemaImportResult, SchemaVersion, SpaceInfo, TagDef, TagDefWithId,
};
