//! 更新操作相关的计划节点
//! 包括更新顶点和边等操作

use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use std::sync::Arc;

/// 更新顶点计划节点
#[derive(Debug, Clone)]
pub struct UpdateVertex {
    pub vertex_id: String,
    pub properties: Vec<(String, String)>,
}

impl UpdateVertex {
    pub fn new(vertex_id: &str, properties: Vec<(String, String)>) -> Self {
        Self {
            vertex_id: vertex_id.to_string(),
            properties,
        }
    }

    pub fn vertex_id(&self) -> &str {
        &self.vertex_id
    }

    pub fn properties(&self) -> &[(String, String)] {
        &self.properties
    }
}

impl From<UpdateVertex> for PlanNodeEnum {
    fn from(vertex: UpdateVertex) -> Self {
        PlanNodeEnum::UpdateVertex(vertex)
    }
}

/// 更新边计划节点
#[derive(Debug, Clone)]
pub struct UpdateEdge {
    pub edge_id: String,
    pub properties: Vec<(String, String)>,
}

impl UpdateEdge {
    pub fn new(edge_id: &str, properties: Vec<(String, String)>) -> Self {
        Self {
            edge_id: edge_id.to_string(),
            properties,
        }
    }

    pub fn edge_id(&self) -> &str {
        &self.edge_id
    }

    pub fn properties(&self) -> &[(String, String)] {
        &self.properties
    }
}

impl From<UpdateEdge> for PlanNodeEnum {
    fn from(edge: UpdateEdge) -> Self {
        PlanNodeEnum::UpdateEdge(edge)
    }
}
