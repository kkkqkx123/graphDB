//! 存储模块公共组件
//!
//! 提供所有存储实现共享的类型、工具函数和 trait

pub mod types;
pub mod io;
pub mod compression;
pub mod metrics;
pub mod r#trait;

// 重新导出常用类型
pub use types::{StorageInfo, FileStorageData};
pub use io::{save_to_file, load_from_file, atomic_write};
pub use compression::{compress_data, decompress_data};
pub use metrics::{StorageMetrics, OperationTimer};
pub use r#trait::StorageInterface;
