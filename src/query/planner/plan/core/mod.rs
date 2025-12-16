//! 计划节点核心模块
//! 包含PlanNode特征、节点类型枚举、访问者模式和通用类型

pub mod common;
pub mod plan_node;
pub mod plan_node_kind;
pub mod plan_node_traits;
pub mod visitor;

// 重新导出核心类型
pub use common::{EdgeProp, TagProp};
pub use plan_node::{
    BinaryInputNode, SingleDependencyNode, SingleInputNode, VariableDependencyNode,
};
pub use plan_node_kind::PlanNodeKind;
pub use plan_node_traits::{
    BasePlanNode, PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeIdentifiable,
    PlanNodeMutable, PlanNodeProperties, PlanNodeVisitable,
};
pub use visitor::{DefaultPlanNodeVisitor, PlanNodeVisitError, PlanNodeVisitor};
