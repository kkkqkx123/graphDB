//! 统计信息模块
//!
//! 提供查询优化器所需的统计信息管理和收集功能
//!
//! ## 模块结构
//!
//! - `manager` - 统计信息管理器，统一管理所有统计信息
//! - `collector` - 统计信息收集器，从存储引擎收集统计信息
//! - `tag` - 标签统计信息
//! - `edge` - 边类型统计信息
//! - `property` - 属性统计信息

pub mod manager;
pub mod collector;
pub mod tag;
pub mod edge;
pub mod property;

pub use manager::StatisticsManager;
pub use collector::{StatisticsCollector, StatisticsCollection};
pub use tag::TagStatistics;
pub use edge::EdgeTypeStatistics;
pub use property::PropertyStatistics;
