//! 执行计划节点相关定义和结构
//! 包含PlanNode特征、各种计划节点类型和执行计划结构

pub mod plan_node;
pub mod execution_plan;
pub mod core;
pub mod operations;
pub mod management;
pub mod algorithms;
pub mod common;

// 重新导出主要的类型
pub use plan_node::{PlanNode, SingleDependencyNode, SingleInputNode, BinaryInputNode, VariableDependencyNode};
pub use core::{PlanNodeKind, PlanNodeVisitor, PlanNodeVisitError, DefaultPlanNodeVisitor};
pub use execution_plan::{ExecutionPlan, SubPlan};

// 从新的模块结构重新导出节点类型
pub use operations::*;
pub use management::*;
pub use algorithms::*;
pub use common::*;
