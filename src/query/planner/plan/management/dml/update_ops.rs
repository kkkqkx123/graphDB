//! 更新操作相关的计划节点
//! 包括更新顶点和边等操作

use crate::query::planner::plan::core::nodes::management_node_enum::ManagementNodeEnum;
use crate::query::planner::plan::core::nodes::management_node_traits::ManagementNode;

/// 更新顶点计划节点
#[derive(Debug, Clone)]
pub struct UpdateVertex {
    pub id: i64,
    pub cost: f64,
    pub vertex_id: String,
    pub properties: Vec<(String, String)>,
}

impl UpdateVertex {
    pub fn new(id: i64, cost: f64, vertex_id: &str, properties: Vec<(String, String)>) -> Self {
        Self {
            id,
            cost,
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

impl ManagementNode for UpdateVertex {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "UpdateVertex"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::UpdateVertex(self)
    }
}

/// 更新边计划节点
#[derive(Debug, Clone)]
pub struct UpdateEdge {
    pub id: i64,
    pub cost: f64,
    pub edge_id: String,
    pub properties: Vec<(String, String)>,
}

impl UpdateEdge {
    pub fn new(id: i64, cost: f64, edge_id: &str, properties: Vec<(String, String)>) -> Self {
        Self {
            id,
            cost,
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

impl ManagementNode for UpdateEdge {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "UpdateEdge"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::UpdateEdge(self)
    }
}
