//! 优化策略模块
//!
//! 提供查询优化策略，包括遍历起点选择、索引选择和基于代价的优化策略
//!
//! ## 模块结构
//!
//! - `traversal_start` - 遍历起点选择器
//! - `index` - 索引选择器
//! - `aggregate_strategy` - 聚合策略选择器
//! - `join_order` - 连接顺序优化器
//! - `traversal_direction` - 图遍历方向优化器

pub mod traversal_start;
pub mod index;
pub mod aggregate_strategy;
pub mod join_order;
pub mod traversal_direction;

pub use traversal_start::{
    TraversalStartSelector,
    CandidateStart,
    SelectionReason as TraversalSelectionReason,
};

pub use index::{
    IndexSelector,
    IndexSelection,
    PropertyPredicate,
    PredicateOperator,
};

pub use aggregate_strategy::{
    AggregateStrategySelector,
    AggregateStrategy,
    AggregateStrategyDecision,
    AggregateContext,
    SelectionReason as AggregateSelectionReason,
};

pub use join_order::{
    JoinOrderOptimizer,
    JoinOrderResult,
    TableInfo,
    JoinCondition,
    OptimizationMethod,
};

pub use traversal_direction::{
    TraversalDirectionOptimizer,
    TraversalDirection,
    TraversalDirectionDecision,
    DirectionContext,
    DirectionSelectionReason,
    DegreeInfo,
};
