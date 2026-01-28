//! 优化上下文定义
//! 定义 OptContext 结构体，管理优化过程中的状态

use std::collections::HashMap;
use std::collections::HashSet;

use super::group::OptGroup;
use super::node::{OptGroupNode, PlanNodeProperties};
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
    pub group_map: HashMap<usize, OptGroup>,
    pub statistics: Statistics,
    object_pool: ObjectPool<OptGroupNode>,
}

impl OptContext {
    pub fn new(query_context: QueryContext) -> Self {
        Self {
            query_context,
            stats: OptimizationStats::default(),
            changed: true,
            visited_groups: HashSet::new(),
            plan_node_to_group_node: HashMap::new(),
            group_map: HashMap::new(),
            statistics: Statistics::default(),
            object_pool: ObjectPool::default(),
        }
    }

    pub fn set_changed(&mut self, changed: bool) {
        self.changed = changed;
    }

    pub fn add_plan_node_and_group_node(&mut self, plan_node_id: usize, group_node: &OptGroupNode) {
        self.plan_node_to_group_node
            .insert(plan_node_id, group_node.clone());
    }

    pub fn find_group_node_by_plan_node_id(&self, plan_node_id: usize) -> Option<&OptGroupNode> {
        self.plan_node_to_group_node.get(&plan_node_id)
    }

    pub fn get_group_node_from_pool(&mut self, id: usize, plan_node: PlanNodeEnum) -> OptGroupNode {
        let mut node = self.object_pool.acquire();
        node.id = id;
        node.plan_node = plan_node;
        node.group_id = 0;
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
        self.object_pool.release(node);
    }

    pub fn validate_data_flow(&self, group_node: &OptGroupNode, boundary: &[&OptGroup]) -> bool {
        let all_deps_in_boundary = group_node
            .dependencies
            .iter()
            .all(|&dep_id| boundary.iter().any(|&group| group.id == dep_id));

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
}
