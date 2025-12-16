//! 执行计划节点相关定义和结构
//! 包含PlanNode特征、各种计划节点类型和执行计划结构

pub mod algorithms;
pub mod common;
pub mod core;
pub mod execution_plan;
pub mod management;
pub mod operations;
pub mod scan_nodes;

// 重新导出主要的类型
pub use core::{
    BinaryInputNode, DefaultPlanNodeVisitor, PlanNode, PlanNodeKind, PlanNodeVisitError,
    PlanNodeVisitor, SingleDependencyNode, SingleInputNode, VariableDependencyNode,
};
pub use execution_plan::{ExecutionPlan, SubPlan};

// 从新的模块结构重新导出节点类型
pub use algorithms::*;
pub use common::*;
pub use management::*;
pub use operations::*;
pub use scan_nodes::{FulltextIndexScan, IndexScan};
