//! 算法相关的计划节点模块
//! 包含路径查找、搜索算法等高级算法的计划节点

pub mod index_scan;
pub mod path_algorithms;

// 重新导出算法节点类型
pub use index_scan::{FulltextIndexScan, IndexLimit, IndexScan};
pub use path_algorithms::{AllPaths, BFSShortest, MultiShortestPath, ShortestPath};
