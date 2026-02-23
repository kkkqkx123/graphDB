//! 核心类型模块
//! 提供优化器所需的核心数据类型，包括代价模型、统计信息、选择性估计和配置

// 基础模块
pub mod cost;
pub mod config;

// 新实现的核心模块
pub mod statistics;
pub mod selectivity;
pub mod cost_model;

// 统计信息收集器（第二阶段实现）
pub mod analyze;

// 从旧模块重新导出（保持向后兼容性）
pub use cost::{Cost, Statistics as LegacyStatistics, TableStats, ColumnStats, PlanNodeProperties};
pub use config::{OptimizationConfig, OptimizationStats};

// 从 statistics 模块重新导出
pub use statistics::{
    TableStatistics, 
    ColumnStatistics, 
    IndexStatistics,
    GraphStatistics,
    StatisticsProvider,
    StatisticsProviderMut,
    MemoryStatisticsProvider,
};

// 从 selectivity 模块重新导出
pub use selectivity::{
    SelectivityEstimator, 
    RangeOp, 
    BooleanOp,
    JoinType as SelectivityJoinType,
};

// 从 cost_model 模块重新导出
pub use cost_model::{
    CostModelConfig, 
    CostContext,
};

// 从 analyze 模块重新导出
pub use analyze::{
    StatisticsCollector,
    AnalyzeConfig,
    AnalyzeError,
};

pub use crate::query::core::OptimizationPhase;
