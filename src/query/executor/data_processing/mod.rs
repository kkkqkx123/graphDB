//! 数据处理执行器模块
//!
//! 包含所有与数据处理相关的执行器，这些执行器处理中间结果的转换和处理
//!
//! 模块组织：
//! - `filter` - 条件过滤（WHERE 子句）
//! - `graph_traversal` - 图遍历相关（Expand、Traverse、ShortestPath 等）
//! - `set_operations` - 集合运算（Union、Intersect、Minus）
//! - `join` - 连接操作（InnerJoin、LeftJoin）
//! - `transformations` - 数据转换（Assign、Unwind、PatternApply）
//! - `loops` - 循环控制

// 条件过滤
pub mod filter;
pub use filter::FilterExecutor;

// 图遍历执行器
pub mod graph_traversal;
pub use graph_traversal::{
    ExpandExecutor, ExpandAllExecutor, TraverseExecutor,
    ShortestPathExecutor, ShortestPathAlgorithm,
};

// 集合运算执行器
pub mod set_operations;

// JOIN 执行器
pub mod join;

// 数据转换执行器
pub mod transformations;
pub use transformations::{
    AssignExecutor, UnwindExecutor, AppendVerticesExecutor,
    PatternApplyExecutor, RollUpApplyExecutor, PatternType, EdgeDirection
};

// 循环控制
pub mod loops;
