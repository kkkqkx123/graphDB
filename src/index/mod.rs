//! 索引系统模块
//!
//! 提供完整的索引功能支持：
//! - 二进制键编码
//! - 并发安全的索引存储
//! - 前缀查询和范围查询
//! - 索引统计信息收集
//! - 统一索引类型定义
//! - 索引错误处理

pub mod binary;
pub mod error;
pub mod stats;
pub mod storage;
pub mod types;

pub use binary::*;
pub use error::*;
pub use stats::*;
pub use storage::*;
pub use types::*;
