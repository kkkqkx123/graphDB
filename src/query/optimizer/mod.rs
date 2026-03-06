//! 查询优化器模块
//!
//! 提供查询优化功能，包括统计信息管理、代价计算和优化策略
//!
//! ## 模块结构
//!
//! - `engine` - 优化器引擎，全局唯一的优化器实例，整合所有优化组件
//! - `stats` - 统计信息模块，管理标签、边类型和属性的统计信息
//! - `cost` - 代价计算模块，计算查询操作的代价
//! - `analysis` - 计划分析模块，提供引用计数和表达式分析
//! - `strategy` - 优化策略模块，提供遍历起点选择和索引选择
//! - `decision` - 优化决策模块，提供基于决策的缓存机制
//!
//! ## 使用示例
//!
//! ```rust
//! use graphdb::query::optimizer::OptimizerEngine;
//!
//! // 创建优化器引擎（全局实例）
//! let optimizer = OptimizerEngine::default();
//!
//! // 计算优化决策
//! let decision = optimizer.compute_decision(&stmt, sentence_kind);
//! ```

pub mod analysis;
pub mod cost;
pub mod decision;
pub mod engine;
pub mod stats;
pub mod strategy;

// 重新导出主要类型
pub use engine::OptimizerEngine;

pub use stats::{
    EdgeTypeStatistics, PropertyStatistics, StatisticsCollection, StatisticsCollector,
    StatisticsManager, TagStatistics,
};

pub use crate::core::error::optimize::CostError;
pub use cost::{CostAssigner, CostCalculator, CostModelConfig, SelectivityEstimator};

// 重新导出分析模块类型
pub use analysis::{
    AnalysisOptions, ExpressionAnalysis, ExpressionAnalyzer, ReferenceCountAnalysis,
    ReferenceCountAnalyzer,
};

pub use strategy::{
    AggregateContext, AggregateSelectionReason, AggregateStrategy, AggregateStrategyDecision,
    AggregateStrategySelector, CandidateStart, DegreeInfo, DirectionContext,
    DirectionSelectionReason, IndexSelection, IndexSelector, JoinCondition, JoinOrderOptimizer,
    JoinOrderResult, KeepReason, MaterializationDecision, MaterializationOptimizer,
    MaterializeReason, NoMaterializeReason, OptimizationMethod, PredicateOperator,
    PropertyPredicate, SortContext, SortEliminationDecision, SortEliminationOptimizer,
    SortKeepReason, SubqueryUnnestingOptimizer, TableInfo, TopNConversionReason,
    TraversalDirection, TraversalDirectionDecision, TraversalDirectionOptimizer,
    TraversalSelectionReason, TraversalStartSelector, UnnestDecision, UnnestReason,
};

pub use decision::{
    AccessPath, EntityIndexChoice, EntityType, IndexChoice, IndexSelectionDecision, JoinAlgorithm,
    JoinOrderDecision, OptimizationDecision, RewriteRuleId, TraversalStartDecision,
};
