//! 管理器接口模块
//! 
//! 包含所有管理器接口的定义，用于管理数据库的各种资源

pub mod schema_manager;
pub mod index_manager;
pub mod storage_client;
pub mod meta_client;

// 重新导出所有公共类型和trait
pub use schema_manager::{Schema, SchemaManager, CharsetInfo};
pub use index_manager::{Index, IndexManager};
pub use storage_client::{StorageClient, StorageOperation, StorageResponse};
pub use meta_client::{MetaClient, ClusterInfo, SpaceInfo};