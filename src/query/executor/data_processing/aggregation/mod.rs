//! 聚合操作执行器模块
//!
//! 包含所有聚合操作相关的执行器，包括：
//! - GroupBy（分组聚合）
//! - Aggregate（整体聚合）
//! - Having（分组后过滤）

pub mod group_by;
pub mod aggregate;
pub mod having;

pub use group_by::{GroupByExecutor, AggregateState as GroupAggregateState};
pub use aggregate::{AggregateExecutor, AggregateState};
pub use having::HavingExecutor;