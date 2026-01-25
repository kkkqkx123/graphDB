//! 探索状态管理
//! 定义 ExplorationState 结构体，用于追踪优化过程中的探索状态

use std::collections::HashSet;

#[derive(Debug)]
pub struct ExplorationState {
    pub visited_groups: HashSet<usize>,
    pub visited_nodes: HashSet<usize>,
    pub applied_rules: Vec<String>,
    pub current_round: usize,
}

impl Default for ExplorationState {
    fn default() -> Self {
        Self {
            visited_groups: HashSet::new(),
            visited_nodes: HashSet::new(),
            applied_rules: Vec::new(),
            current_round: 0,
        }
    }
}

impl ExplorationState {
    pub fn reset_round(&mut self) {
        self.visited_groups.clear();
        self.visited_nodes.clear();
        self.current_round += 1;
    }

    pub fn is_visited_group(&self, group_id: usize) -> bool {
        self.visited_groups.contains(&group_id)
    }

    pub fn is_visited_node(&self, node_id: usize) -> bool {
        self.visited_nodes.contains(&node_id)
    }

    pub fn mark_group_visited(&mut self, group_id: usize) {
        self.visited_groups.insert(group_id);
    }

    pub fn mark_node_visited(&mut self, node_id: usize) {
        self.visited_nodes.insert(node_id);
    }

    pub fn was_rule_applied(&self, rule_name: &str) -> bool {
        self.applied_rules.iter().any(|r| r == rule_name)
    }
}
