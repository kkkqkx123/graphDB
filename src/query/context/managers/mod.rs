//! 管理器接口模块
//!
//! 包含所有管理器接口的定义，用于管理数据库的各种资源

pub mod r#impl;
pub mod index_manager;
pub mod meta_client;
pub mod schema_manager;
pub mod storage_client;
pub mod retry;
pub mod transaction;

// 重新导出所有公共类型和trait
pub use index_manager::{Index, IndexManager, IndexStatus, IndexType};
pub use meta_client::{ClusterInfo, MetaClient, SpaceInfo};
pub use r#impl::*;
pub use schema_manager::{CharsetInfo, Schema, SchemaManager, FieldDef, TagDef, EdgeTypeDef, SchemaVersion, SchemaHistory};
pub use storage_client::{
    StorageClient, StorageOperation, StorageResponse,
    EdgeKey, NewTag, NewVertex, NewEdge, DelTags, UpdatedProp,
    ExecResponse, UpdateResponse
};
pub use crate::core::error::{ManagerError, ErrorCategory};
pub use retry::{RetryConfig, RetryStrategy, retry_with_backoff, retry_with_strategy};
pub use transaction::{
    TransactionManager, Transaction, TransactionId, TransactionState,
    TransactionOperation, IsolationLevel
};
