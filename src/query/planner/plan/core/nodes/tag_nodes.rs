//! 标签管理节点实现
//!
//! 提供标签管理相关的计划节点定义

use super::plan_node_enum::PlanNodeEnum;
use super::plan_node_traits::PlanNode;
use crate::core::types::PropertyDef;
use crate::query::context::validate::types::Variable;

/// 标签管理信息
#[derive(Debug, Clone)]
pub struct TagManageInfo {
    pub space_name: String,
    pub tag_name: String,
    pub properties: Vec<PropertyDef>,
}

impl TagManageInfo {
    pub fn new(space_name: String, tag_name: String) -> Self {
        Self {
            space_name,
            tag_name,
            properties: Vec::new(),
        }
    }

    pub fn with_properties(mut self, properties: Vec<PropertyDef>) -> Self {
        self.properties = properties;
        self
    }
}

/// 标签修改信息
#[derive(Debug, Clone)]
pub struct TagAlterInfo {
    pub space_name: String,
    pub tag_name: String,
    pub additions: Vec<PropertyDef>,
    pub deletions: Vec<String>,
}

impl TagAlterInfo {
    pub fn new(space_name: String, tag_name: String) -> Self {
        Self {
            space_name,
            tag_name,
            additions: Vec::new(),
            deletions: Vec::new(),
        }
    }

    pub fn with_additions(mut self, additions: Vec<PropertyDef>) -> Self {
        self.additions = additions;
        self
    }

    pub fn with_deletions(mut self, deletions: Vec<String>) -> Self {
        self.deletions = deletions;
        self
    }
}

/// 创建标签计划节点
#[derive(Debug, Clone)]
pub struct CreateTagNode {
    id: i64,
    info: TagManageInfo,
}

impl CreateTagNode {
    pub fn new(id: i64, info: TagManageInfo) -> Self {
        Self { id, info }
    }

    pub fn info(&self) -> &TagManageInfo {
        &self.info
    }

    pub fn space_name(&self) -> &str {
        &self.info.space_name
    }

    pub fn tag_name(&self) -> &str {
        &self.info.tag_name
    }
}

impl PlanNode for CreateTagNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "CreateTag"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::CreateTag(self)
    }
}

/// 修改标签计划节点
#[derive(Debug, Clone)]
pub struct AlterTagNode {
    id: i64,
    info: TagAlterInfo,
}

impl AlterTagNode {
    pub fn new(id: i64, info: TagAlterInfo) -> Self {
        Self { id, info }
    }

    pub fn info(&self) -> &TagAlterInfo {
        &self.info
    }

    pub fn space_name(&self) -> &str {
        &self.info.space_name
    }

    pub fn tag_name(&self) -> &str {
        &self.info.tag_name
    }
}

impl PlanNode for AlterTagNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "AlterTag"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::AlterTag(self)
    }
}

/// 描述标签计划节点
#[derive(Debug, Clone)]
pub struct DescTagNode {
    id: i64,
    space_name: String,
    tag_name: String,
}

impl DescTagNode {
    pub fn new(id: i64, space_name: String, tag_name: String) -> Self {
        Self { id, space_name, tag_name }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }
}

impl PlanNode for DescTagNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DescTag"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::DescTag(self)
    }
}

/// 删除标签计划节点
#[derive(Debug, Clone)]
pub struct DropTagNode {
    id: i64,
    space_name: String,
    tag_name: String,
}

impl DropTagNode {
    pub fn new(id: i64, space_name: String, tag_name: String) -> Self {
        Self { id, space_name, tag_name }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }
}

impl PlanNode for DropTagNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DropTag"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::DropTag(self)
    }
}

/// 显示所有标签计划节点
#[derive(Debug, Clone)]
pub struct ShowTagsNode {
    id: i64,
}

impl ShowTagsNode {
    pub fn new(id: i64) -> Self {
        Self { id }
    }
}

impl PlanNode for ShowTagsNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ShowTags"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::ShowTags(self)
    }
}
