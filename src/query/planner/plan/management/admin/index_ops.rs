//! 索引操作相关的计划节点
//! 包括创建/删除索引等操作

use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use std::sync::Arc;

/// 创建索引计划节点
#[derive(Debug, Clone)]
pub struct CreateIndex {
    pub if_not_exists: bool,
    pub index_name: String,
    pub schema_name: String, // 标签或边的名称
    pub fields: Vec<String>, // 索引字段列表
}

impl CreateIndex {
    pub fn new(
        if_not_exists: bool,
        index_name: &str,
        schema_name: &str,
        fields: Vec<String>,
    ) -> Self {
        Self {
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

impl From<CreateIndex> for PlanNodeEnum {
    fn from(index: CreateIndex) -> Self {
        PlanNodeEnum::CreateIndex(index)
    }
}

/// 删除索引计划节点
#[derive(Debug, Clone)]
pub struct DropIndex {
    pub if_exists: bool,
    pub index_name: String,
}

impl DropIndex {
    pub fn new(if_exists: bool, index_name: &str) -> Self {
        Self {
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

impl From<DropIndex> for PlanNodeEnum {
    fn from(index: DropIndex) -> Self {
        PlanNodeEnum::DropIndex(index)
    }
}

/// 显示索引计划节点
#[derive(Debug, Clone)]
pub struct ShowIndexes {
    pub schema_name: Option<String>, // 可选的标签或边名称
}

impl ShowIndexes {
    pub fn new(schema_name: Option<String>) -> Self {
        Self { schema_name }
    }

    pub fn schema_name(&self) -> Option<&str> {
        self.schema_name.as_deref()
    }
}

impl From<ShowIndexes> for PlanNodeEnum {
    fn from(indexes: ShowIndexes) -> Self {
        PlanNodeEnum::ShowIndexes(Arc::new(indexes))
    }
}

/// 描述索引计划节点
#[derive(Debug, Clone)]
pub struct DescIndex {
    pub index_name: String,
}

impl DescIndex {
    pub fn new(index_name: &str) -> Self {
        Self {
            index_name: index_name.to_string(),
        }
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }
}

impl From<DescIndex> for PlanNodeEnum {
    fn from(index: DescIndex) -> Self {
        PlanNodeEnum::DescIndex(index)
    }
}
