//! 空间管理节点实现
//!
//! 提供图空间管理相关的计划节点定义

use crate::define_plan_node;

define_plan_node! {
    pub struct CreateSpaceNode {
        info: SpaceManageInfo,
    }
    enum: CreateSpace
    input: ZeroInputNode
}

impl CreateSpaceNode {
    pub fn new(id: i64, info: SpaceManageInfo) -> Self {
        Self {
            id,
            info,
            output_var: None,
            col_names: Vec::new(),
            cost: 1.0,
        }
    }

    pub fn info(&self) -> &SpaceManageInfo {
        &self.info
    }
}

define_plan_node! {
    pub struct DropSpaceNode {
        space_name: String,
    }
    enum: DropSpace
    input: ZeroInputNode
}

impl DropSpaceNode {
    pub fn new(id: i64, space_name: String) -> Self {
        Self {
            id,
            space_name,
            output_var: None,
            col_names: Vec::new(),
            cost: 1.0,
        }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }
}

define_plan_node! {
    pub struct DescSpaceNode {
        space_name: String,
    }
    enum: DescSpace
    input: ZeroInputNode
}

impl DescSpaceNode {
    pub fn new(id: i64, space_name: String) -> Self {
        Self {
            id,
            space_name,
            output_var: None,
            col_names: Vec::new(),
            cost: 1.0,
        }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }
}

define_plan_node! {
    pub struct ShowSpacesNode {
    }
    enum: ShowSpaces
    input: ZeroInputNode
}

impl ShowSpacesNode {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            output_var: None,
            col_names: Vec::new(),
            cost: 1.0,
        }
    }
}

/// 空间管理信息
#[derive(Debug, Clone)]
pub struct SpaceManageInfo {
    pub space_name: String,
    pub partition_num: usize,
    pub replica_factor: usize,
    pub vid_type: String,
}

impl SpaceManageInfo {
    pub fn new(space_name: String) -> Self {
        Self {
            space_name,
            partition_num: 1,
            replica_factor: 1,
            vid_type: "FIXED_STRING(32)".to_string(),
        }
    }

    pub fn with_partition_num(mut self, partition_num: usize) -> Self {
        self.partition_num = partition_num;
        self
    }

    pub fn with_replica_factor(mut self, replica_factor: usize) -> Self {
        self.replica_factor = replica_factor;
        self
    }

    pub fn with_vid_type(mut self, vid_type: String) -> Self {
        self.vid_type = vid_type;
        self
    }
}
