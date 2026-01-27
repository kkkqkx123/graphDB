//! Planner module for generating execution plans from AST
//! Contains the Planner trait, ExecutionPlan structure, and various specific planners

// 核心模块
pub mod plan;
pub mod planner;
pub mod connector;

// 按功能组织的模块
pub mod statements;

// 重新导出主要的类型
pub use plan::execution_plan::{ExecutionPlan, SubPlan};
pub use planner::{
    ConfigurablePlannerRegistry, Planner, PlannerConfig, PlannerError, PlannerRegistry,
    PlanCache, PlanCacheKey, SequentialPlanner,
};
pub use connector::{JoinType, SegmentsConnector};
pub use statements::{
    MatchStatementPlanner,
};
