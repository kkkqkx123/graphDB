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
    AggregateExecutor, AggregateFunctionSpec, AggregateState, GroupAggregateState, GroupByExecutor,
    HavingExecutor,
};

// Re-export AggregateFunction directly from its source
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

// 统一的执行器接口
pub mod traits;
pub use traits::{ResultProcessor, ResultProcessorContext};
