//! 规则配置和启用控制
//! 支持通过配置启用或禁用特定优化规则
//!
//! # 说明
//!
//! 该配置仅管理基于代价的优化规则（CBO）。
//! 启发式规则（如谓词下推、投影下推等）已迁移到 planner/rewrite 模块，
//! 在计划生成阶段直接应用，不再通过此配置管理。

use std::collections::HashSet;

use super::rule_enum::OptimizationRule;

/// 规则配置
///
/// 使用 HashSet 存储禁用的规则，简化设计：
/// - 不在集合中的规则默认为启用
/// - 提供简洁的 API 控制规则开关
#[derive(Debug, Clone, Default)]
pub struct RuleConfig {
    /// 禁用的规则集合
    disabled_rules: HashSet<OptimizationRule>,
}

impl RuleConfig {
    /// 创建默认配置（所有规则启用）
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建配置并禁用指定规则
    pub fn with_disabled(rules: &[OptimizationRule]) -> Self {
        let mut config = Self::new();
        for &rule in rules {
            config.disable(rule);
        }
        config
    }

    /// 检查规则是否启用
    pub fn is_enabled(&self, rule: OptimizationRule) -> bool {
        !self.disabled_rules.contains(&rule)
    }

    /// 根据规则名称检查是否启用
    pub fn is_enabled_by_name(&self, rule_name: &str) -> bool {
        OptimizationRule::from_name(rule_name)
            .map(|rule| self.is_enabled(rule))
            .unwrap_or(true)
    }

    /// 启用规则
    pub fn enable(&mut self, rule: OptimizationRule) {
        self.disabled_rules.remove(&rule);
    }

    /// 禁用规则
    pub fn disable(&mut self, rule: OptimizationRule) {
        self.disabled_rules.insert(rule);
    }

    /// 切换规则状态
    pub fn toggle(&mut self, rule: OptimizationRule) {
        if self.is_enabled(rule) {
            self.disable(rule);
        } else {
            self.enable(rule);
        }
    }

    /// 获取所有启用的规则
    pub fn get_enabled_rules(&self) -> Vec<OptimizationRule> {
        OptimizationRule::all_cbo_rules()
            .into_iter()
            .filter(|&rule| self.is_enabled(rule))
            .collect()
    }

    /// 获取所有禁用的规则
    pub fn get_disabled_rules(&self) -> Vec<OptimizationRule> {
        self.disabled_rules.iter().copied().collect()
    }

    /// 重置为默认状态（所有规则启用）
    pub fn reset(&mut self) {
        self.disabled_rules.clear();
    }

    /// 检查是否有任何规则被禁用
    pub fn has_disabled_rules(&self) -> bool {
        !self.disabled_rules.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_all_enabled() {
        let config = RuleConfig::default();

        // 所有规则默认启用
        for rule in OptimizationRule::all_cbo_rules() {
            assert!(config.is_enabled(rule), "规则 {:?} 应该默认启用", rule);
        }
    }

    #[test]
    fn test_disable_rule() {
        let mut config = RuleConfig::default();

        config.disable(OptimizationRule::TopN);
        assert!(!config.is_enabled(OptimizationRule::TopN));
        assert!(config.is_enabled(OptimizationRule::JoinOptimization));
    }

    #[test]
    fn test_enable_rule() {
        let mut config = RuleConfig::default();

        config.disable(OptimizationRule::TopN);
        assert!(!config.is_enabled(OptimizationRule::TopN));

        config.enable(OptimizationRule::TopN);
        assert!(config.is_enabled(OptimizationRule::TopN));
    }

    #[test]
    fn test_toggle_rule() {
        let mut config = RuleConfig::default();

        // 初始启用
        assert!(config.is_enabled(OptimizationRule::TopN));

        // 切换为禁用
        config.toggle(OptimizationRule::TopN);
        assert!(!config.is_enabled(OptimizationRule::TopN));

        // 切换回启用
        config.toggle(OptimizationRule::TopN);
        assert!(config.is_enabled(OptimizationRule::TopN));
    }

    #[test]
    fn test_get_enabled_rules() {
        let mut config = RuleConfig::default();

        config.disable(OptimizationRule::TopN);
        config.disable(OptimizationRule::JoinOptimization);

        let enabled = config.get_enabled_rules();
        assert!(!enabled.contains(&OptimizationRule::TopN));
        assert!(!enabled.contains(&OptimizationRule::JoinOptimization));

        // 其他规则仍然启用
        assert!(enabled.contains(&OptimizationRule::IndexScan));
    }

    #[test]
    fn test_get_disabled_rules() {
        let mut config = RuleConfig::default();

        config.disable(OptimizationRule::TopN);
        config.disable(OptimizationRule::JoinOptimization);

        let disabled = config.get_disabled_rules();
        assert!(disabled.contains(&OptimizationRule::TopN));
        assert!(disabled.contains(&OptimizationRule::JoinOptimization));
        assert_eq!(disabled.len(), 2);
    }

    #[test]
    fn test_with_disabled() {
        let config = RuleConfig::with_disabled(&[
            OptimizationRule::TopN,
            OptimizationRule::JoinOptimization,
        ]);

        assert!(!config.is_enabled(OptimizationRule::TopN));
        assert!(!config.is_enabled(OptimizationRule::JoinOptimization));
        assert!(config.is_enabled(OptimizationRule::IndexScan));
    }

    #[test]
    fn test_reset() {
        let mut config = RuleConfig::default();

        config.disable(OptimizationRule::TopN);
        assert!(!config.is_enabled(OptimizationRule::TopN));

        config.reset();
        assert!(config.is_enabled(OptimizationRule::TopN));
        assert!(!config.has_disabled_rules());
    }

    #[test]
    fn test_is_enabled_by_name() {
        let mut config = RuleConfig::default();

        assert!(config.is_enabled_by_name("TopNRule"));

        config.disable(OptimizationRule::TopN);
        assert!(!config.is_enabled_by_name("TopNRule"));

        // 未知规则名称默认启用
        assert!(config.is_enabled_by_name("UnknownRule"));
    }
}
