//! 边操作相关的计划节点
//! 包括创建/删除边等操作

use super::space_ops::Schema;
use crate::query::context::validate::types::Variable;
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use std::sync::Arc;

/// 创建边计划节点
#[derive(Debug, Clone)]
pub struct CreateEdge {
    pub name: String,
    pub schema: Schema,
    pub if_not_exists: bool,
}

impl CreateEdge {
    pub fn new(name: &str, schema: Schema, if_not_exists: bool) -> Self {
        Self {
            name: name.to_string(),
            schema,
            if_not_exists,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    pub fn if_not_exists(&self) -> bool {
        self.if_not_exists
    }
}

impl From<CreateEdge> for PlanNodeEnum {
    fn from(edge: CreateEdge) -> Self {
        PlanNodeEnum::CreateEdge(edge)
    }
}

/// 删除边计划节点
#[derive(Debug, Clone)]
pub struct DropEdge {
    pub if_exists: bool,
    pub edge_name: String,
}

impl DropEdge {
    pub fn new(if_exists: bool, edge_name: &str) -> Self {
        Self {
            if_exists,
            edge_name: edge_name.to_string(),
        }
    }

    pub fn if_exists(&self) -> bool {
        self.if_exists
    }

    pub fn edge_name(&self) -> &str {
        &self.edge_name
    }
}

impl From<DropEdge> for PlanNodeEnum {
    fn from(edge: DropEdge) -> Self {
        PlanNodeEnum::DropEdge(edge)
    }
}

/// 显示边列表计划节点
#[derive(Debug, Clone)]
pub struct ShowEdges;

impl ShowEdges {
    pub fn new() -> Self {
        Self
    }
}

impl From<ShowEdges> for PlanNodeEnum {
    fn from(edges: ShowEdges) -> Self {
        PlanNodeEnum::ShowEdges(Arc::new(edges))
    }
}

/// 显示创建边计划节点
#[derive(Debug, Clone)]
pub struct ShowCreateEdge {
    pub edge_name: String,
}

impl ShowCreateEdge {
    pub fn new(edge_name: &str) -> Self {
        Self {
            edge_name: edge_name.to_string(),
        }
    }

    pub fn edge_name(&self) -> &str {
        &self.edge_name
    }
}

impl From<ShowCreateEdge> for PlanNodeEnum {
    fn from(edge: ShowCreateEdge) -> Self {
        PlanNodeEnum::ShowCreateEdge(Arc::new(edge))
    }
}
