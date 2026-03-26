//! Storage Iterator Module - provides the underlying iterative interface to the storage engine.
//!
//! Offer:
//! - StorageIterator: Storage Engine Iterator Interface
//! - VecPairIterator: Simple KV Pair Iterator
//! - Predicate: Predicate down-propagation optimization
//! - Row: Row data type alias (Vec<Value>)
//!
//! Attention:
//! - Use the core::result::iterator module for query result iterators.
//! - Combined iterator operations (filter, map, take, skip) should use the Rust standard iterator
//! - or the implementation in the core::result::combinators module

pub mod predicate;
pub mod storage_iter;

pub use predicate::{
    CompareOp, CompoundPredicate, Expression, LogicalOp, PredicateEnum, PredicateOptimizer,
    PushdownResult, SimplePredicate,
};
pub use storage_iter::{StorageIterator, VecPairIterator};

use crate::core::Value;

/// Row Definition - Vec<Value> represents a row of data.
///
/// This is a generic type alias that is used to represent a row of data in the query results
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
