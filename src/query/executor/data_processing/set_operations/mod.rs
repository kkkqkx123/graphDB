//! 集合运算执行器模块
//!
//! 包含所有集合运算相关的执行器，包括：
//! - Union（并集，去重）
//! - UnionAll（并集，保留重复）
//! - Intersect（交集）
//! - Minus/Except（差集）

// 基础集合操作执行器
pub mod base;
pub use base::SetExecutor;

// Union操作（并集，去重）
pub mod union;
pub use union::UnionExecutor;

// UnionAll操作（并集，保留重复）
pub mod union_all;
pub use union_all::UnionAllExecutor;

// Intersect操作（交集）
pub mod intersect;
pub use intersect::IntersectExecutor;

// Minus操作（差集）
pub mod minus;
pub use minus::MinusExecutor;
