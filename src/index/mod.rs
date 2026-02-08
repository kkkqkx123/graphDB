//! 索引系统模块
//!
//! 提供完整的索引功能支持：
//! - 二进制键编码
//! - 内存缓存
//! - 统一索引类型定义
//! - 全文索引功能
//! - 索引错误处理
//!
//! 索引存储实现已迁移至 src/storage/index/ 目录

pub mod binary;
pub mod cache;
pub mod config;
pub mod error;
pub mod fulltext;
pub mod stats;
pub mod types;

pub use binary::*;
pub use cache::*;
pub use config::*;
pub use error::*;
pub use fulltext::*;
pub use stats::*;
pub use types::*;
