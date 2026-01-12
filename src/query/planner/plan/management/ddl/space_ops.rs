//! 空间操作相关的计划节点
//! 包括创建/删除空间等操作

use crate::query::planner::plan::core::nodes::management_node_enum::ManagementNodeEnum;
use crate::query::planner::plan::core::nodes::management_node_traits::ManagementNode;

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
    pub id: i64,
    pub cost: f64,
    pub if_not_exist: bool,
    pub space_name: String,
    pub partition_num: i32,
    pub replica_factor: i32,
}

impl CreateSpace {
    pub fn new(
        id: i64,
        cost: f64,
        if_not_exist: bool,
        space_name: &str,
        partition_num: i32,
        replica_factor: i32,
    ) -> Self {
        Self {
            id,
            cost,
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

impl ManagementNode for CreateSpace {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "CreateSpace"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::CreateSpace(self)
    }
}

/// 描述空间计划节点
#[derive(Debug, Clone)]
pub struct DescSpace {
    pub id: i64,
    pub cost: f64,
    pub space_name: String,
}

impl DescSpace {
    pub fn new(id: i64, cost: f64, space_name: &str) -> Self {
        Self {
            id,
            cost,
            space_name: space_name.to_string(),
        }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }
}

impl ManagementNode for DescSpace {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DescSpace"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::DescSpace(self)
    }
}

/// 显示创建空间计划节点
#[derive(Debug, Clone)]
pub struct ShowCreateSpace {
    pub id: i64,
    pub cost: f64,
    pub space_name: String,
}

impl ShowCreateSpace {
    pub fn new(id: i64, cost: f64, space_name: &str) -> Self {
        Self {
            id,
            cost,
            space_name: space_name.to_string(),
        }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }
}

impl ManagementNode for ShowCreateSpace {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ShowCreateSpace"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::ShowCreateSpace(self)
    }
}

/// 显示空间列表计划节点
#[derive(Debug, Clone)]
pub struct ShowSpaces {
    pub id: i64,
    pub cost: f64,
}

impl ShowSpaces {
    pub fn new(id: i64, cost: f64) -> Self {
        Self { id, cost }
    }
}

impl ManagementNode for ShowSpaces {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ShowSpaces"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::ShowSpaces(self)
    }
}

/// 切换空间计划节点
#[derive(Debug, Clone)]
pub struct SwitchSpace {
    pub id: i64,
    pub cost: f64,
    pub space_name: String,
}

impl SwitchSpace {
    pub fn new(id: i64, cost: f64, space_name: &str) -> Self {
        Self {
            id,
            cost,
            space_name: space_name.to_string(),
        }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }
}

impl ManagementNode for SwitchSpace {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "SwitchSpace"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::SwitchSpace(self)
    }
}

/// 删除空间计划节点
#[derive(Debug, Clone)]
pub struct DropSpace {
    pub id: i64,
    pub cost: f64,
    pub if_exists: bool,
    pub space_name: String,
}

impl DropSpace {
    pub fn new(id: i64, cost: f64, if_exists: bool, space_name: &str) -> Self {
        Self {
            id,
            cost,
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

impl ManagementNode for DropSpace {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DropSpace"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::DropSpace(self)
    }
}

/// 清空空间计划节点
#[derive(Debug, Clone)]
pub struct ClearSpace {
    pub id: i64,
    pub cost: f64,
    pub if_exists: bool,
    pub space_name: String,
}

impl ClearSpace {
    pub fn new(id: i64, cost: f64, if_exists: bool, space_name: &str) -> Self {
        Self {
            id,
            cost,
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

impl ManagementNode for ClearSpace {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ClearSpace"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::ClearSpace(self)
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
    pub id: i64,
    pub cost: f64,
    pub space_name: String,
    pub alter_options: Vec<AlterSpaceOption>,
}

impl AlterSpace {
    pub fn new(id: i64, cost: f64, space_name: &str, alter_options: Vec<AlterSpaceOption>) -> Self {
        Self {
            id,
            cost,
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

impl ManagementNode for AlterSpace {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "AlterSpace"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::AlterSpace(self)
    }
}
