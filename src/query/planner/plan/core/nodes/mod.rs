//! 节点类型模块
//!
//! 这个模块包含了所有具体的计划节点类型，每个节点类型都有自己独立的文件
//! 以遵循单一职责原则。

pub mod traits;
pub mod filter_node;
pub mod project_node;
pub mod join_node;
pub mod start_node;
pub mod placeholder_node;
pub mod factory;

// 重新导出所有节点类型
pub use filter_node::FilterNode;
pub use project_node::ProjectNode;
pub use join_node::InnerJoinNode;
pub use start_node::StartNode;
pub use placeholder_node::PlaceholderNode;
pub use factory::PlanNodeFactory;

// 重新导出 trait
pub use traits::{
    PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeIdentifiable,
    PlanNodeMutable, PlanNodeProperties, PlanNodeVisitable
};