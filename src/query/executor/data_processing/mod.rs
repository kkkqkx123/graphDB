//! Data Processing Executor Module
//!
//! This includes all executors related to data processing, which are responsible for the conversion and manipulation of intermediate results.
//!
//! Module organization:
//! `graph_traversal` – Related to graph traversal (functions such as Expand, Traverse, ShortestPath, etc.)
//! `set_operations` – Set operations (Union, Intersect, Difference)
//! `join` – Operations for connecting data (InnerJoin, LeftJoin, FullOuterJoin)
//! “Materialize” refers to a process in computing or data processing where data is transformed from a virtual or abstract form into a physical, tangible form that can be stored, manipulated, or used by systems. This can involve converting data structures, algorithms, or calculations from a theoretical or conceptual state into a practical, executable format. The goal of materialization is often to improve the efficiency of data retrieval, processing, or analysis by making the data more accessible and readily available for use.
//!
//! The RightJoin has been removed; the order of the tables can be swapped using a LeftJoin to achieve the same functionality.

// Graph Traversal Executor
pub mod graph_traversal;
pub use graph_traversal::{
    ExpandAllExecutor, ExpandExecutor, ShortestPathAlgorithm, ShortestPathExecutor,
    TraverseExecutor,
};

// Set operation executor
pub mod set_operations;
pub use set_operations::{
    IntersectExecutor, MinusExecutor, SetExecutor, UnionAllExecutor, UnionExecutor,
};

// JOIN Executor
pub mod join;
pub use join::{
    CrossJoinExecutor, FullOuterJoinExecutor, HashInnerJoinExecutor, HashLeftJoinExecutor,
    InnerJoinConfig, InnerJoinExecutor, JoinConfig, JoinType, LeftJoinConfig, LeftJoinExecutor,
};

// Materialized Executor
pub mod materialize;
pub use materialize::{MaterializeExecutor, MaterializeState};
