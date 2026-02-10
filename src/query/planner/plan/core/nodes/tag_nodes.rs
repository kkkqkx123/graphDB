//! 标签管理节点实现
//!
//! 提供标签管理相关的计划节点定义

use crate::define_plan_node;
use crate::core::types::PropertyDef;

define_plan_node! {
    pub struct CreateTagNode {
        info: TagManageInfo,
    }
    enum: CreateTag
    input: ZeroInputNode
}

impl CreateTagNode {
    pub fn new(id: i64, info: TagManageInfo) -> Self {
        Self {
            id,
            info,
            output_var: None,
            col_names: Vec::new(),
            cost: 1.0,
        }
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

define_plan_node! {
    pub struct AlterTagNode {
        info: TagAlterInfo,
    }
    enum: AlterTag
    input: ZeroInputNode
}

impl AlterTagNode {
    pub fn new(id: i64, info: TagAlterInfo) -> Self {
        Self {
            id,
            info,
            output_var: None,
            col_names: Vec::new(),
            cost: 1.0,
        }
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

define_plan_node! {
    pub struct DescTagNode {
        space_name: String,
        tag_name: String,
    }
    enum: DescTag
    input: ZeroInputNode
}

impl DescTagNode {
    pub fn new(id: i64, space_name: String, tag_name: String) -> Self {
        Self {
            id,
            space_name,
            tag_name,
            output_var: None,
            col_names: Vec::new(),
            cost: 1.0,
        }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }
}

define_plan_node! {
    pub struct DropTagNode {
        space_name: String,
        tag_name: String,
    }
    enum: DropTag
    input: ZeroInputNode
}

impl DropTagNode {
    pub fn new(id: i64, space_name: String, tag_name: String) -> Self {
        Self {
            id,
            space_name,
            tag_name,
            output_var: None,
            col_names: Vec::new(),
            cost: 1.0,
        }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }
}

define_plan_node! {
    pub struct ShowTagsNode {
    }
    enum: ShowTags
    input: ZeroInputNode
}

impl ShowTagsNode {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            output_var: None,
            col_names: Vec::new(),
            cost: 1.0,
        }
    }
}

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
