//! Storage Iterator Module - provides iterative interfaces for the storage engine.
//!
//! ## Components
//!
//! ### Edge Iterators
//! - `EdgeScanIterator`: Lazy iterator over all edges in an EdgeTable
//! - `EdgeRangeIterator`: Iterator over edges of specified vertices
//! - `EdgeFilterIterator`: Iterator with predicate pushdown support
//!
//! ### Vertex Iterators
//! - `VertexScanIterator`: Iterator over all vertices in a PropertyGraph
//! - `VertexRangeIterator`: Iterator over a range of vertices
//! - `VertexFilterIterator`: Iterator with predicate pushdown support
//!
//! ### Index Iterators
//! - `IndexScanIterator`: Iterator using secondary indexes for efficient retrieval
//! - `IndexScanConfig`: Configuration for index scan operations
//!
//! ### Predicate System
//! - `PredicateEnum`: Static predicate type (avoids dynamic dispatch)
//! - `SimplePredicate`: Single condition predicate
//! - `CompoundPredicate`: Combined predicates (AND/OR/NOT)
//! - `PredicateOptimizer`: Predicate pushdown optimizer
//!
//! ### Utilities
//! - `IterStats`: Iterator statistics (records to metrics crate)
//! - `IterConfig`: Iterator configuration
//! - `IterError`: Iterator error types
//!
//! ## Usage Notes
//!
//! - Use the core::result::iterator module for query result iterators
//! - Combined iterator operations (filter, map, take, skip) should use Rust standard iterators
//!   or the implementation in the core::result::combinators module

pub mod edge_iter;
pub mod index_scan_iter;
pub mod predicate;
pub mod storage_iter;
pub mod vertex_iter;

pub use edge_iter::{EdgeFilterIterator, EdgeRangeIterator, EdgeScanIterator, EdgeVertexScanIterator};
pub use index_scan_iter::{IndexScanConfig, IndexScanIterator};
pub use predicate::{
    CompareOp, CompoundPredicate, Expression, LogicalOp, PredicateEnum, PredicateOptimizer,
    PushdownResult, SimplePredicate,
};
pub use storage_iter::{IterConfig, IterError};
pub use vertex_iter::{VertexFilterIterator, VertexRangeIterator, VertexScanIterator, VertexTableScanIterator};

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
