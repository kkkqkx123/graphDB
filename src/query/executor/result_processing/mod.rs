//! Result Processing Executor Module
//!
//! This includes all executors related to result processing, which perform the final processing and optimization of the query results.
//!
//! Module organization:
//! `projection` – Column projection (the columns that are selected using the SELECT statement).
//! `sort` – Sorting (ORDER BY)
//! `limit` – Restriction on the number of results (LIMIT/OFFSET)
//! `aggregation` – Aggregation functions (GROUP BY)
//! `dedup` – Removal of duplicates (i.e., ensuring that each value appears only once in the result set).
//! `filter` – Filtering of the results (HAVING clause)
//! “sample” refers to the process of collecting data or information from a larger set in order to represent it more accurately or to make predictions based on that subset.
//! `topn` – Optimization for sorting (displaying the top N items)
//! “Transformations” – Data transformations (such as Assign, Unwind, AppendVertices, etc.)

// Aggregated data status (refer to nebula-graph AggData)
pub mod agg_data;
pub use agg_data::AggData;

// Aggregation Function Manager (refer to nebula-graph AggFunctionManager)
pub mod agg_function_manager;
pub use agg_function_manager::AggFunctionManager;

// Column projection
pub mod projection;
pub use projection::{ProjectExecutor, ProjectionColumn};

// Sorting Executor
pub mod sort;
pub use sort::{SortConfig, SortExecutor, SortKey, SortOrder};

// Limit the execution of the actuator
pub mod limit;
pub use limit::LimitExecutor;

// Aggregated Executor
pub mod aggregation;
pub use aggregation::{
    AggregateExecutor, AggregateFunctionSpec, GroupAggregateState, GroupByExecutor, HavingExecutor,
};

pub use crate::core::types::operators::AggregateFunction;

// De-duplication executor
pub mod dedup;
pub use dedup::{DedupExecutor, DedupStrategy};

// Filter Executor
pub mod filter;
pub use filter::FilterExecutor;

// Sampling Executor
pub mod sample;
pub use sample::{SampleExecutor, SampleMethod};

// TOP N Optimization
pub mod topn;
pub use topn::TopNExecutor;

// Data conversion operations
// These actuators perform data conversion operations, including:
// Assign (variable assignment)
// “Unwind” (list expansion) – This refers to the process of expanding or displaying all the items in a list in detail. For example, if you have a list with only a few items shown initially, clicking on the “Unwind” button or option will show all the items in the list.
// AppendVertices (Adding Vertices)
// PatternApply (Pattern matching)
// RollUpApply (Aggregation Operation)
pub mod transformations;
pub use transformations::{
    AppendVerticesExecutor, AssignExecutor, PatternApplyExecutor, RollUpApplyExecutor,
    UnwindExecutor,
};
