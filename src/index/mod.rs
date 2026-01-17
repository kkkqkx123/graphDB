//! 索引系统模块
//!
//! 提供完整的索引功能支持：
//! - 二进制键编码
//! - 并发安全的索引存储
//! - 前缀查询和范围查询
//! - 统一索引类型定义
//! - 索引错误处理

pub mod binary;
pub mod error;
pub mod storage;
pub mod stats;
pub mod types;

pub use binary::*;
pub use error::*;
pub use storage::*;
pub use stats::*;
pub use types::*;
