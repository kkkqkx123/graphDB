//! 索引管理节点实现
//!
//! 提供索引管理相关的计划节点定义

use crate::define_plan_node;

define_plan_node! {
    pub struct CreateTagIndexNode {
        info: IndexManageInfo,
    }
    enum: CreateTagIndex
    input: ZeroInputNode
}

impl CreateTagIndexNode {
    pub fn new(id: i64, info: IndexManageInfo) -> Self {
        Self {
            id,
            info,
            output_var: None,
            col_names: Vec::new(),
            cost: 1.0,
        }
    }

    pub fn info(&self) -> &IndexManageInfo {
        &self.info
    }

    pub fn space_name(&self) -> &str {
        &self.info.space_name
    }

    pub fn index_name(&self) -> &str {
        &self.info.index_name
    }
}

define_plan_node! {
    pub struct DropTagIndexNode {
        space_name: String,
        index_name: String,
    }
    enum: DropTagIndex
    input: ZeroInputNode
}

impl DropTagIndexNode {
    pub fn new(id: i64, space_name: String, index_name: String) -> Self {
        Self {
            id,
            space_name,
            index_name,
            output_var: None,
            col_names: Vec::new(),
            cost: 1.0,
        }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }
}

define_plan_node! {
    pub struct DescTagIndexNode {
        space_name: String,
        index_name: String,
    }
    enum: DescTagIndex
    input: ZeroInputNode
}

impl DescTagIndexNode {
    pub fn new(id: i64, space_name: String, index_name: String) -> Self {
        Self {
            id,
            space_name,
            index_name,
            output_var: None,
            col_names: Vec::new(),
            cost: 1.0,
        }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }
}

define_plan_node! {
    pub struct ShowTagIndexesNode {
    }
    enum: ShowTagIndexes
    input: ZeroInputNode
}

impl ShowTagIndexesNode {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            output_var: None,
            col_names: Vec::new(),
            cost: 1.0,
        }
    }
}

define_plan_node! {
    pub struct CreateEdgeIndexNode {
        info: IndexManageInfo,
    }
    enum: CreateEdgeIndex
    input: ZeroInputNode
}

impl CreateEdgeIndexNode {
    pub fn new(id: i64, info: IndexManageInfo) -> Self {
        Self {
            id,
            info,
            output_var: None,
            col_names: Vec::new(),
            cost: 1.0,
        }
    }

    pub fn info(&self) -> &IndexManageInfo {
        &self.info
    }

    pub fn space_name(&self) -> &str {
        &self.info.space_name
    }

    pub fn index_name(&self) -> &str {
        &self.info.index_name
    }
}

define_plan_node! {
    pub struct DropEdgeIndexNode {
        space_name: String,
        index_name: String,
    }
    enum: DropEdgeIndex
    input: ZeroInputNode
}

impl DropEdgeIndexNode {
    pub fn new(id: i64, space_name: String, index_name: String) -> Self {
        Self {
            id,
            space_name,
            index_name,
            output_var: None,
            col_names: Vec::new(),
            cost: 1.0,
        }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }
}

define_plan_node! {
    pub struct DescEdgeIndexNode {
        space_name: String,
        index_name: String,
    }
    enum: DescEdgeIndex
    input: ZeroInputNode
}

impl DescEdgeIndexNode {
    pub fn new(id: i64, space_name: String, index_name: String) -> Self {
        Self {
            id,
            space_name,
            index_name,
            output_var: None,
            col_names: Vec::new(),
            cost: 1.0,
        }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }
}

define_plan_node! {
    pub struct ShowEdgeIndexesNode {
    }
    enum: ShowEdgeIndexes
    input: ZeroInputNode
}

impl ShowEdgeIndexesNode {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            output_var: None,
            col_names: Vec::new(),
            cost: 1.0,
        }
    }
}

define_plan_node! {
    pub struct RebuildTagIndexNode {
        space_name: String,
        index_name: String,
    }
    enum: RebuildTagIndex
    input: ZeroInputNode
}

impl RebuildTagIndexNode {
    pub fn new(id: i64, space_name: String, index_name: String) -> Self {
        Self {
            id,
            space_name,
            index_name,
            output_var: None,
            col_names: Vec::new(),
            cost: 1.0,
        }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }
}

define_plan_node! {
    pub struct RebuildEdgeIndexNode {
        space_name: String,
        index_name: String,
    }
    enum: RebuildEdgeIndex
    input: ZeroInputNode
}

impl RebuildEdgeIndexNode {
    pub fn new(id: i64, space_name: String, index_name: String) -> Self {
        Self {
            id,
            space_name,
            index_name,
            output_var: None,
            col_names: Vec::new(),
            cost: 1.0,
        }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }
}

/// 索引管理信息
#[derive(Debug, Clone)]
pub struct IndexManageInfo {
    pub space_name: String,
    pub index_name: String,
    pub target_type: String,
    pub target_name: String,
    pub properties: Vec<String>,
}

impl IndexManageInfo {
    pub fn new(space_name: String, index_name: String, target_type: String) -> Self {
        Self {
            space_name,
            index_name,
            target_type,
            target_name: String::new(),
            properties: Vec::new(),
        }
    }

    pub fn with_target_name(mut self, target_name: String) -> Self {
        self.target_name = target_name;
        self
    }

    pub fn with_properties(mut self, properties: Vec<String>) -> Self {
        self.properties = properties;
        self
    }
}
