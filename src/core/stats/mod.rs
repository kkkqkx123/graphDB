//! 统计信息模块
//!
//! 提供查询指标、查询画像和错误统计功能。
//!
//! ## 模块结构
//!
//! - `metrics`: 轻量级查询指标（用于返回给客户端）
//! - `profile`: 详细查询画像（用于监控和分析）
//! - `error_stats`: 错误统计
//! - `manager`: 统一管理器
//!
//! ## QueryMetrics vs QueryProfile
//!
//! ### QueryMetrics（轻量级）
//! - 用途：返回给客户端的查询指标
//! - 精度：微秒级（us）
//! - 内容：执行时间、节点数、结果数
//! - 使用场景：API响应、客户端展示
//!
//! ### QueryProfile（详细）
//! - 用途：内部分析和监控
//! - 精度：毫秒级（ms）
//! - 内容：执行时间、执行器统计、错误信息、慢查询日志
//! - 使用场景：性能分析、问题诊断、监控告警

pub mod error_stats;
pub mod manager;
pub mod metrics;
pub mod profile;

// 重新导出常用类型
pub use error_stats::{ErrorInfo, ErrorStatsManager, ErrorSummary, ErrorType, QueryPhase};
pub use manager::{MetricType, MetricValue, StatsManager};
pub use metrics::QueryMetrics;
pub use profile::{ExecutorStat, QueryProfile, QueryStatus, StageMetrics};
