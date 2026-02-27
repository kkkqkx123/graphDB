//! 优化策略模块
//!
//! 提供查询优化策略，包括遍历起点选择和索引选择
//!
//! ## 模块结构
//!
//! - `traversal_start` - 遍历起点选择器
//! - `index` - 索引选择器

pub mod traversal_start;
pub mod index;

pub use traversal_start::{
    TraversalStartSelector,
    CandidateStart,
    SelectionReason,
};

pub use index::{
    IndexSelector,
    IndexSelection,
    PropertyPredicate,
    PredicateOperator,
};
