//! 执行计划节点相关定义和结构
//! 包含PlanNode特征、各种计划节点类型和执行计划结构

pub mod algorithms;
pub mod common;
pub mod core;
pub mod execution_plan;
pub mod management;
pub mod operations;
// plan_node 模块已被移除，使用新的节点体系
pub mod plan_node_visitor;
pub mod utils;

// 重新导出主要的类型
pub use core::{
    DefaultPlanNodeVisitor, PlanNode, PlanNodeKind, PlanNodeVisitError, PlanNodeVisitor,
};
pub use execution_plan::{ExecutionPlan, SubPlan};
// 旧的节点类型已被移除，使用新的节点体系

// 从新的模块结构重新导出节点类型
// 注意：只导出 operations 和 management 中的新实现，避免旧模块导出的重复定义
pub use algorithms::*;
pub use common::*;
pub use management::*;
pub use operations::*;
pub use utils::*;
