//! 主机操作相关的计划节点
//! 包括添加/删除主机等操作

use crate::query::planner::plan::core::nodes::management_node_enum::ManagementNodeEnum;
use crate::query::planner::plan::core::nodes::management_node_traits::ManagementNode;

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
    pub id: i64,
    pub cost: f64,
    pub hosts: Vec<HostInfo>,
}

impl AddHosts {
    pub fn new(id: i64, cost: f64, hosts: Vec<HostInfo>) -> Self {
        Self { id, cost, hosts }
    }

    pub fn hosts(&self) -> &[HostInfo] {
        &self.hosts
    }
}

impl ManagementNode for AddHosts {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "AddHosts"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::AddHosts(self)
    }
}

/// 删除主机计划节点
#[derive(Debug, Clone)]
pub struct DropHosts {
    pub id: i64,
    pub cost: f64,
    pub hosts: Vec<HostInfo>,
}

impl DropHosts {
    pub fn new(id: i64, cost: f64, hosts: Vec<HostInfo>) -> Self {
        Self { id, cost, hosts }
    }

    pub fn hosts(&self) -> &[HostInfo] {
        &self.hosts
    }
}

impl ManagementNode for DropHosts {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DropHosts"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::DropHosts(self)
    }
}

/// 显示主机计划节点
#[derive(Debug, Clone)]
pub struct ShowHosts {
    pub id: i64,
    pub cost: f64,
}

impl ShowHosts {
    pub fn new(id: i64, cost: f64) -> Self {
        Self { id, cost }
    }
}

impl ManagementNode for ShowHosts {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ShowHosts"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::ShowHosts(self)
    }
}

/// 显示主机状态计划节点
#[derive(Debug, Clone)]
pub struct ShowHostsStatus {
    pub id: i64,
    pub cost: f64,
}

impl ShowHostsStatus {
    pub fn new(id: i64, cost: f64) -> Self {
        Self { id, cost }
    }
}

impl ManagementNode for ShowHostsStatus {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ShowHostsStatus"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::ShowHostsStatus(self)
    }
}
