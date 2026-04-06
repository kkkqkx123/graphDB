//! 存储接口模块
//!
//! 提供持久化存储的抽象接口和实现
//!
//! ## 模块结构
//!
//! ```text
//! storage/
//! ├── common/              # 公共组件（类型、trait、工具函数）
//! │   ├── mod.rs
//! │   ├── base.rs          # 存储基类
//! │   ├── config.rs        # 存储配置
//! │   ├── compression.rs   # 压缩/解压缩
//! │   ├── error.rs         # 存储错误类型
//! │   ├── io.rs            # 文件 I/O 操作
//! │   ├── metrics.rs       # 性能指标
//! │   ├── trait.rs         # 存储接口 trait
//! │   ├── types.rs         # 共享类型定义
//! │   └── utils.rs         # 工具函数
//! ├── file.rs              # 文件存储实现
//! ├── redis.rs             # Redis 存储实现
//! ├── wal.rs               # WAL 预写日志
//! ├── memory.rs            # 内存存储实现（测试用）
//! ├── factory.rs           # 存储工厂
//! └── cold_warm_cache/     # 冷热缓存存储实现（默认）
//!     ├── mod.rs
//!     ├── config.rs
//!     ├── manager.rs
//!     ├── policy.rs
//!     ├── stats.rs
//!     └── background.rs
//! ```
//!
//! ## 条件编译特性
//!
//! - `store-cold-warm-cache`: 冷热缓存存储（默认启用）
//! - `store-file`: 文件存储
//! - `store-redis`: Redis 存储
//! - `store-wal`: WAL 预写日志存储

// 公共组件 - 所有存储实现共享
pub mod common;

// 条件编译的存储实现
#[cfg(feature = "store-file")]
pub mod file;

#[cfg(feature = "store-redis")]
pub mod redis;

#[cfg(feature = "store-wal")]
pub mod wal;

// 冷热缓存存储实现（默认）
pub mod cold_warm_cache;

// 测试用内存存储（仅用于测试）
pub mod memory;

// 存储工厂
pub mod factory;

// 存储管理器
pub mod manager;

// 持久化管理器
pub mod persistence;

// 重新导出常用类型和 trait，方便使用
pub use common::{
    compression::{compress_data, decompress_data},
    config::{StorageConfig, StorageType},
    error::{StorageError, StorageResult},
    io::{atomic_write, get_file_size, load_from_file, remove_file_safe, save_to_file},
    metrics::{MetricsCollector, OperationTimer},
    FileStorageData, StorageInfo, StorageInterface, StorageMetrics,
};

// 重新导出工厂
pub use factory::StorageFactory;

// 重新导出存储管理器
pub use manager::{DefaultStorage, StorageManager, StorageManagerBuilder};

// 重新导出持久化管理器
pub use persistence::{BackupInfo, IndexMetadata, IndexSnapshot, PersistenceManager};
