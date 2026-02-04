//! 优化上下文定义
//! 定义 OptContext 结构体，管理优化过程中的状态
//!
//! OptContext 是优化器的全局上下文，提供：
//! - 查询上下文访问：提供对查询执行上下文的访问
//! - 对象池管理：复用 OptGroupNode 对象，减少内存分配
//! - 节点管理：维护 planNodeId 到 OptGroupNode 的映射
//! - 变化追踪：追踪优化过程中的变化状态
//!
//! 设计原则：保持简洁，避免过度复杂化，专注于核心功能

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::group::OptGroup;
use super::node::{OptGroupNode};
use crate::utils::ObjectPool;
use crate::query::context::execution::QueryContext;
use crate::query::planner::plan::PlanNodeEnum;

#[derive(Debug)]
pub struct OptContext {
    /// 查询上下文 - 提供查询执行所需的基础环境
    qctx: Rc<QueryContext>,
    
    /// 变化状态 - 标记优化过程是否产生了变化
    changed: bool,
    
    /// 计划节点到组节点的映射 - 维护节点关系
    plan_node_to_group_node: RefCell<HashMap<usize, Rc<RefCell<OptGroupNode>>>>,
    
    /// 组节点映射 - 按ID索引组节点
    group_nodes_by_id: RefCell<HashMap<usize, Rc<RefCell<OptGroupNode>>>>,
    
    /// 组映射 - 管理优化组
    group_map: RefCell<HashMap<usize, OptGroup>>,
    
    /// 对象池 - 复用OptGroupNode对象
    obj_pool: ObjectPool<OptGroupNode>,
    
    /// 节点ID计数器 - 生成唯一节点ID
    node_id_counter: usize,
}

impl OptContext {
    /// 创建新的优化上下文
    pub fn new(query_context: QueryContext) -> Self {
        Self {
            qctx: Rc::new(query_context),
            changed: true,
            plan_node_to_group_node: RefCell::new(HashMap::new()),
            group_nodes_by_id: RefCell::new(HashMap::new()),
            group_map: RefCell::new(HashMap::new()),
            obj_pool: ObjectPool::default(),
            node_id_counter: 0,
        }
    }
    
    /// 获取查询上下文
    pub fn qctx(&self) -> &QueryContext {
        &self.qctx
    }
    
    /// 获取对象池的可变引用
    pub fn obj_pool(&mut self) -> &mut ObjectPool<OptGroupNode> {
        &mut self.obj_pool
    }
    
    /// 设置变化状态
    pub fn set_changed(&mut self, changed: bool) {
        self.changed = changed;
    }
    
    /// 添加计划节点和组节点的映射关系
    pub fn add_plan_node_and_group_node(&mut self, plan_node_id: usize, group_node: Rc<RefCell<OptGroupNode>>) {
        self.plan_node_to_group_node.borrow_mut().insert(plan_node_id, group_node);
    }
    
    /// 通过计划节点ID查找组节点
    pub fn find_group_node_by_plan_node_id(&self, plan_node_id: usize) -> Option<Rc<RefCell<OptGroupNode>>> {
        self.plan_node_to_group_node.borrow().get(&plan_node_id).cloned()
    }
    
    /// 通过ID查找组节点
    pub fn find_group_node_by_id(&self, id: usize) -> Option<Rc<RefCell<OptGroupNode>>> {
        self.group_nodes_by_id.borrow().get(&id).cloned()
    }
    
    /// 添加组节点
    pub fn add_group_node(&mut self, group_node: Rc<RefCell<OptGroupNode>>) -> Result<(), super::node::OptimizerError> {
        let id = group_node.borrow().id;
        let plan_node_id = group_node.borrow().plan_node.id() as usize;
        self.group_nodes_by_id.borrow_mut().insert(id, group_node.clone());
        self.plan_node_to_group_node.borrow_mut().insert(plan_node_id, group_node);
        Ok(())
    }

    /// 分配新的节点ID
    pub fn allocate_node_id(&mut self) -> usize {
        let id = self.node_id_counter;
        self.node_id_counter += 1;
        id
    }
    
    /// 从对象池获取组节点
    pub fn get_group_node_from_pool(&mut self, id: usize, plan_node: PlanNodeEnum) -> OptGroupNode {
        let mut node = self.obj_pool.acquire();
        node.id = id;
        node.plan_node = plan_node;
        node.dependencies.clear();
        node.explored_rules.clear();
        node
    }
    
    /// 将组节点返回对象池
    pub fn return_group_node_to_pool(&mut self, mut node: OptGroupNode) {
        node.dependencies.clear();
        node.explored_rules.clear();
        self.obj_pool.release(node);
    }
    
    /// 注册优化组
    pub fn register_group(&mut self, group: OptGroup) {
        let id = group.id;
        self.group_map.borrow_mut().insert(id, group);
    }
    
    /// 通过ID查找优化组
    pub fn find_group_by_id(&self, group_id: usize) -> Option<OptGroup> {
        self.group_map.borrow().get(&group_id).cloned()
    }
    
    /// 通过ID查找优化组的可变引用
    pub fn find_group_by_id_mut(&mut self, group_id: usize) -> Option<OptGroup> {
        self.group_map.borrow_mut().remove(&group_id)
    }
    
    /// 获取变化状态
    pub fn changed(&self) -> bool {
        self.changed
    }
    
    /// 获取组映射的可变引用
    pub fn group_map_mut(&mut self) -> std::cell::RefMut<HashMap<usize, OptGroup>> {
        self.group_map.borrow_mut()
    }
    
    /// 获取计划节点到组节点映射的可变引用
    pub fn plan_node_to_group_node_mut(&mut self) -> std::cell::RefMut<HashMap<usize, Rc<RefCell<OptGroupNode>>>> {
        self.plan_node_to_group_node.borrow_mut()
    }
}

impl Default for OptContext {
    fn default() -> Self {
        Self::new(QueryContext::default())
    }
}