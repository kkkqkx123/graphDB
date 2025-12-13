//! 计划节点核心模块
//! 包含PlanNode特征、节点类型枚举、访问者模式和通用类型

pub mod plan_node;
pub mod plan_node_kind;
pub mod plan_node_traits;
pub mod visitor;
pub mod common;

// 重新导出核心类型
pub use plan_node::{SingleDependencyNode, SingleInputNode, BinaryInputNode, VariableDependencyNode};
pub use plan_node_kind::PlanNodeKind;
pub use plan_node_traits::{
    PlanNode, PlanNodeIdentifiable, PlanNodeProperties, PlanNodeDependencies, 
    PlanNodeMutable, PlanNodeVisitable, PlanNodeClonable, BasePlanNode
};
pub use visitor::{PlanNodeVisitor, PlanNodeVisitError, DefaultPlanNodeVisitor};
pub use common::{TagProp, EdgeProp};