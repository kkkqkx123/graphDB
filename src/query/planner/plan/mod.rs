//! 执行计划节点相关定义和结构
//! 包含PlanNode特征、各种计划节点类型和执行计划结构

pub mod admin_nodes;
pub mod aggregation_ops;
pub mod algo_nodes;
pub mod algorithms;
pub mod common;
pub mod core;
pub mod execution_plan;
pub mod join_ops;
pub mod maintain_nodes;
pub mod management;
pub mod mutate_nodes;
pub mod operations;
pub mod plan_node;
pub mod plan_node_visitor;



// 重新导出主要的类型
pub use core::{
    BinaryInputNode, DefaultPlanNodeVisitor, PlanNode, PlanNodeKind, PlanNodeVisitError,
    PlanNodeVisitor, SingleDependencyNode, SingleInputNode, VariableDependencyNode,
};
pub use execution_plan::{ExecutionPlan, SubPlan};

// 从新的模块结构重新导出节点类型
// 注意：只导出 operations 中的新实现，避免旧模块导出的重复定义
pub use admin_nodes::*;
pub use aggregation_ops::*;
pub use algo_nodes::*;
pub use algorithms::*;
pub use common::*;
pub use join_ops::*;
pub use maintain_nodes::*;
pub use management::*;
pub use mutate_nodes::*;
pub use operations::*;
pub use plan_node::*;
pub use plan_node_visitor::*;
