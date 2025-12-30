//! 插入操作相关的计划节点
//! 包括插入顶点和边等操作

use crate::query::planner::plan::core::nodes::management_node_enum::ManagementNodeEnum;
use crate::query::planner::plan::core::nodes::management_node_traits::ManagementNode;

/// 插入顶点计划节点
#[derive(Debug, Clone)]
pub struct InsertVertices {
    pub id: i64,
    pub cost: f64,
    pub vertices: Vec<(String, Vec<(String, String)>)>,
}

impl InsertVertices {
    pub fn new(id: i64, cost: f64, vertices: Vec<(String, Vec<(String, String)>)>) -> Self {
        Self { id, cost, vertices }
    }

    pub fn vertices(&self) -> &[(String, Vec<(String, String)>)] {
        &self.vertices
    }
}

impl ManagementNode for InsertVertices {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "InsertVertices"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::InsertVertices(self)
    }
}

/// 插入边计划节点
#[derive(Debug, Clone)]
pub struct InsertEdges {
    pub id: i64,
    pub cost: f64,
    pub edges: Vec<(String, String, String, Vec<(String, String)>)>,
}

impl InsertEdges {
    pub fn new(
        id: i64,
        cost: f64,
        edges: Vec<(String, String, String, Vec<(String, String)>)>,
    ) -> Self {
        Self { id, cost, edges }
    }

    pub fn edges(&self) -> &[(String, String, String, Vec<(String, String)>)] {
        &self.edges
    }
}

impl ManagementNode for InsertEdges {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "InsertEdges"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::InsertEdges(self)
    }
}
