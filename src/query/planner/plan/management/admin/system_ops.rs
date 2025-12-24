//! 系统管理操作相关的计划节点
//! 包括提交任务、创建快照等维护操作

use crate::query::planner::plan::core::nodes::management_node_enum::ManagementNodeEnum;
use crate::query::planner::plan::core::nodes::management_node_traits::ManagementNode;
use std::sync::Arc;

// 任务类型枚举
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
    pub job_type: JobType,       // 任务类型
    pub parameters: Vec<String>, // 任务参数
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
    pub name: String,            // 快照名称
    pub comment: Option<String>, // 快照说明
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
    pub name: String, // 快照名称
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
