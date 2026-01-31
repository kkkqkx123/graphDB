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

use super::group::OptGroup;
use super::node::{OptGroupNode};
use crate::query::optimizer::core::PlanNodeProperties;
use crate::utils::ObjectPool;
use crate::query::context::execution::QueryContext;
use crate::query::optimizer::core::{Cost, OptimizationStats, Statistics};
use crate::query::planner::plan::PlanNodeEnum;

#[derive(Debug)]
pub struct OptContext {
    pub query_context: QueryContext,
    pub stats: OptimizationStats,
    pub changed: bool,
    pub visited_groups: HashSet<usize>,
    pub plan_node_to_group_node: HashMap<usize, OptGroupNode>,
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

    pub fn add_plan_node_and_group_node(&mut self, plan_node_id: usize, group_node: OptGroupNode) {
        self.plan_node_to_group_node.insert(plan_node_id, group_node);
    }

    pub fn find_group_node_by_plan_node_id(&self, plan_node_id: usize) -> Option<&OptGroupNode> {
        self.plan_node_to_group_node.get(&plan_node_id)
    }

    pub fn find_group_node_by_plan_node_id_mut(&mut self, plan_node_id: usize) -> Option<&mut OptGroupNode> {
        self.plan_node_to_group_node.get_mut(&plan_node_id)
    }

    pub fn find_group_node_by_id(&self, id: usize) -> Option<&Rc<RefCell<OptGroupNode>>> {
        self.group_nodes_by_id.get(&id)
    }

    pub fn find_group_node_by_id_mut(&mut self, id: usize) -> Option<&mut Rc<RefCell<OptGroupNode>>> {
        self.group_nodes_by_id.get_mut(&id)
    }

    pub fn add_group_node(&mut self, group_node: Rc<RefCell<OptGroupNode>>) -> Result<(), super::node::OptimizerError> {
        let id = group_node.borrow().id;
        self.group_nodes_by_id.insert(id, group_node);
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
            match group_node.plan_node.name() {
                "Traverse" | "Expand" => {
                    let col_names = group_node.plan_node.col_names();
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
            match group_node.plan_node.name() {
                "ScanVertices" | "IndexScan" => {
                    let col_names = group_node.plan_node.col_names();
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
            for &dep_id in &group_node.dependencies {
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

    pub fn get_pool(&mut self) -> &mut ObjectPool<OptGroupNode> {
        &mut self.pool
    }
}

impl Default for OptContext {
    fn default() -> Self {
        Self::new(QueryContext::default())
    }
}
