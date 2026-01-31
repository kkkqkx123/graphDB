//! 优化组定义
//! 定义 OptGroup 结构体，管理一组等价的执行计划节点
//!
//! OptGroup 是优化过程中的核心数据结构之一：
//! - 包含多个等价的执行计划节点（OptGroupNode）
//! - 支持成本选择：选择成本最低的节点作为最终计划
//! - 管理依赖关系：追踪组之间的数据依赖
//! - 追踪探索规则：避免重复应用相同规则

use std::collections::HashSet;
use std::fmt;

use super::node::{OptGroupNode, PlanCandidate};
use super::Pattern;

#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationPhase {
    Rewrite,
    Logical,
    Physical,
    Unknown,
}

impl Default for OptimizationPhase {
    fn default() -> Self {
        OptimizationPhase::Unknown
    }
}

impl fmt::Display for OptimizationPhase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OptimizationPhase::Rewrite => write!(f, "Rewrite"),
            OptimizationPhase::Logical => write!(f, "Logical"),
            OptimizationPhase::Physical => write!(f, "Physical"),
            OptimizationPhase::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug)]
pub struct OptGroup {
    pub id: usize,
    pub nodes: Vec<std::rc::Rc<std::cell::RefCell<OptGroupNode>>>,
    pub logical: bool,
    pub explored_rules: HashSet<String>,
    pub root_group: bool,
    pub output_var: Option<String>,
    pub bodies: Vec<usize>,
    pub group_nodes_referenced: HashSet<usize>,
    pub candidates: Vec<PlanCandidate>,
    pub phase: OptimizationPhase,
}

impl OptGroup {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            nodes: Vec::new(),
            logical: true,
            explored_rules: HashSet::new(),
            root_group: false,
            output_var: None,
            bodies: Vec::new(),
            group_nodes_referenced: HashSet::new(),
            candidates: Vec::new(),
            phase: OptimizationPhase::Unknown,
        }
    }

    pub fn add_node(&mut self, node: std::rc::Rc<std::cell::RefCell<OptGroupNode>>) {
        if !self.nodes.iter().any(|n| n.borrow().id == node.borrow().id) {
            self.nodes.push(node);
        }
    }

    pub fn remove_node(&mut self, node_id: usize) {
        self.nodes.retain(|n| n.borrow().id != node_id);
    }

    pub fn get_mut_node_by_id(&mut self, node_id: usize) -> Option<&mut std::rc::Rc<std::cell::RefCell<OptGroupNode>>> {
        self.nodes.iter_mut().find(|n| n.borrow().id == node_id)
    }

    pub fn get_node_by_id(&self, node_id: usize) -> Option<&std::rc::Rc<std::cell::RefCell<OptGroupNode>>> {
        self.nodes.iter().find(|n| n.borrow().id == node_id)
    }

    pub fn get_min_cost_group_node(&self) -> Option<std::rc::Rc<std::cell::RefCell<OptGroupNode>>> {
        self.nodes.iter()
            .min_by(|a, b| {
                let cost_a = a.borrow().cost.total();
                let cost_b = b.borrow().cost.total();
                cost_a.partial_cmp(&cost_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(std::rc::Rc::clone)
    }

    pub fn set_logical(&mut self, logical: bool) {
        self.logical = logical;
    }

    pub fn is_logical(&self) -> bool {
        self.logical
    }

    pub fn set_root_group(&mut self, root_group: bool) {
        self.root_group = root_group;
    }

    pub fn is_root_group(&self) -> bool {
        self.root_group
    }

    pub fn set_output_var(&mut self, output_var: Option<String>) {
        self.output_var = output_var;
    }

    pub fn get_output_var(&self) -> Option<&String> {
        self.output_var.as_ref()
    }

    pub fn add_body(&mut self, body_id: usize) {
        if !self.bodies.contains(&body_id) {
            self.bodies.push(body_id);
        }
    }

    pub fn get_bodies(&self) -> &[usize] {
        &self.bodies
    }

    pub fn add_group_node_referenced(&mut self, node_id: usize) {
        self.group_nodes_referenced.insert(node_id);
    }

    pub fn get_group_nodes_referenced(&self) -> &HashSet<usize> {
        &self.group_nodes_referenced
    }

    pub fn add_candidate(&mut self, candidate: PlanCandidate) {
        self.candidates.push(candidate);
    }

    pub fn get_candidates(&self) -> &[PlanCandidate] {
        &self.candidates
    }

    pub fn clear_candidates(&mut self) {
        self.candidates.clear();
    }

    pub fn set_phase(&mut self, phase: OptimizationPhase) {
        self.phase = phase;
    }

    pub fn get_phase(&self) -> &OptimizationPhase {
        &self.phase
    }

    pub fn can_be_removed(&self) -> bool {
        self.nodes.is_empty() && self.group_nodes_referenced.is_empty()
    }

    pub fn set_unexplored_exploration(&mut self) {
        self.explored_rules.clear();
    }

    pub fn get_all_dependencies(&self) -> Vec<usize> {
        let mut deps = Vec::new();
        for node_rc in &self.nodes {
            let node = node_rc.borrow();
            for &dep_id in &node.dependencies {
                if !deps.contains(&dep_id) {
                    deps.push(dep_id);
                }
            }
        }
        deps
    }

    pub fn get_all_bodies(&self) -> Vec<usize> {
        let mut bodies = Vec::new();
        for node_rc in &self.nodes {
            let node = node_rc.borrow();
            for &body_id in &node.bodies {
                if !bodies.contains(&body_id) {
                    bodies.push(body_id);
                }
            }
        }
        bodies
    }
}

impl Clone for OptGroup {
    fn clone(&self) -> Self {
        let mut new_group = OptGroup::new(self.id);
        new_group.logical = self.logical;
        new_group.root_group = self.root_group;
        new_group.output_var = self.output_var.clone();
        new_group.bodies = self.bodies.clone();
        new_group.group_nodes_referenced = self.group_nodes_referenced.clone();
        new_group.candidates = self.candidates.clone();
        new_group.phase = self.phase.clone();

        for node_rc in &self.nodes {
            new_group.nodes.push(std::rc::Rc::clone(node_rc));
        }

        new_group
    }
}
