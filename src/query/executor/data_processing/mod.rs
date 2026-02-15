//! 数据处理执行器模块
//!
//! 包含所有与数据处理相关的执行器，这些执行器处理中间结果的转换和处理
//!
//! 模块组织：
//! - `graph_traversal` - 图遍历相关（Expand、Traverse、ShortestPath 等）
//! - `set_operations` - 集合运算（Union、Intersect、Minus）
//! - `join` - 连接操作（InnerJoin、LeftJoin、FullOuterJoin）
//!
//! 注意：RightJoin 已被移除，可用 LeftJoin 交换表顺序实现

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
    LeftJoinExecutor,
};
