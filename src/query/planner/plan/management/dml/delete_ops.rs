//! 删除操作相关的计划节点
//! 包括删除顶点、标签和边等操作

use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use std::sync::Arc;

/// 删除顶点计划节点
#[derive(Debug, Clone)]
pub struct DeleteVertices {
    pub vertex_ids: Vec<String>,
}

impl DeleteVertices {
    pub fn new(vertex_ids: Vec<String>) -> Self {
        Self { vertex_ids }
    }

    pub fn vertex_ids(&self) -> &[String] {
        &self.vertex_ids
    }
}

impl From<DeleteVertices> for PlanNodeEnum {
    fn from(vertices: DeleteVertices) -> Self {
        PlanNodeEnum::DeleteVertices(vertices)
    }
}

/// 删除标签计划节点
#[derive(Debug, Clone)]
pub struct DeleteTags {
    pub vertex_ids: Vec<String>,
    pub tags: Vec<String>,
}

impl DeleteTags {
    pub fn new(vertex_ids: Vec<String>, tags: Vec<String>) -> Self {
        Self { vertex_ids, tags }
    }

    pub fn vertex_ids(&self) -> &[String] {
        &self.vertex_ids
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }
}

impl From<DeleteTags> for PlanNodeEnum {
    fn from(tags: DeleteTags) -> Self {
        PlanNodeEnum::DeleteTags(tags)
    }
}

/// 删除边计划节点
#[derive(Debug, Clone)]
pub struct DeleteEdges {
    pub edge_ids: Vec<String>,
}

impl DeleteEdges {
    pub fn new(edge_ids: Vec<String>) -> Self {
        Self { edge_ids }
    }

    pub fn edge_ids(&self) -> &[String] {
        &self.edge_ids
    }
}

impl From<DeleteEdges> for PlanNodeEnum {
    fn from(edges: DeleteEdges) -> Self {
        PlanNodeEnum::DeleteEdges(edges)
    }
}
