//! 系统管理操作相关的计划节点
//! 包括提交任务、创建快照等维护操作

use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
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
    pub job_type: JobType,       // 任务类型
    pub parameters: Vec<String>, // 任务参数
}

impl SubmitJob {
    pub fn new(job_type: JobType, parameters: Vec<String>) -> Self {
        Self { job_type, parameters }
    }

    pub fn job_type(&self) -> &JobType {
        &self.job_type
    }

    pub fn parameters(&self) -> &[String] {
        &self.parameters
    }
}

impl From<SubmitJob> for PlanNodeEnum {
    fn from(job: SubmitJob) -> Self {
        PlanNodeEnum::SubmitJob(Arc::new(job))
    }
}

/// 创建快照计划节点
#[derive(Debug, Clone)]
pub struct CreateSnapshot {
    pub name: String,            // 快照名称
    pub comment: Option<String>, // 快照说明
}

impl CreateSnapshot {
    pub fn new(name: &str, comment: Option<String>) -> Self {
        Self {
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

impl From<CreateSnapshot> for PlanNodeEnum {
    fn from(snapshot: CreateSnapshot) -> Self {
        PlanNodeEnum::CreateSnapshot(Arc::new(snapshot))
    }
}

/// 删除快照计划节点
#[derive(Debug, Clone)]
pub struct DropSnapshot {
    pub name: String, // 快照名称
}

impl DropSnapshot {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl From<DropSnapshot> for PlanNodeEnum {
    fn from(snapshot: DropSnapshot) -> Self {
        PlanNodeEnum::DropSnapshot(Arc::new(snapshot))
    }
}

/// 显示快照计划节点
#[derive(Debug, Clone)]
pub struct ShowSnapshots;

impl ShowSnapshots {
    pub fn new() -> Self {
        Self
    }
}

impl From<ShowSnapshots> for PlanNodeEnum {
    fn from(snapshots: ShowSnapshots) -> Self {
        PlanNodeEnum::ShowSnapshots(Arc::new(snapshots))
    }
}