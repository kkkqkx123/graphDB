//! 执行计划结构定义
//! 包含ExecutionPlan和SubPlan结构

use super::plan_node::PlanNode;

/// 执行计划结构
/// 表示完整的可执行计划，包含根节点和计划ID
#[derive(Debug)]
pub struct ExecutionPlan {
    /// 计划树的根节点
    pub root: Option<Box<dyn PlanNode>>,
    
    /// 计划的唯一ID
    pub id: i64,
    
    /// 优化时间（微秒）
    pub optimize_time_in_us: u64,
    
    /// 输出格式
    pub format: String,
}

impl ExecutionPlan {
    /// 创建新的执行计划
    pub fn new(root: Option<Box<dyn PlanNode>>) -> Self {
        Self {
            root,
            id: -1, // 将在后续分配
            optimize_time_in_us: 0,
            format: "default".to_string(),
        }
    }

    /// 设置计划的根节点
    pub fn set_root(&mut self, root: Box<dyn PlanNode>) {
        self.root = Some(root);
    }

    /// 获取计划的根节点引用
    pub fn root(&self) -> &Option<Box<dyn PlanNode>> {
        &self.root
    }
    
    /// 获取可变的根节点引用
    pub fn root_mut(&mut self) -> &mut Option<Box<dyn PlanNode>> {
        &mut self.root
    }
    
    /// 设置计划的ID
    pub fn set_id(&mut self, id: i64) {
        self.id = id;
    }
    
    /// 设置优化时间
    pub fn set_optimize_time(&mut self, time_us: u64) {
        self.optimize_time_in_us = time_us;
    }
    
    /// 设置输出格式
    pub fn set_format(&mut self, format: String) {
        self.format = format;
    }
}

/// SubPlan结构
/// 表示执行计划的一个子部分，包含根节点和尾节点
/// 用于复杂查询的分段规划
#[derive(Debug, Clone)]
pub struct SubPlan {
    /// 子计划的根节点
    pub root: Option<Box<dyn PlanNode>>,
    
    /// 子计划的尾节点
    /// 用于连接多个子计划
    pub tail: Option<Box<dyn PlanNode>>,
}

impl SubPlan {
    /// 创建新的SubPlan
    pub fn new(root: Option<Box<dyn PlanNode>>, tail: Option<Box<dyn PlanNode>>) -> Self {
        Self { root, tail }
    }
    
    /// 创建仅包含根节点的SubPlan
    pub fn from_root(root: Box<dyn PlanNode>) -> Self {
        Self {
            root: Some(root.clone_plan_node()),
            tail: Some(root),
        }
    }
    
    /// 获取根节点引用
    pub fn root(&self) -> &Option<Box<dyn PlanNode>> {
        &self.root
    }
    
    /// 获取尾节点引用
    pub fn tail(&self) -> &Option<Box<dyn PlanNode>> {
        &self.tail
    }
    
    /// 设置根节点
    pub fn set_root(&mut self, root: Box<dyn PlanNode>) {
        self.root = Some(root);
    }
    
    /// 设置尾节点
    pub fn set_tail(&mut self, tail: Box<dyn PlanNode>) {
        self.tail = Some(tail);
    }
}
