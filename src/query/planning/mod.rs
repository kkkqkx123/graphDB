//! Planner module for generating execution plans from AST
//! Contains the Planner trait, ExecutionPlan structure, and various specific planners

// Core modules
pub mod connector;
pub mod plan;
pub mod planner;
pub mod template_extractor;

// Modules organized by function
pub mod statements;

// Plan to rewrite the module (heuristic optimization rules)
pub mod rewrite;

// Re-export the main types.
pub use connector::SegmentsConnector;
pub use plan::execution_plan::{ExecutionPlan, SubPlan};
pub use planner::{Planner, PlannerConfig, PlannerError};
pub use template_extractor::{ParameterizedResult, ParameterizingTransformer, TemplateExtractor};

// Re-export the planned cache types from the cache module (for backward compatibility)
pub use crate::query::cache::{
    CachedPlan, ParamPosition, ParameterizedQueryHandler, PlanCacheConfig, PlanCacheKey,
    PlanCacheStats, QueryPlanCache,
};

// Re-export the JoinType from the core module.
pub use crate::core::types::JoinType;
pub use statements::MatchStatementPlanner;

// Export related to static registration
pub use planner::PlannerEnum;

use std::sync::atomic::{AtomicI64, Ordering};

pub struct PlanIdGenerator {
    counter: AtomicI64,
}

impl PlanIdGenerator {
    pub fn instance() -> &'static Self {
        static INSTANCE: PlanIdGenerator = PlanIdGenerator {
            counter: AtomicI64::new(0),
        };
        &INSTANCE
    }

    pub fn next_id(&self) -> i64 {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }
}
