//! 空间操作相关的计划节点
//! 包括创建/删除空间等操作

use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use std::sync::Arc;

// 元数据定义相关结构
#[derive(Debug, Clone)]
pub struct Schema {
    pub fields: Vec<SchemaField>,
}

#[derive(Debug, Clone)]
pub struct SchemaField {
    pub name: String,
    pub field_type: String, // 简化为字符串，实际可能是复杂类型
    pub nullable: bool,
    pub default_value: Option<String>,
}

/// 创建空间计划节点
#[derive(Debug, Clone)]
pub struct CreateSpace {
    pub if_not_exist: bool,
    pub space_name: String,
    pub partition_num: i32,
    pub replica_factor: i32,
}

impl CreateSpace {
    pub fn new(
        if_not_exist: bool,
        space_name: &str,
        partition_num: i32,
        replica_factor: i32,
    ) -> Self {
        Self {
            if_not_exist,
            space_name: space_name.to_string(),
            partition_num,
            replica_factor,
        }
    }

    pub fn if_not_exist(&self) -> bool {
        self.if_not_exist
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    pub fn partition_num(&self) -> i32 {
        self.partition_num
    }

    pub fn replica_factor(&self) -> i32 {
        self.replica_factor
    }
}

impl From<CreateSpace> for PlanNodeEnum {
    fn from(space: CreateSpace) -> Self {
        PlanNodeEnum::CreateSpace(space)
    }
}

/// 描述空间计划节点
#[derive(Debug, Clone)]
pub struct DescSpace {
    pub space_name: String,
}

impl DescSpace {
    pub fn new(space_name: &str) -> Self {
        Self {
            space_name: space_name.to_string(),
        }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }
}

impl From<DescSpace> for PlanNodeEnum {
    fn from(space: DescSpace) -> Self {
        PlanNodeEnum::DescSpace(space)
    }
}

/// 显示创建空间计划节点
#[derive(Debug, Clone)]
pub struct ShowCreateSpace {
    pub space_name: String,
}

impl ShowCreateSpace {
    pub fn new(space_name: &str) -> Self {
        Self {
            space_name: space_name.to_string(),
        }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }
}

impl From<ShowCreateSpace> for PlanNodeEnum {
    fn from(space: ShowCreateSpace) -> Self {
        PlanNodeEnum::ShowCreateSpace(space)
    }
}

/// 显示空间列表计划节点
#[derive(Debug, Clone)]
pub struct ShowSpaces;

impl ShowSpaces {
    pub fn new() -> Self {
        Self
    }
}

impl From<ShowSpaces> for PlanNodeEnum {
    fn from(spaces: ShowSpaces) -> Self {
        PlanNodeEnum::ShowSpaces(spaces)
    }
}

/// 切换空间计划节点
#[derive(Debug, Clone)]
pub struct SwitchSpace {
    pub space_name: String,
}

impl SwitchSpace {
    pub fn new(space_name: &str) -> Self {
        Self {
            space_name: space_name.to_string(),
        }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }
}

impl From<SwitchSpace> for PlanNodeEnum {
    fn from(space: SwitchSpace) -> Self {
        PlanNodeEnum::SwitchSpace(space)
    }
}

/// 删除空间计划节点
#[derive(Debug, Clone)]
pub struct DropSpace {
    pub if_exists: bool,
    pub space_name: String,
}

impl DropSpace {
    pub fn new(if_exists: bool, space_name: &str) -> Self {
        Self {
            if_exists,
            space_name: space_name.to_string(),
        }
    }

    pub fn if_exists(&self) -> bool {
        self.if_exists
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }
}

impl From<DropSpace> for PlanNodeEnum {
    fn from(space: DropSpace) -> Self {
        PlanNodeEnum::DropSpace(space)
    }
}

/// 清空空间计划节点
#[derive(Debug, Clone)]
pub struct ClearSpace {
    pub if_exists: bool,
    pub space_name: String,
}

impl ClearSpace {
    pub fn new(if_exists: bool, space_name: &str) -> Self {
        Self {
            if_exists,
            space_name: space_name.to_string(),
        }
    }

    pub fn if_exists(&self) -> bool {
        self.if_exists
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }
}

impl From<ClearSpace> for PlanNodeEnum {
    fn from(space: ClearSpace) -> Self {
        PlanNodeEnum::ClearSpace(space)
    }
}

/// 修改空间选项
#[derive(Debug, Clone)]
pub enum AlterSpaceOption {
    AddZone(String),
    RemoveZone(String),
    SetPartitionNum(i32),
    SetReplicaFactor(i32),
}

/// 修改空间计划节点
#[derive(Debug, Clone)]
pub struct AlterSpace {
    pub space_name: String,
    pub alter_options: Vec<AlterSpaceOption>,
}

impl AlterSpace {
    pub fn new(space_name: &str, alter_options: Vec<AlterSpaceOption>) -> Self {
        Self {
            space_name: space_name.to_string(),
            alter_options,
        }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    pub fn alter_options(&self) -> &[AlterSpaceOption] {
        &self.alter_options
    }
}

impl From<AlterSpace> for PlanNodeEnum {
    fn from(space: AlterSpace) -> Self {
        PlanNodeEnum::AlterSpace(space)
    }
}
