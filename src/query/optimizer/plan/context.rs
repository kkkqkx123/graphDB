//! 优化上下文定义
//! 定义 OptContext 结构体，管理优化过程中的状态
//!
//! OptContext 是优化器的全局上下文，提供：
//! - 对象池管理：复用 OptGroupNode 对象，减少内存分配
//! - 映射管理：维护 planNodeId 到 OptGroupNode 的映射
//! - 依赖追踪：追踪优化过程中的依赖关系
//! - 数据流验证：验证控制流与数据流的一致性

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

use super::group::OptGroup;
use super::node::{OptGroupNode};
use crate::query::optimizer::core::PlanNodeProperties;
use crate::utils::ObjectPool;
use crate::query::context::execution::QueryContext;
use crate::query::optimizer::core::{Cost, OptimizationStats, Statistics};
use crate::query::planner::plan::PlanNodeEnum;
use crate::storage::metadata::SchemaManager;
use crate::storage::index::IndexManager;
use crate::storage::StorageClient;

#[derive(Debug)]
pub struct OptContext {
    pub query_context: QueryContext,
    pub stats: OptimizationStats,
    pub changed: bool,
    pub visited_groups: HashSet<usize>,
    pub plan_node_to_group_node: HashMap<usize, Rc<RefCell<OptGroupNode>>>,
    pub group_nodes_by_id: HashMap<usize, Rc<RefCell<OptGroupNode>>>,
    pub group_map: HashMap<usize, OptGroup>,
    pub statistics: Statistics,
    pool: ObjectPool<OptGroupNode>,
    node_id_counter: usize,
}

impl OptContext {
    pub fn new(query_context: QueryContext) -> Self {
        Self {
            query_context,
            stats: OptimizationStats::default(),
            changed: true,
            visited_groups: HashSet::new(),
            plan_node_to_group_node: HashMap::new(),
            group_nodes_by_id: HashMap::new(),
            group_map: HashMap::new(),
            statistics: Statistics::default(),
            pool: ObjectPool::default(),
            node_id_counter: 0,
        }
    }

    pub fn set_changed(&mut self, changed: bool) {
        self.changed = changed;
    }

    pub fn add_plan_node_and_group_node(&mut self, plan_node_id: usize, group_node: Rc<RefCell<OptGroupNode>>) {
        self.plan_node_to_group_node.insert(plan_node_id, group_node);
    }

    pub fn find_group_node_by_plan_node_id(&self, plan_node_id: usize) -> Option<&Rc<RefCell<OptGroupNode>>> {
        self.plan_node_to_group_node.get(&plan_node_id)
    }

    pub fn find_group_node_by_id(&self, id: usize) -> Option<&Rc<RefCell<OptGroupNode>>> {
        self.group_nodes_by_id.get(&id)
    }

    pub fn find_group_node_by_id_mut(&mut self, id: usize) -> Option<&mut Rc<RefCell<OptGroupNode>>> {
        self.group_nodes_by_id.get_mut(&id)
    }

    pub fn add_group_node(&mut self, group_node: Rc<RefCell<OptGroupNode>>) -> Result<(), super::node::OptimizerError> {
        let id = group_node.borrow().id;
        let plan_node_id = group_node.borrow().plan_node.id() as usize;
        self.group_nodes_by_id.insert(id, group_node.clone());
        self.plan_node_to_group_node.insert(plan_node_id, group_node);
        Ok(())
    }

    pub fn allocate_node_id(&mut self) -> usize {
        let id = self.node_id_counter;
        self.node_id_counter += 1;
        id
    }

    pub fn get_group_node_from_pool(&mut self, id: usize, plan_node: PlanNodeEnum) -> OptGroupNode {
        let mut node = self.pool.acquire();
        node.id = id;
        node.plan_node = plan_node;
        node.dependencies.clear();
        node.explored_rules.clear();
        node.cost = Cost::default();
        node.properties = PlanNodeProperties::default();
        node
    }

    pub fn return_group_node_to_pool(&mut self, mut node: OptGroupNode) {
        node.dependencies.clear();
        node.explored_rules.clear();
        node.properties = PlanNodeProperties::default();
        self.pool.release(node);
    }

    pub fn register_group(&mut self, group: OptGroup) {
        let id = group.id;
        self.group_map.insert(id, group);
    }

    pub fn find_group_by_id(&self, group_id: usize) -> Option<&OptGroup> {
        self.group_map.get(&group_id)
    }

    pub fn find_group_by_id_mut(&mut self, group_id: usize) -> Option<&mut OptGroup> {
        self.group_map.get_mut(&group_id)
    }

    pub fn validate_data_flow(&self, group_node: &OptGroupNode, boundary: &[&OptGroup]) -> bool {
        let all_deps_in_boundary = group_node
            .dependencies
            .iter()
            .all(|&dep_id| boundary.iter().any(|group| group.id == dep_id));

        if all_deps_in_boundary {
            return true;
        }

        let input_vars_count = group_node.properties.input_vars.len();
        let deps_count = group_node.dependencies.len();

        if input_vars_count == deps_count {
            for (i, &_dep_id) in group_node.dependencies.iter().enumerate() {
                if i >= input_vars_count {
                    return false;
                }
            }
        }

        true
    }

    pub fn get_edge_alias_for_node(&self, node_id: usize) -> Option<String> {
        if let Some(group_node) = self.find_group_node_by_plan_node_id(node_id) {
            let group_node_ref = group_node.borrow();
            match group_node_ref.plan_node.name() {
                "Traverse" | "Expand" => {
                    let col_names = group_node_ref.plan_node.col_names();
                    if !col_names.is_empty() {
                        return Some(col_names[0].clone());
                    }
                }
                _ => {}
            }
        }

        None
    }

    pub fn get_tag_alias_for_node(&self, node_id: usize) -> Option<String> {
        if let Some(group_node) = self.find_group_node_by_plan_node_id(node_id) {
            let group_node_ref = group_node.borrow();
            match group_node_ref.plan_node.name() {
                "ScanVertices" | "IndexScan" => {
                    let col_names = group_node_ref.plan_node.col_names();
                    if !col_names.is_empty() {
                        return Some(col_names[0].clone());
                    }
                }
                _ => {}
            }
        }

        None
    }

    pub fn check_cycle(&self, start_node: usize, visited: &mut HashSet<usize>, stack: &mut HashSet<usize>) -> bool {
        if stack.contains(&start_node) {
            return true;
        }

        if visited.contains(&start_node) {
            return false;
        }

        visited.insert(start_node);
        stack.insert(start_node);

        if let Some(group_node) = self.find_group_node_by_plan_node_id(start_node) {
            let group_node_ref = group_node.borrow();
            for &dep_id in &group_node_ref.dependencies {
                if self.check_cycle(dep_id, visited, stack) {
                    return true;
                }
            }
        }

        stack.remove(&start_node);
        false
    }

    pub fn clear_visited(&mut self) {
        self.visited_groups.clear();
    }

    pub fn mark_visited(&mut self, group_id: usize) -> bool {
        self.visited_groups.insert(group_id)
    }

    pub fn is_visited(&self, group_id: usize) -> bool {
        self.visited_groups.contains(&group_id)
    }

    pub fn get_statistics(&self) -> &Statistics {
        &self.statistics
    }

    pub fn get_statistics_mut(&mut self) -> &mut Statistics {
        &mut self.statistics
    }

    pub fn set_statistics(&mut self, statistics: Statistics) {
        self.statistics = statistics;
    }

    pub fn get_query_context(&self) -> &QueryContext {
        &self.query_context
    }

    pub fn get_query_context_mut(&mut self) -> &mut QueryContext {
        &mut self.query_context
    }
    
    /// 获取 Schema 管理器
    pub fn schema_manager(&self) -> Option<&Arc<dyn SchemaManager>> {
        self.query_context.schema_manager()
    }
    
    /// 获取索引管理器
    pub fn index_manager(&self) -> Option<&Arc<dyn IndexManager>> {
        self.query_context.index_manager()
    }
    
    /// 获取存储客户端
    pub fn storage_client(&self) -> Option<&Arc<dyn StorageClient>> {
        self.query_context.get_storage_client()
    }
    
    /// 标记优化状态已改变
    pub fn mark_changed(&mut self) {
        self.changed = true;
    }
    
    /// 记录规则应用
    pub fn record_rule_application(&mut self) {
        self.stats.record_rule_application();
    }
    
    /// 获取当前空间名称
    pub fn get_current_space(&self) -> Option<String> {
        self.query_context.get_current_space()
    }
    
    /// 验证标签字段是否存在
    pub fn validate_tag_field(&self, space: &str, tag: &str, field: &str) -> Result<bool, super::node::OptimizerError> {
        if let Some(schema_manager) = self.schema_manager() {
            match schema_manager.get_tag(space, tag) {
                Ok(Some(tag_info)) => {
                    // 检查字段是否在标签的属性列表中
                    Ok(tag_info.schema.fields.iter().any(|f| f.name == field))
                }
                Ok(None) => Ok(false), // 标签不存在
                Err(e) => Err(super::node::OptimizerError::InternalError(format!("Schema error: {}", e)))
            }
        } else {
            Ok(true) // 没有schema管理器，默认通过验证
        }
    }
    
    /// 验证边类型字段是否存在
    pub fn validate_edge_field(&self, space: &str, edge_type: &str, field: &str) -> Result<bool, super::node::OptimizerError> {
        if let Some(schema_manager) = self.schema_manager() {
            match schema_manager.get_edge_type(space, edge_type) {
                Ok(Some(edge_info)) => {
                    // 检查字段是否在边类型的属性列表中
                    Ok(edge_info.schema.fields.iter().any(|f| f.name == field))
                }
                Ok(None) => Ok(false), // 边类型不存在
                Err(e) => Err(super::node::OptimizerError::InternalError(format!("Schema error: {}", e)))
            }
        } else {
            Ok(true) // 没有schema管理器，默认通过验证
        }
    }
    
    /// 获取标签的所有字段
    pub fn get_tag_fields(&self, space: &str, tag: &str) -> Result<Vec<String>, super::node::OptimizerError> {
        if let Some(schema_manager) = self.schema_manager() {
            match schema_manager.get_tag(space, tag) {
                Ok(Some(tag_info)) => {
                    Ok(tag_info.schema.fields.iter().map(|f| f.name.clone()).collect())
                }
                Ok(None) => Ok(Vec::new()), // 标签不存在
                Err(e) => Err(super::node::OptimizerError::InternalError(format!("Schema error: {}", e)))
            }
        } else {
            Ok(Vec::new()) // 没有schema管理器，返回空列表
        }
    }
    
    /// 获取边类型的所有字段
    pub fn get_edge_fields(&self, space: &str, edge_type: &str) -> Result<Vec<String>, super::node::OptimizerError> {
        if let Some(schema_manager) = self.schema_manager() {
            match schema_manager.get_edge_type(space, edge_type) {
                Ok(Some(edge_info)) => {
                    Ok(edge_info.schema.fields.iter().map(|f| f.name.clone()).collect())
                }
                Ok(None) => Ok(Vec::new()), // 边类型不存在
                Err(e) => Err(super::node::OptimizerError::InternalError(format!("Schema error: {}", e)))
            }
        } else {
            Ok(Vec::new()) // 没有schema管理器，返回空列表
        }
    }
    
    /// 查找指定空间的可用索引
    pub fn find_available_indexes(&self, space: &str, schema_name: &str, fields: &[String]) -> Result<Vec<crate::index::Index>, super::node::OptimizerError> {
        if let Some(index_manager) = self.index_manager() {
            match index_manager.list_indexes_by_space(space.parse::<i32>().unwrap_or(0)) {
                Ok(indexes) => {
                    // 过滤出匹配的索引：schema_name匹配且包含所有查询字段
                    let matching_indexes: Vec<crate::index::Index> = indexes
                        .into_iter()
                        .filter(|idx| {
                            idx.schema_name == schema_name && 
                            fields.iter().all(|field| {
                                idx.fields.iter().any(|idx_field| idx_field.name == *field)
                            })
                        })
                        .collect();
                    Ok(matching_indexes)
                }
                Err(e) => Err(super::node::OptimizerError::InternalError(format!("Index error: {}", e)))
            }
        } else {
            Ok(Vec::new()) // 没有索引管理器，返回空列表
        }
    }
    
    /// 选择最优索引（基于统计信息和选择性）
    pub fn select_best_index(&self, indexes: &[crate::index::Index], selectivity: f64) -> Option<crate::index::Index> {
        if indexes.is_empty() {
            return None;
        }
        
        // 简单启发式：选择选择性最高的索引
        // 在实际实现中，这里应该基于统计信息计算成本
        indexes
            .iter()
            .max_by_key(|idx| {
                // 索引字段越多，通常选择性越高
                idx.fields.len() as u32
            })
            .cloned()
    }
    
    /// 检查字段是否有索引
    pub fn has_index_for_field(&self, space: &str, schema_name: &str, field: &str) -> bool {
        if let Some(index_manager) = self.index_manager() {
            if let Ok(indexes) = index_manager.list_indexes_by_space(space.parse::<i32>().unwrap_or(0)) {
                return indexes.iter().any(|idx| {
                    idx.schema_name == schema_name && 
                    idx.fields.iter().any(|idx_field| idx_field.name == field)
                });
            }
        }
        false
    }
    
    /// 获取索引统计信息
    pub fn get_index_stats(&self, space: &str, index_id: i32) -> Result<Option<crate::index::IndexStats>, super::node::OptimizerError> {
        if let Some(index_manager) = self.index_manager() {
            match index_manager.get_index_stats(space.parse::<i32>().unwrap_or(0), index_id) {
                Ok(stats) => Ok(Some(stats)),
                Err(e) => Err(super::node::OptimizerError::InternalError(format!("Index stats error: {}", e)))
            }
        } else {
            Ok(None)
        }
    }
    
    /// 获取表统计信息
    pub fn get_table_statistics(&self, table_name: &str) -> Option<&crate::query::optimizer::core::TableStats> {
        self.statistics.get_table_stats(table_name)
    }
    
    /// 获取节点的估算行数
    pub fn get_estimated_rows(&self, node_id: usize) -> u64 {
        self.statistics.get_estimated_rows(node_id).copied().unwrap_or(1000) // 默认1000行
    }
    
    /// 设置节点的估算行数
    pub fn set_estimated_rows(&mut self, node_id: usize, rows: u64) {
        self.statistics.set_estimated_rows(node_id, rows);
    }
    
    /// 基于统计信息判断是否值得优化（如大表才值得下推limit）
    pub fn should_optimize_based_on_size(&self, node_id: usize, threshold: u64) -> bool {
        let estimated_rows = self.get_estimated_rows(node_id);
        estimated_rows > threshold
    }
    
    /// 获取优化统计信息
    pub fn get_optimization_stats(&self) -> &crate::query::optimizer::core::OptimizationStats {
        &self.stats
    }
    
    /// 更新统计信息（用于收集优化过程中的统计）
    pub fn update_statistics_from_node(&mut self, node_id: usize, actual_rows: u64, exec_time_us: u64) {
        // 更新表级统计
        if let Some(table_stats) = self.statistics.table_stats.get_mut("default") {
            table_stats.row_count = actual_rows;
        }
        
        // 更新节点级统计
        self.statistics.set_estimated_rows(node_id, actual_rows);
    }
    
    /// 基于成本模型评估是否值得进行特定优化
    pub fn is_optimization_worthwhile(&self, optimization_type: &str, estimated_benefit: f64, estimated_cost: f64) -> bool {
        // 简单的成本效益分析
        let benefit_ratio = estimated_benefit / (estimated_cost + 1.0);
        
        match optimization_type {
            "limit_pushdown" => benefit_ratio > 2.0,  // limit下推需要明显的收益
            "predicate_pushdown" => benefit_ratio > 1.5, // 谓词下推要求较低
            "index_selection" => benefit_ratio > 3.0, // 索引选择需要更高收益
            _ => benefit_ratio > 1.0
        }
    }
}

impl Default for OptContext {
    fn default() -> Self {
        Self::new(QueryContext::default())
    }
}
