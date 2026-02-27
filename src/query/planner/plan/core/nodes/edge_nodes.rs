//! 边类型管理节点实现
//!
//! 提供边类型管理相关的计划节点定义

use crate::define_plan_node;
use crate::core::types::PropertyDef;

define_plan_node! {
    pub struct CreateEdgeNode {
        info: EdgeManageInfo,
    }
    enum: CreateEdge
    input: ZeroInputNode
}

impl CreateEdgeNode {
    pub fn new(id: i64, info: EdgeManageInfo) -> Self {
        Self {
            id,
            info,
            output_var: None,
            col_names: Vec::new(),
        }
    }

    pub fn info(&self) -> &EdgeManageInfo {
        &self.info
    }

    pub fn space_name(&self) -> &str {
        &self.info.space_name
    }

    pub fn edge_name(&self) -> &str {
        &self.info.edge_name
    }
}

define_plan_node! {
    pub struct AlterEdgeNode {
        info: EdgeAlterInfo,
    }
    enum: AlterEdge
    input: ZeroInputNode
}

impl AlterEdgeNode {
    pub fn new(id: i64, info: EdgeAlterInfo) -> Self {
        Self {
            id,
            info,
            output_var: None,
            col_names: Vec::new(),
        }
    }

    pub fn info(&self) -> &EdgeAlterInfo {
        &self.info
    }

    pub fn space_name(&self) -> &str {
        &self.info.space_name
    }

    pub fn edge_name(&self) -> &str {
        &self.info.edge_name
    }
}

define_plan_node! {
    pub struct DescEdgeNode {
        space_name: String,
        edge_name: String,
    }
    enum: DescEdge
    input: ZeroInputNode
}

impl DescEdgeNode {
    pub fn new(id: i64, space_name: String, edge_name: String) -> Self {
        Self {
            id,
            space_name,
            edge_name,
            output_var: None,
            col_names: Vec::new(),
        }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    pub fn edge_name(&self) -> &str {
        &self.edge_name
    }
}

define_plan_node! {
    pub struct DropEdgeNode {
        space_name: String,
        edge_name: String,
    }
    enum: DropEdge
    input: ZeroInputNode
}

impl DropEdgeNode {
    pub fn new(id: i64, space_name: String, edge_name: String) -> Self {
        Self {
            id,
            space_name,
            edge_name,
            output_var: None,
            col_names: Vec::new(),
        }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    pub fn edge_name(&self) -> &str {
        &self.edge_name
    }
}

define_plan_node! {
    pub struct ShowEdgesNode {
    }
    enum: ShowEdges
    input: ZeroInputNode
}

impl ShowEdgesNode {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            output_var: None,
            col_names: Vec::new(),
        }
    }
}

/// 边类型管理信息
#[derive(Debug, Clone)]
pub struct EdgeManageInfo {
    pub space_name: String,
    pub edge_name: String,
    pub properties: Vec<PropertyDef>,
}

impl EdgeManageInfo {
    pub fn new(space_name: String, edge_name: String) -> Self {
        Self {
            space_name,
            edge_name,
            properties: Vec::new(),
        }
    }

    pub fn with_properties(mut self, properties: Vec<PropertyDef>) -> Self {
        self.properties = properties;
        self
    }
}

/// 边类型修改信息
#[derive(Debug, Clone)]
pub struct EdgeAlterInfo {
    pub space_name: String,
    pub edge_name: String,
    pub additions: Vec<PropertyDef>,
    pub deletions: Vec<String>,
}

impl EdgeAlterInfo {
    pub fn new(space_name: String, edge_name: String) -> Self {
        Self {
            space_name,
            edge_name,
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
