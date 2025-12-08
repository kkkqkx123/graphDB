//! 执行计划节点相关定义和结构
//! 包含PlanNode特征、各种计划节点类型和执行计划结构

pub mod plan_node;
pub mod execution_plan;
pub mod plan_node_visitor;
pub mod query_nodes;
pub mod logic_nodes;
pub mod admin_nodes;
pub mod algo_nodes;
pub mod mutate_nodes;
pub mod maintain_nodes;
pub mod scan_nodes;

// 重新导出主要的类型
pub use plan_node::{PlanNode, PlanNodeKind, SingleDependencyNode, SingleInputNode, BinaryInputNode, VariableDependencyNode};
pub use execution_plan::{ExecutionPlan, SubPlan};
pub use plan_node_visitor::{PlanNodeVisitor, PlanNodeVisitError};
