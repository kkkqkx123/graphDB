//! 存储接口模块
//!
//! 提供 BM25 存储的抽象接口和实现
//!
//! ## 模块结构
//!
//! ```text
//! storage/
//! ├── common/
//! │   ├── types.rs         # 共享类型定义
//! │   └── trait.rs         # 存储接口 trait
//! ├── factory.rs           # 存储工厂
//! ├── redis.rs             # Redis 存储实现（可选）
//! └── tantivy.rs           # Tantivy 本地存储（默认）
//! ```
//!
//! ## 条件编译特性
//!
//! - `storage-redis`: Redis 存储
//! - `storage-tantivy`: Tantivy 本地文件存储（默认启用）

pub mod common;

#[cfg(feature = "storage-tantivy")]
pub mod tantivy;

#[cfg(feature = "storage-redis")]
pub mod redis;

pub mod factory;
pub mod manager;

pub use common::{
    r#trait::StorageInterface,
    types::{Bm25Stats, StorageInfo},
};

#[cfg(feature = "storage-tantivy")]
pub use tantivy::TantivyStorage;

#[cfg(feature = "storage-redis")]
pub use redis::RedisStorage;

pub use factory::StorageFactory;
pub use manager::{DefaultStorage, MutableStorageManager, StorageManager, StorageManagerBuilder};
