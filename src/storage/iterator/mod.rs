//! 存储迭代器模块 - 提供存储引擎的底层迭代接口
//!
//! 提供：
//! - StorageIterator: 存储引擎迭代器接口
//! - VecPairIterator: 简单的 KV 对迭代器
//! - Predicate: 谓词下推优化
//! - Row: 行数据类型别名 (Vec<Value>)
//!
//! 注意：
//! - 查询结果迭代器请使用 core::result::iterator 模块
//! - 组合迭代器操作（filter、map、take、skip）应使用 Rust 标准迭代器
//!   或 core::result::combinators 模块中的实现

pub mod predicate;
pub mod storage_iter;

pub use predicate::{
    CompareOp, CompoundPredicate, Expression, LogicalOp, PredicateEnum, PredicateOptimizer,
    PushdownResult, SimplePredicate,
};
pub use storage_iter::{StorageIterator, VecPairIterator};

use crate::core::Value;

/// 行定义 - Vec<Value> 表示一行数据
///
/// 这是一个通用类型别名，用于表示查询结果中的一行数据
pub type Row = Vec<Value>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_type() {
        let row: Row = vec![Value::Int(1), Value::String("test".to_string())];
        assert_eq!(row.len(), 2);
    }
}
