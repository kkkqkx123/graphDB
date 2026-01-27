//! 语句级planner
//!
//! 包含所有图数据库语句的planner实现
//! 支持Cypher和NGQL的所有语句类型
//!
//! ## 架构说明
//!
//! 采用三层架构设计：
//! - Planner trait：基础规划器接口
//! - StatementPlanner trait：语句级规划器，处理完整语句
//! - ClausePlanner trait：子句级规划器，处理单个子句

pub mod core;
pub mod clauses;
pub mod paths;
pub mod seeks;

pub mod statement_planner;
pub mod match_statement_planner;

pub mod fetch_edges_planner;
pub mod fetch_vertices_planner;
pub mod go_planner;
pub mod lookup_planner;
pub mod maintain_planner;
pub mod path_planner;
pub mod subgraph_planner;
pub mod match_planner;

// 重新导出核心模块的主要类型
pub use core::{
    ClauseType, ContextPropagator, CypherClausePlanner, DataFlowManager, DataFlowNode,
    FlowDirection, PlanningContext, QueryInfo, VariableInfo,
};
pub use core::MatchClausePlanner;

// 重新导出语句规划器模块
pub use statement_planner::{
    ClausePlanner, PlannerRegistry, PlanningPhase, StatementPlanningContext, StatementPlanner,
};
pub use match_statement_planner::MatchStatementPlanner;



