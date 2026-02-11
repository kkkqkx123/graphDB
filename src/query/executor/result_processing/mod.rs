//! 结果处理执行器模块
//!
//! 包含所有与结果处理相关的执行器，这些执行器对查询结果进行最终的处理和优化
//!
//! 模块组织：
//! - `projection` - 列投影（SELECT 列）
//! - `sort` - 排序（ORDER BY）
//! - `limit` - 结果限制（LIMIT/OFFSET）
//! - `aggregation` - 聚合函数（GROUP BY）
//! - `dedup` - 去重（DISTINCT）
//! - `filter` - 结果过滤（HAVING）
//! - `sample` - 采样（SAMPLING）
//! - `topn` - 排序优化（TOP N）
//! - `transformations` - 数据转换（Assign、Unwind、AppendVertices等）

// 聚合数据状态（参考 nebula-graph AggData）
pub mod agg_data;
pub use agg_data::AggData;

// 聚合函数管理器（参考 nebula-graph AggFunctionManager）
pub mod agg_function_manager;
pub use agg_function_manager::AggFunctionManager;

// 列投影
pub mod projection;
pub use projection::{ProjectExecutor, ProjectionColumn};

// 排序执行器
pub mod sort;
pub use sort::{SortConfig, SortExecutor, SortKey, SortOrder};

// 限制执行器
pub mod limit;
pub use limit::LimitExecutor;

// 聚合执行器
pub mod aggregation;
pub use aggregation::{
    AggregateExecutor, AggregateFunctionSpec, GroupAggregateState, GroupByExecutor,
    HavingExecutor,
};

pub use crate::core::types::operators::AggregateFunction;

// 去重执行器
pub mod dedup;
pub use dedup::{DedupExecutor, DedupStrategy};

// 过滤执行器
pub mod filter;
pub use filter::FilterExecutor;

// 采样执行器
pub mod sample;
pub use sample::{SampleExecutor, SampleMethod};

// TOP N 优化
pub mod topn;
pub use topn::TopNExecutor;

// 数据转换操作
// 这些执行器处理数据转换操作，包括：
// - Assign（变量赋值）
// - Unwind（列表展开）
// - AppendVertices（追加顶点）
// - PatternApply（模式匹配）
// - RollUpApply（聚合操作）
pub mod transformations;
pub use transformations::{
    AppendVerticesExecutor, AssignExecutor, PatternApplyExecutor, RollUpApplyExecutor, UnwindExecutor,
};

// 统一的执行器接口
pub mod traits;
pub use traits::{ResultProcessor, ResultProcessorContext};
