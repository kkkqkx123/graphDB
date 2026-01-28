//! 优化配置和统计
//! 定义优化器的配置参数和统计信息收集

use crate::query::optimizer::{OptimizationPhase, OptimizationRule, RuleConfig};

#[derive(Debug)]
pub struct OptimizationConfig {
    pub max_iteration_rounds: usize,
    pub max_exploration_rounds: usize,
    pub enable_cost_model: bool,
    pub enable_multi_plan: bool,
    pub enable_property_pruning: bool,
    pub rule_config: Option<RuleConfig>,
    pub enable_rule_registration: bool,
    pub enable_adaptive_iteration: bool,
    pub stable_threshold: usize,
    pub min_iteration_rounds: usize,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            max_iteration_rounds: 5,
            max_exploration_rounds: 128,
            enable_cost_model: true,
            enable_multi_plan: true,
            enable_property_pruning: true,
            rule_config: None,
            enable_rule_registration: false,
            enable_adaptive_iteration: true,
            stable_threshold: 2,
            min_iteration_rounds: 1,
        }
    }
}

impl OptimizationConfig {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_rule_config(rule_config: RuleConfig) -> Self {
        Self {
            max_iteration_rounds: 5,
            max_exploration_rounds: 128,
            enable_cost_model: true,
            enable_multi_plan: true,
            enable_property_pruning: true,
            rule_config: Some(rule_config),
            enable_rule_registration: false,
            enable_adaptive_iteration: true,
            stable_threshold: 2,
            min_iteration_rounds: 1,
        }
    }
    
    pub fn is_rule_enabled(&self, rule: OptimizationRule) -> bool {
        self.rule_config
            .as_ref()
            .map(|c| c.is_enabled(rule))
            .unwrap_or(true)
    }
    
    pub fn disable_rule(&mut self, rule: OptimizationRule) {
        if let Some(ref mut config) = self.rule_config {
            config.disable(rule);
        }
    }
    
    pub fn enable_rule(&mut self, rule: OptimizationRule) {
        if let Some(ref mut config) = self.rule_config {
            config.enable(rule);
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
