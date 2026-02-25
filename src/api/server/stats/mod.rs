//! 统计管理模块
//!
//! 提供查询统计和性能监控功能
//!
//! 注意：实际实现已移动到 core::stats，此模块仅用于向后兼容

pub use crate::core::{StatsManager, QueryMetrics, QueryProfile, MetricType, MetricValue, QueryPhase, ErrorType, ErrorInfo, ErrorSummary, QueryStatus};
