//! 空间管理节点实现
//!
//! 提供图空间管理相关的计划节点定义

use super::plan_node_enum::PlanNodeEnum;
use super::plan_node_traits::PlanNode;
use crate::query::context::validate::types::Variable;

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

/// 创建图空间计划节点
#[derive(Debug, Clone)]
pub struct CreateSpaceNode {
    id: i64,
    info: SpaceManageInfo,
}

impl CreateSpaceNode {
    pub fn new(id: i64, info: SpaceManageInfo) -> Self {
        Self { id, info }
    }

    pub fn info(&self) -> &SpaceManageInfo {
        &self.info
    }
}

impl PlanNode for CreateSpaceNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "CreateSpace"
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
        PlanNodeEnum::CreateSpace(self)
    }
}

/// 删除图空间计划节点
#[derive(Debug, Clone)]
pub struct DropSpaceNode {
    id: i64,
    space_name: String,
}

impl DropSpaceNode {
    pub fn new(id: i64, space_name: String) -> Self {
        Self { id, space_name }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }
}

impl PlanNode for DropSpaceNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DropSpace"
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
        PlanNodeEnum::DropSpace(self)
    }
}

/// 描述图空间计划节点
#[derive(Debug, Clone)]
pub struct DescSpaceNode {
    id: i64,
    space_name: String,
}

impl DescSpaceNode {
    pub fn new(id: i64, space_name: String) -> Self {
        Self { id, space_name }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }
}

impl PlanNode for DescSpaceNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DescSpace"
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
        PlanNodeEnum::DescSpace(self)
    }
}

/// 显示所有图空间计划节点
#[derive(Debug, Clone)]
pub struct ShowSpacesNode {
    id: i64,
}

impl ShowSpacesNode {
    pub fn new(id: i64) -> Self {
        Self { id }
    }
}

impl PlanNode for ShowSpacesNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ShowSpaces"
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
        PlanNodeEnum::ShowSpaces(self)
    }
}
