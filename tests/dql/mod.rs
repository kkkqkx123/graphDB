//! Data Query Language (DQL) Integration Tests
//!
//! Test coverage:
//! - GO - Graph traversal
//! - MATCH - Pattern matching
//! - FETCH - Property fetching
//! - LOOKUP - Index-based lookup
//! - Aggregation - GROUP BY, ORDER BY, LIMIT
//! - Subquery - WITH, UNWIND
//! - FIND PATH - Path finding
//! - SUBGRAPH - Subgraph retrieval

mod common;
mod go;
mod match_query;
mod fetch;
mod lookup;
mod aggregation;
mod subquery;
mod find_path;
mod subgraph;
