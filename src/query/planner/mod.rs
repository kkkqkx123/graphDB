//! Planner module for generating execution plans from AST
//! Contains the Planner trait, ExecutionPlan structure, and various specific planners

// 核心模块
pub mod plan;
pub mod planner;

// 新的模块化结构
pub mod match_planning;
pub mod ngql;

// 重新导出主要的类型
pub use match_planning::MatchPlanner;
pub use ngql::{GoPlanner, LookupPlanner, PathPlanner, SubgraphPlanner};
pub use plan::execution_plan::{ExecutionPlan, SubPlan};
pub use planner::{Planner, PlannerError, PlannerRegistry, SequentialPlanner};
