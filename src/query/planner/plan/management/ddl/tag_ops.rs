//! 标签操作相关的计划节点
//! 包括创建/删除标签等操作

use super::space_ops::Schema;
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use std::sync::Arc;

/// 创建标签计划节点
#[derive(Debug, Clone)]
pub struct CreateTag {
    pub name: String,
    pub schema: Schema,
    pub if_not_exists: bool,
}

impl CreateTag {
    pub fn new(name: &str, schema: Schema, if_not_exists: bool) -> Self {
        Self {
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

impl From<CreateTag> for PlanNodeEnum {
    fn from(tag: CreateTag) -> Self {
        PlanNodeEnum::CreateTag(Arc::new(tag))
    }
}

/// 描述标签计划节点
#[derive(Debug, Clone)]
pub struct DescTag {
    pub tag_name: String,
}

impl DescTag {
    pub fn new(tag_name: &str) -> Self {
        Self {
            tag_name: tag_name.to_string(),
        }
    }

    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }
}

impl From<DescTag> for PlanNodeEnum {
    fn from(tag: DescTag) -> Self {
        PlanNodeEnum::DescTag(Arc::new(tag))
    }
}

/// 删除标签计划节点
#[derive(Debug, Clone)]
pub struct DropTag {
    pub if_exists: bool,
    pub tag_name: String,
}

impl DropTag {
    pub fn new(if_exists: bool, tag_name: &str) -> Self {
        Self {
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

impl From<DropTag> for PlanNodeEnum {
    fn from(tag: DropTag) -> Self {
        PlanNodeEnum::DropTag(Arc::new(tag))
    }
}

/// 显示标签列表计划节点
#[derive(Debug, Clone)]
pub struct ShowTags;

impl ShowTags {
    pub fn new() -> Self {
        Self
    }
}

impl From<ShowTags> for PlanNodeEnum {
    fn from(tags: ShowTags) -> Self {
        PlanNodeEnum::ShowTags(Arc::new(tags))
    }
}

/// 显示创建标签计划节点
#[derive(Debug, Clone)]
pub struct ShowCreateTag {
    pub tag_name: String,
}

impl ShowCreateTag {
    pub fn new(tag_name: &str) -> Self {
        Self {
            tag_name: tag_name.to_string(),
        }
    }

    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }
}

impl From<ShowCreateTag> for PlanNodeEnum {
    fn from(tag: ShowCreateTag) -> Self {
        PlanNodeEnum::ShowCreateTag(Arc::new(tag))
    }
}
