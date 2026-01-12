//! 标签操作相关的计划节点
//! 包括创建/删除标签等操作

use super::space_ops::Schema;
use super::space_ops::SchemaField;
use crate::query::planner::plan::core::nodes::management_node_enum::ManagementNodeEnum;
use crate::query::planner::plan::core::nodes::management_node_traits::ManagementNode;

/// 创建标签计划节点
#[derive(Debug, Clone)]
pub struct CreateTag {
    pub id: i64,
    pub cost: f64,
    pub name: String,
    pub schema: Schema,
    pub if_not_exists: bool,
}

impl CreateTag {
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

impl ManagementNode for CreateTag {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "CreateTag"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::CreateTag(self)
    }
}

/// 描述标签计划节点
#[derive(Debug, Clone)]
pub struct DescTag {
    pub id: i64,
    pub cost: f64,
    pub tag_name: String,
}

impl DescTag {
    pub fn new(id: i64, cost: f64, tag_name: &str) -> Self {
        Self {
            id,
            cost,
            tag_name: tag_name.to_string(),
        }
    }

    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }
}

impl ManagementNode for DescTag {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DescTag"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::DescTag(self)
    }
}

/// 删除标签计划节点
#[derive(Debug, Clone)]
pub struct DropTag {
    pub id: i64,
    pub cost: f64,
    pub if_exists: bool,
    pub tag_name: String,
}

impl DropTag {
    pub fn new(id: i64, cost: f64, if_exists: bool, tag_name: &str) -> Self {
        Self {
            id,
            cost,
            if_exists,
            tag_name: tag_name.to_string(),
        }
    }

    pub fn if_exists(&self) -> bool {
        self.if_exists
    }

    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }
}

impl ManagementNode for DropTag {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DropTag"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::DropTag(self)
    }
}

/// 显示标签列表计划节点
#[derive(Debug, Clone)]
pub struct ShowTags {
    pub id: i64,
    pub cost: f64,
}

impl ShowTags {
    pub fn new(id: i64, cost: f64) -> Self {
        Self { id, cost }
    }
}

impl ManagementNode for ShowTags {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ShowTags"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::ShowTags(self)
    }
}

/// 显示创建标签计划节点
#[derive(Debug, Clone)]
pub struct ShowCreateTag {
    pub id: i64,
    pub cost: f64,
    pub tag_name: String,
}

impl ShowCreateTag {
    pub fn new(id: i64, cost: f64, tag_name: &str) -> Self {
        Self {
            id,
            cost,
            tag_name: tag_name.to_string(),
        }
    }

    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }
}

/// 标签修改操作类型
#[derive(Debug, Clone)]
pub enum TagAlterOperation {
    AddField(SchemaField),
    DropField(String),
    ModifyField(String, SchemaField), // 字段名, 新字段定义
    SetTtlDuration(i64),              // 设置 TTL 时长
    SetTtlCol(String),                // 设置 TTL 列
    DropTtl,                          // 删除 TTL 设置
}

/// 修改标签计划节点
#[derive(Debug, Clone)]
pub struct AlterTag {
    pub id: i64,
    pub cost: f64,
    pub if_exists: bool,
    pub tag_name: String,
    pub operations: Vec<TagAlterOperation>,
}

impl AlterTag {
    pub fn new(
        id: i64,
        cost: f64,
        if_exists: bool,
        tag_name: &str,
        operations: Vec<TagAlterOperation>,
    ) -> Self {
        Self {
            id,
            cost,
            if_exists,
            tag_name: tag_name.to_string(),
            operations,
        }
    }

    pub fn if_exists(&self) -> bool {
        self.if_exists
    }

    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }

    pub fn operations(&self) -> &[TagAlterOperation] {
        &self.operations
    }
}

impl ManagementNode for AlterTag {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "AlterTag"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::AlterTag(self)
    }
}

impl ManagementNode for ShowCreateTag {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ShowCreateTag"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::ShowCreateTag(self)
    }
}
