//! Planner module for generating execution plans from AST
//! Contains the Planner trait, ExecutionPlan structure, and various specific planners

// 核心模块
pub mod plan;
pub mod planner;

// NGQL特定的规划器（旧位置，兼容性）
pub mod go_planner;
pub mod lookup_planner;
pub mod path_planner;
pub mod subgraph_planner;

// 新的模块化结构
pub mod match_planning;
pub mod ngql;

// 重新导出主要的类型
pub use match_planning::MatchPlanner;
pub use ngql::{GoPlanner, LookupPlanner, PathPlanner, SubgraphPlanner};
pub use plan::{ExecutionPlan, PlanNode, PlanNodeKind, PlanNodeVisitor, SubPlan};
pub use planner::{Planner, PlannerError, PlannersRegistry, SequentialPlanner};
