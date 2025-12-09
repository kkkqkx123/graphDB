//! 查询操作相关的计划节点
//! 如GetNeighbors、GetVertices、GetEdges等

use super::plan_node::{PlanNode, PlanNodeKind, SingleInputNode, SingleDependencyNode};
use super::plan_node_visitor::{PlanNodeVisitor, PlanNodeVisitError};

// 默认的查询节点实现框架
// 具体的查询节点类型（GetNeighbors, GetVertices等）在此基础上扩展
