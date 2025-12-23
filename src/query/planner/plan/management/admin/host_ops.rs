//! 主机操作相关的计划节点
//! 包括添加/删除主机等操作

use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use std::sync::Arc;

// 主机信息结构
#[derive(Debug, Clone)]
pub struct HostInfo {
    pub host: String,
    pub port: u16,
    pub status: String, // ONLINE, OFFLINE, UNKNOWN
    pub leader_count: i32,
    pub leader_distribution: Vec<String>,
}

/// 添加主机计划节点
#[derive(Debug, Clone)]
pub struct AddHosts {
    pub hosts: Vec<HostInfo>,
}

impl AddHosts {
    pub fn new(hosts: Vec<HostInfo>) -> Self {
        Self { hosts }
    }

    pub fn hosts(&self) -> &[HostInfo] {
        &self.hosts
    }
}

impl From<AddHosts> for PlanNodeEnum {
    fn from(hosts: AddHosts) -> Self {
        PlanNodeEnum::AddHosts(Arc::new(hosts))
    }
}

/// 删除主机计划节点
#[derive(Debug, Clone)]
pub struct DropHosts {
    pub hosts: Vec<HostInfo>,
}

impl DropHosts {
    pub fn new(hosts: Vec<HostInfo>) -> Self {
        Self { hosts }
    }

    pub fn hosts(&self) -> &[HostInfo] {
        &self.hosts
    }
}

impl From<DropHosts> for PlanNodeEnum {
    fn from(hosts: DropHosts) -> Self {
        PlanNodeEnum::DropHosts(Arc::new(hosts))
    }
}

/// 显示主机计划节点
#[derive(Debug, Clone)]
pub struct ShowHosts;

impl ShowHosts {
    pub fn new() -> Self {
        Self
    }
}

impl From<ShowHosts> for PlanNodeEnum {
    fn from(hosts: ShowHosts) -> Self {
        PlanNodeEnum::ShowHosts(Arc::new(hosts))
    }
}

/// 显示主机状态计划节点
#[derive(Debug, Clone)]
pub struct ShowHostsStatus;

impl ShowHostsStatus {
    pub fn new() -> Self {
        Self
    }
}

impl From<ShowHostsStatus> for PlanNodeEnum {
    fn from(status: ShowHostsStatus) -> Self {
        PlanNodeEnum::ShowHostsStatus(Arc::new(status))
    }
}