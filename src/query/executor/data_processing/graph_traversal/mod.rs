//! 图遍历执行器模块
//!
//! 包含所有与图遍历相关的执行器，包括：
//! - 单步扩展（Expand）
//! - 全路径扩展（ExpandAll）
//! - 完整遍历（Traverse）
//! - 最短路径（ShortestPath）
//! - 所有路径（AllPaths）- 新增
//! - 多最短路径（MultiShortestPath）- 新增
//! - 子图提取（Subgraph）

// 算法模块 - 解耦算法实现与执行流程
pub mod algorithms;
pub mod all_paths;
pub mod expand;
pub mod expand_all;
pub mod factory;
pub mod impls;
pub mod shortest_path;
pub mod tests;
pub mod traits;
pub mod traverse;
pub mod traversal_utils;

// 重新导出主要类型
pub use expand::ExpandExecutor;
pub use expand_all::ExpandAllExecutor;
pub use all_paths::{
    AllPathsExecutor,
};
pub use shortest_path::ShortestPathExecutor;
pub use traverse::TraverseExecutor;

// 导出算法模块
pub use algorithms::{
    AStar, AlgorithmContext, AlgorithmStats, BidirectionalBFS, Dijkstra,
    PathFindingAlgorithm, ShortestPathAlgorithm, ShortestPathAlgorithmType,
    TraversalAlgorithm,
};

// 导出通用特征和工厂
pub use factory::GraphTraversalExecutorFactory;
pub use traits::GraphTraversalExecutor;
