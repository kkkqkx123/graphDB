//! 删除操作相关的计划节点
//! 包括删除顶点、标签和边等操作

use crate::query::planner::plan::core::nodes::management_node_enum::ManagementNodeEnum;
use crate::query::planner::plan::core::nodes::management_node_traits::ManagementNode;

/// 删除顶点计划节点
#[derive(Debug, Clone)]
pub struct DeleteVertices {
    pub id: i64,
    pub cost: f64,
    pub vertex_ids: Vec<String>,
}

impl DeleteVertices {
    pub fn new(id: i64, cost: f64, vertex_ids: Vec<String>) -> Self {
        Self {
            id,
            cost,
            vertex_ids,
        }
    }

    pub fn vertex_ids(&self) -> &[String] {
        &self.vertex_ids
    }
}

impl ManagementNode for DeleteVertices {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DeleteVertices"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::DeleteVertices(self)
    }
}

/// 删除标签计划节点
#[derive(Debug, Clone)]
pub struct DeleteTags {
    pub id: i64,
    pub cost: f64,
    pub vertex_ids: Vec<String>,
    pub tags: Vec<String>,
}

impl DeleteTags {
    pub fn new(id: i64, cost: f64, vertex_ids: Vec<String>, tags: Vec<String>) -> Self {
        Self {
            id,
            cost,
            vertex_ids,
            tags,
        }
    }

    pub fn vertex_ids(&self) -> &[String] {
        &self.vertex_ids
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }
}

impl ManagementNode for DeleteTags {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DeleteTags"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::DeleteTags(self)
    }
}

/// 删除边计划节点
#[derive(Debug, Clone)]
pub struct DeleteEdges {
    pub id: i64,
    pub cost: f64,
    pub edge_ids: Vec<String>,
}

impl DeleteEdges {
    pub fn new(id: i64, cost: f64, edge_ids: Vec<String>) -> Self {
        Self { id, cost, edge_ids }
    }

    pub fn edge_ids(&self) -> &[String] {
        &self.edge_ids
    }
}

impl ManagementNode for DeleteEdges {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DeleteEdges"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::DeleteEdges(self)
    }
}
