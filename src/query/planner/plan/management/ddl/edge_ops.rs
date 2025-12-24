//! 边操作相关的计划节点
//! 包括创建/删除边等操作

use super::space_ops::Schema;
use crate::query::context::validate::types::Variable;
use crate::query::planner::plan::core::nodes::management_node_enum::ManagementNodeEnum;
use crate::query::planner::plan::core::nodes::management_node_traits::ManagementNode;
use std::sync::Arc;

/// 创建边计划节点
#[derive(Debug, Clone)]
pub struct CreateEdge {
    pub id: i64,
    pub cost: f64,
    pub name: String,
    pub schema: Schema,
    pub if_not_exists: bool,
}

impl CreateEdge {
    pub fn new(id: i64, cost: f64, name: &str, schema: Schema, if_not_exists: bool) -> Self {
        Self {
            id,
            cost,
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

impl ManagementNode for CreateEdge {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "CreateEdge"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::CreateEdge(self)
    }
}

/// 删除边计划节点
#[derive(Debug, Clone)]
pub struct DropEdge {
    pub id: i64,
    pub cost: f64,
    pub if_exists: bool,
    pub edge_name: String,
}

impl DropEdge {
    pub fn new(id: i64, cost: f64, if_exists: bool, edge_name: &str) -> Self {
        Self {
            id,
            cost,
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

impl ManagementNode for DropEdge {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DropEdge"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::DropEdge(self)
    }
}

/// 显示边列表计划节点
#[derive(Debug, Clone)]
pub struct ShowEdges {
    pub id: i64,
    pub cost: f64,
}

impl ShowEdges {
    pub fn new(id: i64, cost: f64) -> Self {
        Self { id, cost }
    }
}

impl ManagementNode for ShowEdges {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ShowEdges"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::ShowEdges(self)
    }
}

/// 显示创建边计划节点
#[derive(Debug, Clone)]
pub struct ShowCreateEdge {
    pub id: i64,
    pub cost: f64,
    pub edge_name: String,
}

impl ShowCreateEdge {
    pub fn new(id: i64, cost: f64, edge_name: &str) -> Self {
        Self {
            id,
            cost,
            edge_name: edge_name.to_string(),
        }
    }

    pub fn edge_name(&self) -> &str {
        &self.edge_name
    }
}

impl ManagementNode for ShowCreateEdge {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ShowCreateEdge"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::ShowCreateEdge(self)
    }
}
