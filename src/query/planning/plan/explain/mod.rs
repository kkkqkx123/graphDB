//! Query Plan Explanation Module
//!
//! This module provides functionality to describe and format execution plans
//! for human-readable output (EXPLAIN command).

pub mod description;
pub mod describe_visitor;

pub use description::{
    Pair, PlanDescription, PlanNodeBranchInfo, PlanNodeDescription, ProfilingStats,
};
pub use describe_visitor::DescribeVisitor;
