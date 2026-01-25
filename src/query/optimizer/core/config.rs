//! 优化配置和统计
//! 定义优化器的配置参数和统计信息收集

use super::phase::OptimizationPhase;

#[derive(Debug)]
pub struct OptimizationConfig {
    pub max_iteration_rounds: usize,
    pub max_exploration_rounds: usize,
    pub enable_cost_model: bool,
    pub enable_multi_plan: bool,
    pub enable_property_pruning: bool,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            max_iteration_rounds: 5,
            max_exploration_rounds: 128,
            enable_cost_model: true,
            enable_multi_plan: true,
            enable_property_pruning: true,
        }
    }
}

#[derive(Debug, Default)]
pub struct OptimizationStats {
    pub rules_applied: usize,
    pub plan_nodes_before: usize,
    pub plan_nodes_after: usize,
    pub cost_before: f64,
    pub cost_after: f64,
    pub total_iterations: usize,
    pub phase: OptimizationPhase,
}

impl OptimizationStats {
    pub fn start_phase(&mut self, phase: OptimizationPhase) {
        self.phase = phase;
        self.rules_applied = 0;
    }

    pub fn record_rule_application(&mut self) {
        self.rules_applied += 1;
    }

    pub fn finalize_phase(&mut self, cost: f64) {
        if self.cost_before == 0.0 {
            self.cost_before = cost;
        }
        self.cost_after = cost;
    }
}
