//! 系统管理操作相关的计划节点
//! 包括提交任务、创建快照等维护操作

use crate::query::planner::plan::core::nodes::management_node_enum::ManagementNodeEnum;
use crate::query::planner::plan::core::nodes::management_node_traits::ManagementNode;

/// 任务类型枚举
#[derive(Debug, Clone)]
pub enum JobType {
    Compaction,
    Flush,
    Stats,
    DataBalance,
    ZoneBalance,
}

/// 提交任务计划节点
#[derive(Debug, Clone)]
pub struct SubmitJob {
    pub id: i64,
    pub cost: f64,
    pub job_type: JobType,
    pub parameters: Vec<String>,
}

impl SubmitJob {
    pub fn new(id: i64, cost: f64, job_type: JobType, parameters: Vec<String>) -> Self {
        Self {
            id,
            cost,
            job_type,
            parameters,
        }
    }

    pub fn job_type(&self) -> &JobType {
        &self.job_type
    }

    pub fn parameters(&self) -> &[String] {
        &self.parameters
    }
}

impl ManagementNode for SubmitJob {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "SubmitJob"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::SubmitJob(self)
    }
}

/// 创建快照计划节点
#[derive(Debug, Clone)]
pub struct CreateSnapshot {
    pub id: i64,
    pub cost: f64,
    pub name: String,
    pub comment: Option<String>,
}

impl CreateSnapshot {
    pub fn new(id: i64, cost: f64, name: &str, comment: Option<String>) -> Self {
        Self {
            id,
            cost,
            name: name.to_string(),
            comment,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }
}

impl ManagementNode for CreateSnapshot {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "CreateSnapshot"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::CreateSnapshot(self)
    }
}

/// 删除快照计划节点
#[derive(Debug, Clone)]
pub struct DropSnapshot {
    pub id: i64,
    pub cost: f64,
    pub name: String,
}

impl DropSnapshot {
    pub fn new(id: i64, cost: f64, name: &str) -> Self {
        Self {
            id,
            cost,
            name: name.to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl ManagementNode for DropSnapshot {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DropSnapshot"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::DropSnapshot(self)
    }
}

/// 显示快照计划节点
#[derive(Debug, Clone)]
pub struct ShowSnapshots {
    pub id: i64,
    pub cost: f64,
}

impl ShowSnapshots {
    pub fn new(id: i64, cost: f64) -> Self {
        Self { id, cost }
    }
}

impl ManagementNode for ShowSnapshots {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ShowSnapshots"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::ShowSnapshots(self)
    }
}

/// 显示统计信息计划节点
#[derive(Debug, Clone)]
pub struct ShowStats {
    pub id: i64,
    pub cost: f64,
    pub space_name: Option<String>,
}

impl ShowStats {
    pub fn new(id: i64, cost: f64, space_name: Option<String>) -> Self {
        Self {
            id,
            cost,
            space_name,
        }
    }

    pub fn space_name(&self) -> Option<&str> {
        self.space_name.as_deref()
    }
}

impl ManagementNode for ShowStats {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ShowStats"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::ShowStats(self)
    }
}

/// 显示字符集计划节点
#[derive(Debug, Clone)]
pub struct ShowCharset {
    pub id: i64,
    pub cost: f64,
}

impl ShowCharset {
    pub fn new(id: i64, cost: f64) -> Self {
        Self { id, cost }
    }
}

impl ManagementNode for ShowCharset {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ShowCharset"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::ShowCharset(self)
    }
}

/// 显示排序规则计划节点
#[derive(Debug, Clone)]
pub struct ShowCollation {
    pub id: i64,
    pub cost: f64,
}

impl ShowCollation {
    pub fn new(id: i64, cost: f64) -> Self {
        Self { id, cost }
    }
}

impl ManagementNode for ShowCollation {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ShowCollation"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::ShowCollation(self)
    }
}
