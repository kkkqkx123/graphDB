//! 优化组定义
//! 定义 OptGroup 结构体，表示优化过程中的逻辑/物理组

use std::collections::HashSet;

use super::node::{OptGroupNode, OptRule, PlanCandidate};
use crate::query::optimizer::core::OptimizationPhase;
use crate::query::optimizer::OptimizerError;

#[derive(Debug)]
pub struct OptGroup {
    pub id: usize,
    pub nodes: Vec<OptGroupNode>,
    pub logical: bool,
    pub explored_rules: Vec<String>,
    pub root_group: bool,
    pub output_var: Option<String>,
    pub bodies: Vec<OptGroup>,
    pub group_nodes_referenced: HashSet<usize>,
    pub candidates: Vec<PlanCandidate>,
    pub phase: OptimizationPhase,
}

impl OptGroup {
    pub fn new(id: usize, logical: bool) -> Self {
        Self {
            id,
            nodes: Vec::new(),
            logical,
            explored_rules: Vec::new(),
            root_group: false,
            output_var: None,
            bodies: Vec::new(),
            group_nodes_referenced: HashSet::new(),
            candidates: Vec::new(),
            phase: OptimizationPhase::LogicalOptimization,
        }
    }

    pub fn is_explored(&self, rule_name: &str) -> bool {
        self.explored_rules.contains(&rule_name.to_string())
    }

    pub fn set_explored(&mut self, rule: &dyn OptRule) {
        if !self.explored_rules.contains(&rule.name().to_string()) {
            self.explored_rules.push(rule.name().to_string());
        }
    }

    pub fn set_unexplored(&mut self, rule: &dyn OptRule) {
        let rule_name = rule.name();
        self.explored_rules.retain(|r| r != rule_name);
    }

    pub fn add_ref_group_node(&mut self, node_id: usize) {
        self.group_nodes_referenced.insert(node_id);
    }

    pub fn delete_ref_group_node(&mut self, node_id: usize) {
        self.group_nodes_referenced.remove(&node_id);
    }

    pub fn get_min_cost_group_node(&self) -> Option<&OptGroupNode> {
        self.nodes.iter().min_by(|a, b| {
            a.cost
                .total()
                .partial_cmp(&b.cost.total())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn get_min_cost_candidate(&self) -> Option<&PlanCandidate> {
        self.candidates.iter().min_by(|a, b| {
            a.cost
                .total()
                .partial_cmp(&b.cost.total())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn add_candidate(&mut self, candidate: PlanCandidate) {
        self.candidates.push(candidate);
    }

    pub fn add_node(&mut self, node: OptGroupNode) {
        self.nodes.push(node);
    }

    pub fn get_best_node(&self, enable_multi_plan: bool) -> Option<&OptGroupNode> {
        if enable_multi_plan {
            self.get_min_cost_group_node()
        } else {
            self.nodes.first()
        }
    }

    pub fn get_best_candidate(&self) -> Option<&PlanCandidate> {
        self.get_min_cost_candidate()
    }

    pub fn set_phase(&mut self, phase: OptimizationPhase) {
        self.phase = phase;
    }

    pub fn get_phase(&self) -> &OptimizationPhase {
        &self.phase
    }

    pub fn validate(&self, _rule: &dyn OptRule) -> Result<(), OptimizerError> {
        for node in &self.nodes {
            self.validate_data_flow(node)?;
        }
        Ok(())
    }

    fn validate_data_flow(&self, node: &OptGroupNode) -> Result<(), OptimizerError> {
        for &dep_id in &node.dependencies {
            if !self.nodes.iter().any(|n| n.id == dep_id) {
                return Err(OptimizerError::OptimizationFailed(format!(
                    "无效的依赖：节点 {} 依赖于不存在的节点 {}",
                    node.id, dep_id
                )));
            }
        }
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.candidates.is_empty()
    }

    pub fn len(&self) -> usize {
        self.nodes.len() + self.candidates.len()
    }
}
