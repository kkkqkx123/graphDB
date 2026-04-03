//! 存储模块公共组件
//!
//! 提供所有存储实现共享的类型、工具函数和 trait

pub mod compression;
pub mod io;
pub mod metrics;
pub mod r#trait;
pub mod types;

// 重新导出常用类型
pub use compression::{compress_data, decompress_data};
pub use io::{atomic_write, load_from_file, save_to_file};
pub use metrics::{OperationTimer, StorageMetrics};
pub use r#trait::StorageInterface;
pub use types::{FileStorageData, StorageInfo};
