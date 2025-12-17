//! 核心规划器模块
//! 
//! 提供查询规划器的核心组件，包括：
//! - CypherClausePlanner trait：定义子句规划器的统一接口
//! - ClauseType enum：定义子句类型
//! - PlanningContext：管理规划过程中的上下文信息
//! - DataFlowValidator：验证数据流的正确性

pub mod match_planner;
pub mod cypher_clause_planner;
pub mod match_clause_planner;

// 重新导出新的接口
pub use cypher_clause_planner::{
    CypherClausePlanner, ClauseType, PlanningContext, FlowDirection,
    VariableRequirement, VariableProvider, VariableType, DataFlowValidator
};
pub use match_clause_planner::MatchClausePlanner;