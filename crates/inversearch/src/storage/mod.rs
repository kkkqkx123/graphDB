//! 存储接口模块
//!
//! 提供持久化存储的抽象接口和实现
//!
//! ## 模块结构
//!
//! ```text
//! storage/
//! ├── common/         # 公共组件（类型、trait、工具函数）
//! │   ├── types.rs    # 共享类型定义
//! │   ├── trait.rs    # 存储接口 trait
//! │   ├── io.rs       # 文件 I/O 操作
//! │   ├── compression.rs  # 压缩/解压缩
//! │   └── metrics.rs  # 性能指标
//! ├── base.rs         # 存储基类
//! ├── utils.rs        # 工具函数
//! ├── memory.rs       # 内存存储实现
//! ├── file.rs         # 文件存储实现
//! ├── cached.rs       # 缓存存储实现
//! ├── redis.rs        # Redis 存储实现
//! └── wal/            # WAL 模块
//!     ├── mod.rs
//!     ├── log.rs      # 日志管理
//!     ├── snapshot.rs # 快照管理
//!     └── cleanup.rs  # 清理任务
//! ```
//!
//! ## 条件编译特性
//!
//! - `store-memory`: 内存存储
//! - `store-file`: 文件存储（默认启用）
//! - `store-redis`: Redis 存储
//! - `store-wal`: WAL 预写日志存储
//! - `store-cached`: 缓存存储（内存+文件，默认启用）

// 公共组件 - 所有存储实现共享
pub mod common;

// 存储基类
pub mod base;

// 工具函数
pub mod utils;

// 条件编译的存储实现
#[cfg(feature = "store-memory")]
pub mod memory;

#[cfg(feature = "store-file")]
pub mod file;

#[cfg(feature = "store-redis")]
pub mod redis;

#[cfg(feature = "store-wal")]
pub mod wal;

#[cfg(feature = "store-wal")]
pub mod wal_storage;

#[cfg(feature = "store-cached")]
pub mod cached;

// 重新导出常用类型和 trait，方便使用
pub use common::{
    StorageInfo,
    FileStorageData,
    StorageMetrics,
    StorageInterface,
    io::{save_to_file, load_from_file, atomic_write, get_file_size, remove_file_safe},
    compression::{compress_data, decompress_data},
    metrics::{MetricsCollector, OperationTimer},
};
