//! 优化上下文定义
//! 定义 OptContext 结构体，管理优化过程中的状态
//!
//! OptContext 是优化器的全局上下文，提供：
//! - 查询上下文访问：提供对查询执行上下文的访问
//! - 对象池管理：复用 OptGroupNode 对象，减少内存分配
//! - 节点管理：维护 planNodeId 到 OptGroupNode 的映射
//! - 变化追踪：追踪优化过程中的变化状态
//! - 统计信息缓存：缓存表级统计信息，避免重复查询
//! - 运行时行数反馈：使用实际执行结果校准估算
//!
//! 设计原则：保持简洁，避免过度复杂化，专注于核心功能
//!
//! # 重构变更
//! - 使用 Arc<QueryContext> 替代 Rc<QueryContext>，支持跨线程共享
//! - new 方法接收 Arc<QueryContext> 替代 QueryContext，避免克隆

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use super::group::OptGroup;
use super::node::{OptGroupNode};
use crate::utils::ObjectPool;
use crate::query::context::QueryContext;
use crate::query::planner::plan::PlanNodeEnum;
use crate::query::optimizer::core::cost::{TableStats, FeedbackStats};

#[derive(Debug)]
pub struct OptContext {
    /// 查询上下文 - 提供查询执行所需的基础环境
    qctx: Arc<QueryContext>,

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

    /// 统计信息缓存（表名 -> TableStats）
    /// 避免重复查询存储层，提升优化速度
    stats_cache: RefCell<HashMap<String, TableStats>>,

    /// 实际行数反馈（节点ID -> 实际行数）
    /// 用于校准后续估算
    actual_row_counts: RefCell<HashMap<usize, u64>>,

    /// 反馈统计
    /// 记录估算与实际值的对比，计算误差率
    feedback_stats: RefCell<FeedbackStats>,
}

impl OptContext {
    /// 创建新的优化上下文
    ///
    /// # 重构变更
    /// - 接收 Arc<QueryContext> 替代 QueryContext，避免克隆
    pub fn new(query_context: Arc<QueryContext>) -> Self {
        Self {
            qctx: query_context,
            changed: true,
            plan_node_to_group_node: RefCell::new(HashMap::new()),
            group_nodes_by_id: RefCell::new(HashMap::new()),
            group_map: RefCell::new(HashMap::new()),
            obj_pool: ObjectPool::default(),
            node_id_counter: 0,
            stats_cache: RefCell::new(HashMap::new()),
            actual_row_counts: RefCell::new(HashMap::new()),
            feedback_stats: RefCell::new(FeedbackStats::new()),
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

    // ==================== 统计信息缓存 ====================

    /// 获取表统计信息（带缓存）
    ///
    /// 优先从缓存获取，如果没有则返回 None
    /// 未来可从存储层获取并自动缓存
    pub fn get_table_stats(&self, table_name: &str) -> Option<TableStats> {
        self.stats_cache.borrow().get(table_name).cloned()
    }

    /// 设置表统计信息
    ///
    /// 用于手动设置统计信息（测试或手动优化）
    pub fn set_table_stats(&self, table_name: &str, stats: TableStats) {
        self.stats_cache.borrow_mut().insert(table_name.to_string(), stats);
    }

    /// 批量设置表统计信息
    pub fn set_table_stats_batch(&self, stats_map: HashMap<String, TableStats>) {
        self.stats_cache.borrow_mut().extend(stats_map);
    }

    /// 清除统计信息缓存
    pub fn clear_stats_cache(&self) {
        self.stats_cache.borrow_mut().clear();
    }

    /// 检查是否有指定表的统计信息
    pub fn has_table_stats(&self, table_name: &str) -> bool {
        self.stats_cache.borrow().contains_key(table_name)
    }

    /// 获取缓存的表名列表
    pub fn get_cached_table_names(&self) -> Vec<String> {
        self.stats_cache.borrow().keys().cloned().collect()
    }

    // ==================== 运行时行数反馈 ====================

    /// 更新实际行数（执行后调用）
    ///
    /// # 参数
    /// - `node_id`: 计划节点ID
    /// - `actual_rows`: 实际处理的行数
    /// - `estimated_rows`: 优化时估算的行数（用于校准）
    pub fn update_actual_row_count(&self, node_id: usize, actual_rows: u64, estimated_rows: u64) {
        self.actual_row_counts.borrow_mut().insert(node_id, actual_rows);
        self.feedback_stats.borrow_mut().record(estimated_rows, actual_rows);
    }

    /// 获取实际行数（如果有反馈）
    pub fn get_actual_row_count(&self, node_id: usize) -> Option<u64> {
        self.actual_row_counts.borrow().get(&node_id).copied()
    }

    /// 获取校准后的行数估算
    ///
    /// 如果有实际值则使用实际值，否则根据历史误差率校准
    pub fn get_calibrated_row_estimate(&self, node_id: usize, estimated_rows: u64) -> u64 {
        // 如果有实际值，优先使用
        if let Some(actual) = self.get_actual_row_count(node_id) {
            return actual;
        }

        // 根据历史误差率校准
        let calibration_factor = self.feedback_stats.borrow().get_calibration_factor();
        (estimated_rows as f64 * calibration_factor) as u64
    }

    /// 获取反馈统计
    pub fn get_feedback_stats(&self) -> FeedbackStats {
        self.feedback_stats.borrow().clone()
    }

    /// 获取平均误差率
    ///
    /// 返回值范围 [0.0, 1.0]，0.0 表示完全准确
    pub fn get_estimate_error_rate(&self) -> f64 {
        self.feedback_stats.borrow().average_error_rate()
    }

    /// 检查估算是否系统性偏高
    pub fn is_consistently_overestimating(&self) -> bool {
        self.feedback_stats.borrow().is_consistently_overestimating()
    }

    /// 检查估算是否系统性偏低
    pub fn is_consistently_underestimating(&self) -> bool {
        self.feedback_stats.borrow().is_consistently_underestimating()
    }

    /// 清除行数反馈
    pub fn clear_row_count_feedback(&self) {
        self.actual_row_counts.borrow_mut().clear();
        *self.feedback_stats.borrow_mut() = FeedbackStats::new();
    }

    /// 获取有反馈的节点ID列表
    pub fn get_feedback_node_ids(&self) -> Vec<usize> {
        self.actual_row_counts.borrow().keys().copied().collect()
    }
}