//! 计划节点核心模块
//! 包含PlanNode特征、节点类型枚举、访问者模式和通用类型

pub mod common;
pub mod nodes;
pub mod plan_node_kind;
pub mod visitor;

// 重新导出核心类型
pub use common::{EdgeProp, TagProp};
pub use nodes::{
    FilterNode, InnerJoinNode, PlaceholderNode, PlanNodeFactory, ProjectNode, StartNode, PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeIdentifiable,
    PlanNodeMutable, PlanNodeProperties, PlanNodeVisitable,
};
pub use plan_node_kind::PlanNodeKind;
pub use visitor::{DefaultPlanNodeVisitor, PlanNodeVisitError, PlanNodeVisitor};

// 为了向后兼容，创建 plan_node_traits 模块别名
pub mod plan_node_traits {
    pub use super::nodes::traits::*;
}
