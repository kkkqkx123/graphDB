//! 插入操作相关的计划节点
//! 包括插入顶点和边等操作

use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use std::sync::Arc;

/// 插入顶点计划节点
#[derive(Debug, Clone)]
pub struct InsertVertices {
    pub vertices: Vec<(String, Vec<(String, String)>)>,
}

impl InsertVertices {
    pub fn new(vertices: Vec<(String, Vec<(String, String)>)>) -> Self {
        Self { vertices }
    }

    pub fn vertices(&self) -> &[(String, Vec<(String, String)>)] {
        &self.vertices
    }
}

impl From<InsertVertices> for PlanNodeEnum {
    fn from(vertices: InsertVertices) -> Self {
        PlanNodeEnum::InsertVertices(Arc::new(vertices))
    }
}

/// 插入边计划节点
#[derive(Debug, Clone)]
pub struct InsertEdges {
    pub edges: Vec<(String, String, String, Vec<(String, String)>)>,
}

impl InsertEdges {
    pub fn new(edges: Vec<(String, String, String, Vec<(String, String)>)>) -> Self {
        Self { edges }
    }

    pub fn edges(&self) -> &[(String, String, String, Vec<(String, String)>)] {
        &self.edges
    }
}

impl From<InsertEdges> for PlanNodeEnum {
    fn from(edges: InsertEdges) -> Self {
        PlanNodeEnum::InsertEdges(edges)
    }
}
