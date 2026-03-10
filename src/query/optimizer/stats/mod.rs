//! 统计信息模块
//!
//! 提供查询优化器所需的统计信息管理和收集功能
//!
//! ## 模块结构
//!
//! - `manager` - 统计信息管理器，统一管理所有统计信息
//! - `tag` - 标签统计信息
//! - `edge` - 边类型统计信息
//! - `property` - 属性统计信息
//! - `histogram` - 直方图统计信息
//! - `feedback` - 运行时统计反馈模块

pub mod edge;
pub mod feedback;
pub mod histogram;
pub mod manager;
pub mod property;
pub mod tag;

// 从feedback模块重新导出主要类型
pub use feedback::{
    ExecutionFeedbackCollector, FeedbackDrivenSelectivity, OperatorFeedback,
    QueryExecutionFeedback, QueryFeedbackHistory, SelectivityFeedbackManager,
    AutoFeedbackConfig, AutoFeedbackTrigger,
    normalize_query, generate_query_fingerprint,
};
pub use edge::{EdgeTypeStatistics, HotVertexInfo, SkewnessLevel};
pub use histogram::{Histogram, HistogramBucket, RangeCondition};
pub use manager::StatisticsManager;
pub use property::PropertyStatistics;
pub use tag::TagStatistics;
