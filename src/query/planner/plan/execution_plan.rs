//! 执行计划结构定义
//! 包含ExecutionPlan和SubPlan结构

use crate::query::planner::plan::PlanNodeEnum;

/// 执行计划结构
/// 表示完整的可执行计划，包含根节点和计划ID
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    /// 计划树的根节点
    pub root: Option<PlanNodeEnum>,

    /// 计划的唯一ID
    pub id: i64,

    /// 优化时间（微秒）
    pub optimize_time_in_us: u64,

    /// 输出格式
    pub format: String,
}

impl ExecutionPlan {
    /// 创建新的执行计划
    pub fn new(root: Option<PlanNodeEnum>) -> Self {
        Self {
            root,
            id: -1, // 将在后续分配
            optimize_time_in_us: 0,
            format: "default".to_string(),
        }
    }

    /// 设置计划的根节点
    pub fn set_root(&mut self, root: PlanNodeEnum) {
        self.root = Some(root);
    }

    /// 获取计划的根节点引用
    pub fn root(&self) -> &Option<PlanNodeEnum> {
        &self.root
    }

    /// 获取可变的根节点引用
    pub fn root_mut(&mut self) -> &mut Option<PlanNodeEnum> {
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
    
    /// 计算计划中的节点数量（简化版）
    pub fn node_count(&self) -> usize {
        fn count_nodes(node: &Option<PlanNodeEnum>) -> usize {
            match node {
                Some(n) => {
                    let count = 1;
                    count
                }
                None => 0,
            }
        }
        count_nodes(&self.root)
    }
}

/// SubPlan结构
/// 表示执行计划的一个子部分，包含根节点和尾节点
/// 用于复杂查询的分段规划
#[derive(Debug, Clone)]
pub struct SubPlan {
    /// 子计划的根节点
    pub root: Option<PlanNodeEnum>,

    /// 子计划的尾节点
    /// 用于连接多个子计划
    pub tail: Option<PlanNodeEnum>,
}

impl SubPlan {
    /// 创建新的SubPlan
    pub fn new(root: Option<PlanNodeEnum>, tail: Option<PlanNodeEnum>) -> Self {
        Self { root, tail }
    }

    /// 创建仅包含根节点的SubPlan
    pub fn from_root(root: PlanNodeEnum) -> Self {
        Self {
            root: Some(root.clone()),
            tail: Some(root),
        }
    }

    /// 创建仅包含单个节点的SubPlan
    pub fn from_single_node(node: PlanNodeEnum) -> Self {
        Self {
            root: Some(node.clone()),
            tail: Some(node),
        }
    }

    /// 获取根节点引用
    pub fn root(&self) -> &Option<PlanNodeEnum> {
        &self.root
    }

    /// 获取尾节点引用
    pub fn tail(&self) -> &Option<PlanNodeEnum> {
        &self.tail
    }

    /// 设置根节点
    pub fn set_root(&mut self, root: PlanNodeEnum) {
        self.root = Some(root);
    }

    /// 设置尾节点
    pub fn set_tail(&mut self, tail: PlanNodeEnum) {
        self.tail = Some(tail);
    }

    /// 检查SubPlan是否为空
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// 获取SubPlan中的所有节点
    pub fn collect_nodes(&self) -> Vec<PlanNodeEnum> {
        let mut nodes = Vec::new();

        if let Some(root) = &self.root {
            nodes.push(root.clone());
        }

        if let Some(tail) = &self.tail {
            nodes.push(tail.clone());
        }

        nodes
    }

    /// 合并两个SubPlan
    pub fn merge(&self, other: &SubPlan) -> SubPlan {
        let root = self.root.clone();
        let tail = other.tail.clone();

        SubPlan::new(root, tail)
    }
}
