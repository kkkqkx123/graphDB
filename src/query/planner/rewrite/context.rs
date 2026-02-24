//! 重写上下文定义
//!
//! 定义 RewriteContext 结构体，管理重写过程中的状态。
//! 这是从 optimizer 层独立出来的简化版本，专注于启发式重写规则的需求。

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::query::planner::plan::PlanNodeEnum;

/// 重写上下文
///
/// 管理重写过程中的状态和节点信息。
/// 相比 optimizer 的 OptContext，这是一个轻量级版本，
/// 不包含统计信息缓存、代价计算等优化器特有功能。
#[derive(Debug)]
pub struct RewriteContext {
    /// 节点ID计数器 - 生成唯一节点ID
    node_id_counter: usize,
    /// 计划节点到ID的映射
    plan_node_to_id: RefCell<HashMap<usize, usize>>,
    /// ID到计划节点的映射
    nodes_by_id: RefCell<HashMap<usize, Rc<RefCell<PlanNodeWrapper>>>>,
}

/// 计划节点包装器
///
/// 包装 PlanNodeEnum 并添加重写所需的元数据
#[derive(Debug, Clone)]
pub struct PlanNodeWrapper {
    pub id: usize,
    pub plan_node: PlanNodeEnum,
    pub dependencies: Vec<usize>,
}

impl PlanNodeWrapper {
    pub fn new(id: usize, plan_node: PlanNodeEnum) -> Self {
        Self {
            id,
            plan_node,
            dependencies: Vec::new(),
        }
    }
}

impl RewriteContext {
    /// 创建新的重写上下文
    pub fn new() -> Self {
        Self {
            node_id_counter: 0,
            plan_node_to_id: RefCell::new(HashMap::new()),
            nodes_by_id: RefCell::new(HashMap::new()),
        }
    }

    /// 分配新的节点ID
    pub fn allocate_node_id(&mut self) -> usize {
        let id = self.node_id_counter;
        self.node_id_counter += 1;
        id
    }

    /// 注册计划节点
    pub fn register_node(&mut self, node_id: usize, plan_node: PlanNodeEnum) -> Rc<RefCell<PlanNodeWrapper>> {
        let wrapper = Rc::new(RefCell::new(PlanNodeWrapper::new(node_id, plan_node)));
        self.nodes_by_id.borrow_mut().insert(node_id, wrapper.clone());
        wrapper
    }

    /// 通过ID查找节点
    pub fn find_node_by_id(&self, id: usize) -> Option<Rc<RefCell<PlanNodeWrapper>>> {
        self.nodes_by_id.borrow().get(&id).cloned()
    }

    /// 添加节点映射
    pub fn add_plan_node_mapping(&self, plan_node_id: usize, rewrite_node_id: usize) {
        self.plan_node_to_id.borrow_mut().insert(plan_node_id, rewrite_node_id);
    }

    /// 通过计划节点ID查找重写节点ID
    pub fn find_rewrite_id_by_plan_id(&self, plan_node_id: usize) -> Option<usize> {
        self.plan_node_to_id.borrow().get(&plan_node_id).copied()
    }

    /// 获取当前节点计数
    pub fn node_count(&self) -> usize {
        self.node_id_counter
    }
}

impl Default for RewriteContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::plan::core::nodes::graph_scan_node::ScanVerticesNode;

    #[test]
    fn test_context_creation() {
        let ctx = RewriteContext::new();
        assert_eq!(ctx.node_count(), 0);
    }

    #[test]
    fn test_allocate_node_id() {
        let mut ctx = RewriteContext::new();
        assert_eq!(ctx.allocate_node_id(), 0);
        assert_eq!(ctx.allocate_node_id(), 1);
        assert_eq!(ctx.allocate_node_id(), 2);
    }

    #[test]
    fn test_register_and_find_node() {
        let mut ctx = RewriteContext::new();
        let node_id = ctx.allocate_node_id();
        let plan_node = PlanNodeEnum::ScanVertices(ScanVerticesNode::new(1));
        
        let wrapper = ctx.register_node(node_id, plan_node);
        assert_eq!(wrapper.borrow().id, node_id);
        
        let found = ctx.find_node_by_id(node_id);
        assert!(found.is_some());
        assert_eq!(found.expect("Failed to find node").borrow().id, node_id);
    }
}
