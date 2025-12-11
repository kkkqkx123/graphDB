//! 算法相关的计划节点模块
//! 包含路径查找、搜索算法等高级算法的计划节点

mod path_algorithms;
mod index_scan;

// 重新导出算法节点类型
pub use path_algorithms::{ShortestPath, BFSShortest, AllPaths, MultiShortestPath};
pub use index_scan::{IndexScan, FulltextIndexScan};