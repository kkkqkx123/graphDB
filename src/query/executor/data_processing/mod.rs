//! 数据处理执行器模块
//!
//! 包含所有与数据处理相关的执行器，这些执行器处理中间结果的转换和处理
//!
//! 注意：以下执行器已迁移到 result_processing 模块：
//! - filter, dedup, sample, aggregation, sort, pagination
//!
//! 模块组织：
//! - `graph_traversal` - 图遍历相关（Expand、Traverse、ShortestPath 等）
//! - `set_operations` - 集合运算（Union、Intersect、Minus）
//! - `join` - 连接操作（InnerJoin、LeftJoin）
//! - `transformations` - 数据转换（Assign、Unwind、PatternApply）
//! - `loops` - 循环控制

// 图遍历执行器
pub mod graph_traversal;
pub use graph_traversal::{
    ExpandAllExecutor, ExpandExecutor, ShortestPathAlgorithm, ShortestPathExecutor,
    TraverseExecutor,
};

// 集合运算执行器
pub mod set_operations;
pub use set_operations::{
    IntersectExecutor, MinusExecutor, SetExecutor, UnionAllExecutor, UnionExecutor,
};

// JOIN 执行器
pub mod join;
pub use join::{
    CrossJoinExecutor, FullOuterJoinExecutor, InnerJoinExecutor, JoinConfig, JoinType,
    LeftJoinExecutor, RightJoinExecutor,
};

// 数据转换执行器
pub mod transformations;
pub use transformations::{
    AppendVerticesExecutor, AssignExecutor, PatternApplyExecutor, PatternType, RollUpApplyExecutor,
    UnwindExecutor,
};

// 循环控制
pub mod loops;
pub use loops::{ForLoopExecutor, LoopExecutor, LoopState, WhileLoopExecutor};
