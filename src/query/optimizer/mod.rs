//! 查询优化器模块
//!
//! 提供查询优化功能，包括统计信息管理、代价计算和优化策略
//!
//! ## 模块结构
//!
//! - `stats` - 统计信息模块，管理标签、边类型和属性的统计信息
//! - `cost` - 代价计算模块，计算查询操作的代价
//! - `strategy` - 优化策略模块，提供遍历起点选择和索引选择
//!
//! ## 使用示例
//!
//! ```rust
//! use graphdb::query::optimizer::StatisticsManager;
//! use graphdb::query::optimizer::CostCalculator;
//! use std::sync::Arc;
//!
//! // 创建统计信息管理器
//! let stats_manager = Arc::new(StatisticsManager::new());
//!
//! // 创建代价计算器
//! let cost_calculator = CostCalculator::new(stats_manager);
//!
//! // 计算扫描代价
//! let scan_cost = cost_calculator.calculate_scan_cost("Person");
//! ```

pub mod stats;
pub mod cost;
pub mod strategy;

// 重新导出主要类型
pub use stats::{
    StatisticsManager,
    StatisticsCollector,
    StatisticsCollection,
    TagStatistics,
    EdgeTypeStatistics,
    PropertyStatistics,
};

pub use cost::{
    CostCalculator,
    SelectivityEstimator,
};

pub use strategy::{
    TraversalStartSelector,
    CandidateStart,
    SelectionReason,
    IndexSelector,
    IndexSelection,
    PropertyPredicate,
    PredicateOperator,
};
