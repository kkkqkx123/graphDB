//! 索引系统模块
//!
//! 提供完整的索引功能支持：
//! - 二进制键编码
//! - 并发安全的索引存储
//! - 内存缓存
//! - 统一存储接口（内存 + 持久化）
//! - 统一索引类型定义
//! - 索引错误处理

pub mod binary;
pub mod cache;
pub mod config;
pub mod error;
pub mod stats;
pub mod storage;
pub mod types;

pub use binary::*;
pub use cache::*;
pub use config::*;
pub use error::*;
pub use stats::*;
pub use storage::*;
pub use types::*;
