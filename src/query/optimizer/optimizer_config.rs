//! 优化器配置加载
//! 从配置文件加载优化器配置

use std::collections::HashMap;
use std::path::Path;

use toml::Value;

use crate::query::optimizer::{OptimizationRule, RuleConfig};

pub fn load_optimizer_config(config_path: &Path) -> Result<OptimizerConfigInfo, String> {
    let config_content = std::fs::read_to_string(config_path)
        .map_err(|e| format!("无法读取配置文件: {}", e))?;
    
    let config: Value = config_content.parse()
        .map_err(|e| format!("配置文件解析失败: {}", e))?;
    
    let mut config_info = OptimizerConfigInfo::default();
    
    if let Some(optimizer_table) = config.get("optimizer") {
        if let Some(max_iterations) = optimizer_table.get("max_iteration_rounds") {
            if let Some(val) = max_iterations.as_integer() {
                config_info.max_iteration_rounds = val as usize;
            }
        }
        
        if let Some(max_exploration) = optimizer_table.get("max_exploration_rounds") {
            if let Some(val) = max_exploration.as_integer() {
                config_info.max_exploration_rounds = val as usize;
            }
        }
        
        if let Some(enable_cost) = optimizer_table.get("enable_cost_model") {
            if let Some(val) = enable_cost.as_bool() {
                config_info.enable_cost_model = val;
            }
        }
        
        if let Some(enable_multi) = optimizer_table.get("enable_multi_plan") {
            if let Some(val) = enable_multi.as_bool() {
                config_info.enable_multi_plan = val;
            }
        }
        
        if let Some(enable_prune) = optimizer_table.get("enable_property_pruning") {
            if let Some(val) = enable_prune.as_bool() {
                config_info.enable_property_pruning = val;
            }
        }
        
        if let Some(enable_adaptive) = optimizer_table.get("enable_adaptive_iteration") {
            if let Some(val) = enable_adaptive.as_bool() {
                config_info.enable_adaptive_iteration = val;
            }
        }
        
        if let Some(stable) = optimizer_table.get("stable_threshold") {
            if let Some(val) = stable.as_integer() {
                config_info.stable_threshold = val as usize;
            }
        }
        
        if let Some(min) = optimizer_table.get("min_iteration_rounds") {
            if let Some(val) = min.as_integer() {
                config_info.min_iteration_rounds = val as usize;
            }
        }
        
        if let Some(disabled_rules) = optimizer_table.get("disabled_rules") {
            if let Some(table) = disabled_rules.as_table() {
                for (rule_name, value) in table {
                    if let Some(enabled) = value.as_bool() {
                        if !enabled {
                            if let Some(rule) = OptimizationRule::from_name(rule_name) {
                                config_info.disabled_rules.push(rule);
                            }
                        }
                    }
                }
            }
        }
        
        if let Some(enabled_rules) = optimizer_table.get("enabled_rules") {
            if let Some(table) = enabled_rules.as_table() {
                for (rule_name, value) in table {
                    if let Some(enabled) = value.as_bool() {
                        if enabled {
                            if let Some(rule) = OptimizationRule::from_name(rule_name) {
                                config_info.enabled_rules.push(rule);
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(config_info)
}

#[derive(Debug, Default)]
pub struct OptimizerConfigInfo {
    pub max_iteration_rounds: usize,
    pub max_exploration_rounds: usize,
    pub enable_cost_model: bool,
    pub enable_multi_plan: bool,
    pub enable_property_pruning: bool,
    pub enable_adaptive_iteration: bool,
    pub stable_threshold: usize,
    pub min_iteration_rounds: usize,
    pub disabled_rules: Vec<OptimizationRule>,
    pub enabled_rules: Vec<OptimizationRule>,
}

impl OptimizerConfigInfo {
    pub fn to_rule_config(&self) -> RuleConfig {
        let mut rule_config = RuleConfig::default();
        
        for rule in &self.disabled_rules {
            rule_config.disable(*rule);
        }
        
        for rule in &self.enabled_rules {
            rule_config.enable(*rule);
        }
        
        rule_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_load_optimizer_config() {
        let config_content = r#"
[optimizer]
max_iteration_rounds = 10
enable_cost_model = false

[optimizer.disabled_rules]
FilterPushDownRule = false
"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();
        
        let config_info = load_optimizer_config(temp_file.path()).unwrap();
        
        assert_eq!(config_info.max_iteration_rounds, 10);
        assert!(!config_info.enable_cost_model);
        assert_eq!(config_info.disabled_rules.len(), 1);
    }
}
