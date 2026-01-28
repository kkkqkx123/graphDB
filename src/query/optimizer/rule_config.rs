//! 规则配置和启用控制
//! 支持通过配置启用或禁用特定优化规则

use std::collections::HashMap;

use super::rule_enum::OptimizationRule;

#[derive(Debug, Clone)]
pub struct RuleConfig {
    pub enabled_rules: Vec<OptimizationRule>,
    pub disabled_rules: Vec<OptimizationRule>,
    pub rule_flags: HashMap<&'static str, bool>,
}

impl Default for RuleConfig {
    fn default() -> Self {
        let mut config = Self {
            enabled_rules: Vec::new(),
            disabled_rules: Vec::new(),
            rule_flags: HashMap::new(),
        };
        
        config.init_default_flags();
        config
    }
}

impl RuleConfig {
    fn init_default_flags(&mut self) {
        self.rule_flags.insert("FilterPushDownRule", true);
        self.rule_flags.insert("PredicatePushDownRule", true);
        self.rule_flags.insert("ProjectionPushDownRule", true);
        self.rule_flags.insert("CombineFilterRule", true);
        self.rule_flags.insert("CollapseProjectRule", true);
        self.rule_flags.insert("DedupEliminationRule", true);
        self.rule_flags.insert("EliminateFilterRule", true);
        self.rule_flags.insert("RemoveNoopProjectRule", true);
        self.rule_flags.insert("EliminateAppendVerticesRule", true);
        self.rule_flags.insert("RemoveAppendVerticesBelowJoinRule", true);
        self.rule_flags.insert("TopNRule", true);
        self.rule_flags.insert("MergeGetVerticesAndProjectRule", true);
        self.rule_flags.insert("MergeGetVerticesAndDedupRule", true);
        self.rule_flags.insert("MergeGetNbrsAndProjectRule", true);
        self.rule_flags.insert("MergeGetNbrsAndDedupRule", true);
        
        self.rule_flags.insert("JoinOptimizationRule", true);
        self.rule_flags.insert("PushLimitDownRule", true);
        self.rule_flags.insert("PushLimitDownGetVerticesRule", true);
        self.rule_flags.insert("PushLimitDownGetNeighborsRule", true);
        self.rule_flags.insert("PushLimitDownGetEdgesRule", true);
        self.rule_flags.insert("PushLimitDownScanVerticesRule", true);
        self.rule_flags.insert("PushLimitDownScanEdgesRule", true);
        self.rule_flags.insert("PushLimitDownIndexScanRule", true);
        self.rule_flags.insert("PushLimitDownProjectRule", true);
        self.rule_flags.insert("ScanWithFilterOptimizationRule", true);
        self.rule_flags.insert("IndexFullScanRule", true);
        self.rule_flags.insert("IndexScanRule", true);
        self.rule_flags.insert("EdgeIndexFullScanRule", true);
        self.rule_flags.insert("TagIndexFullScanRule", true);
        self.rule_flags.insert("UnionAllEdgeIndexScanRule", true);
        self.rule_flags.insert("UnionAllTagIndexScanRule", true);
        self.rule_flags.insert("OptimizeEdgeIndexScanByFilterRule", true);
        self.rule_flags.insert("OptimizeTagIndexScanByFilterRule", true);
        
        self.rule_flags.insert("RemoveUselessNodeRule", false);
    }
    
    pub fn is_enabled(&self, rule: OptimizationRule) -> bool {
        let rule_name = rule.name();
        
        if self.disabled_rules.contains(&rule) {
            return false;
        }
        
        if !self.enabled_rules.is_empty() {
            return self.enabled_rules.contains(&rule);
        }
        
        self.rule_flags.get(rule_name).copied().unwrap_or(true)
    }
    
    pub fn is_enabled_by_name(&self, rule_name: &str) -> bool {
        if let Some(rule) = OptimizationRule::from_name(rule_name) {
            self.is_enabled(rule)
        } else {
            self.rule_flags.get(rule_name).copied().unwrap_or(true)
        }
    }
    
    pub fn enable(&mut self, rule: OptimizationRule) {
        self.disabled_rules.retain(|r| r != &rule);
        self.enabled_rules.push(rule);
        self.rule_flags.insert(rule.name(), true);
    }
    
    pub fn disable(&mut self, rule: OptimizationRule) {
        self.enabled_rules.retain(|r| r != &rule);
        self.disabled_rules.push(rule);
        self.rule_flags.insert(rule.name(), false);
    }
    
    pub fn set_flag(&mut self, rule_name: &'static str, enabled: bool) {
        self.rule_flags.insert(rule_name, enabled);
    }
    
    pub fn enable_by_name(&mut self, rule_name: &'static str) {
        if let Some(rule) = OptimizationRule::from_name(rule_name) {
            self.enable(rule);
        } else {
            self.rule_flags.insert(rule_name, true);
        }
    }
    
    pub fn disable_by_name(&mut self, rule_name: &'static str) {
        if let Some(rule) = OptimizationRule::from_name(rule_name) {
            self.disable(rule);
        } else {
            self.rule_flags.insert(rule_name, false);
        }
    }
    
    pub fn get_enabled_rules(&self) -> Vec<OptimizationRule> {
        let mut all_rules = Vec::new();
        
        for rule in self.iter_all_rules() {
            if self.is_enabled(rule) {
                all_rules.push(rule);
            }
        }
        
        all_rules
    }
    
    fn iter_all_rules(&self) -> impl Iterator<Item = OptimizationRule> {
        [
            OptimizationRule::FilterPushDown,
            OptimizationRule::PredicatePushDown,
            OptimizationRule::ProjectionPushDown,
            OptimizationRule::CombineFilter,
            OptimizationRule::CollapseProject,
            OptimizationRule::DedupElimination,
            OptimizationRule::EliminateFilter,
            OptimizationRule::RemoveNoopProject,
            OptimizationRule::EliminateAppendVertices,
            OptimizationRule::RemoveAppendVerticesBelowJoin,
            OptimizationRule::TopN,
            OptimizationRule::MergeGetVerticesAndProject,
            OptimizationRule::MergeGetVerticesAndDedup,
            OptimizationRule::MergeGetNbrsAndProject,
            OptimizationRule::MergeGetNbrsAndDedup,
            OptimizationRule::JoinOptimization,
            OptimizationRule::PushLimitDown,
            OptimizationRule::PushLimitDownGetVertices,
            OptimizationRule::PushLimitDownGetNeighbors,
            OptimizationRule::PushLimitDownGetEdges,
            OptimizationRule::PushLimitDownScanVertices,
            OptimizationRule::PushLimitDownScanEdges,
            OptimizationRule::PushLimitDownIndexScan,
            OptimizationRule::PushLimitDownProjectRule,
            OptimizationRule::ScanWithFilterOptimization,
            OptimizationRule::IndexFullScan,
            OptimizationRule::IndexScan,
            OptimizationRule::EdgeIndexFullScan,
            OptimizationRule::TagIndexFullScan,
            OptimizationRule::UnionAllEdgeIndexScan,
            OptimizationRule::UnionAllTagIndexScan,
            OptimizationRule::OptimizeEdgeIndexScanByFilter,
            OptimizationRule::OptimizeTagIndexScanByFilter,
            OptimizationRule::RemoveUselessNode,
        ].into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_enabled() {
        let config = RuleConfig::default();
        assert!(config.is_enabled(OptimizationRule::FilterPushDown));
        assert!(config.is_enabled(OptimizationRule::CollapseProject));
        assert!(!config.is_enabled(OptimizationRule::RemoveUselessNode));
    }
    
    #[test]
    fn test_disable_rule() {
        let mut config = RuleConfig::default();
        config.disable(OptimizationRule::FilterPushDown);
        assert!(!config.is_enabled(OptimizationRule::FilterPushDown));
    }
    
    #[test]
    fn test_enable_rule() {
        let mut config = RuleConfig::default();
        config.enable(OptimizationRule::RemoveUselessNode);
        assert!(config.is_enabled(OptimizationRule::RemoveUselessNode));
    }
    
    #[test]
    fn test_enabled_rules() {
        let mut config = RuleConfig::default();
        config.disable(OptimizationRule::FilterPushDown);
        config.disable(OptimizationRule::TopN);
        
        let enabled = config.get_enabled_rules();
        assert!(!enabled.contains(&OptimizationRule::FilterPushDown));
        assert!(!enabled.contains(&OptimizationRule::TopN));
    }
}
