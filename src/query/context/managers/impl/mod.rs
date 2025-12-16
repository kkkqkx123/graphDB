//! 管理器实现模块
//!
//! 包含所有管理器接口的具体实现

pub mod schema_manager_impl;
pub mod index_manager_impl;
pub mod storage_client_impl;
pub mod meta_client_impl;

// 重新导出所有公共类型
pub use schema_manager_impl::MemorySchemaManager;
pub use index_manager_impl::MemoryIndexManager;
pub use storage_client_impl::MemoryStorageClient;
pub use meta_client_impl::MemoryMetaClient;