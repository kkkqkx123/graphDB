//! 算法模块
//!
//! 包含各种算法实现，按类型分组

pub mod graph;
pub mod search;
pub mod sorting;
pub mod string;

// 重新导出所有算法结构体，保持向后兼容性
pub use graph::GraphAlgorithms;
pub use search::SearchAlgorithms;
pub use sorting::SortingAlgorithms;
pub use string::StringAlgorithms;
