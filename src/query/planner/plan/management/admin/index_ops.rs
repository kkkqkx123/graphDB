//! 索引操作相关的计划节点
//! 包括创建/删除索引等操作

use crate::query::planner::plan::core::nodes::management_node_enum::ManagementNodeEnum;
use crate::query::planner::plan::core::nodes::management_node_traits::ManagementNode;
use std::sync::Arc;

/// 创建索引计划节点
#[derive(Debug, Clone)]
pub struct CreateIndex {
    pub id: i64,
    pub cost: f64,
    pub if_not_exists: bool,
    pub index_name: String,
    pub schema_name: String, // 标签或边的名称
    pub fields: Vec<String>, // 索引字段列表
}

impl CreateIndex {
    pub fn new(
        id: i64,
        cost: f64,
        if_not_exists: bool,
        index_name: &str,
        schema_name: &str,
        fields: Vec<String>,
    ) -> Self {
        Self {
            id,
            cost,
            if_not_exists,
            index_name: index_name.to_string(),
            schema_name: schema_name.to_string(),
            fields,
        }
    }

    pub fn if_not_exists(&self) -> bool {
        self.if_not_exists
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }

    pub fn schema_name(&self) -> &str {
        &self.schema_name
    }

    pub fn fields(&self) -> &[String] {
        &self.fields
    }
}

impl ManagementNode for CreateIndex {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "CreateIndex"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::CreateIndex(self)
    }
}

/// 删除索引计划节点
#[derive(Debug, Clone)]
pub struct DropIndex {
    pub id: i64,
    pub cost: f64,
    pub if_exists: bool,
    pub index_name: String,
}

impl DropIndex {
    pub fn new(id: i64, cost: f64, if_exists: bool, index_name: &str) -> Self {
        Self {
            id,
            cost,
            if_exists,
            index_name: index_name.to_string(),
        }
    }

    pub fn if_exists(&self) -> bool {
        self.if_exists
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }
}

impl ManagementNode for DropIndex {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DropIndex"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::DropIndex(self)
    }
}

/// 显示索引计划节点
#[derive(Debug, Clone)]
pub struct ShowIndexes {
    pub id: i64,
    pub cost: f64,
    pub schema_name: Option<String>, // 可选的标签或边名称
}

impl ShowIndexes {
    pub fn new(id: i64, cost: f64, schema_name: Option<String>) -> Self {
        Self { id, cost, schema_name }
    }

    pub fn schema_name(&self) -> Option<&str> {
        self.schema_name.as_deref()
    }
}

impl ManagementNode for ShowIndexes {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ShowIndexes"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::ShowIndexes(self)
    }
}

/// 描述索引计划节点
#[derive(Debug, Clone)]
pub struct DescIndex {
    pub id: i64,
    pub cost: f64,
    pub index_name: String,
}

impl DescIndex {
    pub fn new(id: i64, cost: f64, index_name: &str) -> Self {
        Self {
            id,
            cost,
            index_name: index_name.to_string(),
        }
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }
}

impl ManagementNode for DescIndex {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DescIndex"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::DescIndex(self)
    }
}
