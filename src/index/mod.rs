//! 索引核心模块
//!
//! 提供索引核心类型定义和工具：
//! - 二进制键编码
//! - 统一索引类型定义
//! - 索引错误处理
//! - 索引配置
//!
//! 运行时服务（缓存、统计、全文索引）已移至 service 子模块
//! 索引存储实现已迁移至 src/storage/index/ 目录

pub mod binary;
pub mod config;
pub mod error;
pub mod types;

pub use binary::*;
pub use config::*;
pub use error::*;
pub use types::*;
