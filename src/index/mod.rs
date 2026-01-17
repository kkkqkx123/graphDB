//! 索引系统模块
//!
//! 提供完整的索引功能支持：
//! - 二进制键编码
//! - 并发安全的索引存储
//! - 前缀查询和范围查询
//! - 索引统计信息收集

pub mod binary;
pub mod stats;
pub mod storage;

pub use binary::*;
pub use stats::*;
pub use storage::*;
