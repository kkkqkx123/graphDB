//! 规则注册机制
//! 提供静态规则注册表，支持规则发现和注册

use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

use crate::query::optimizer::{OptRule, OptimizationRule};

type RuleCreator = Arc<dyn Fn() -> Box<dyn OptRule> + Send + Sync>;

static RULE_REGISTRY: OnceLock<RwLock<HashMap<OptimizationRule, RuleCreator>>> = OnceLock::new();
static RULES_INITIALIZED: OnceLock<bool> = OnceLock::new();

pub struct RuleRegistry;

impl RuleRegistry {
    pub fn register<F>(rule: OptimizationRule, creator: F)
    where
        F: Fn() -> Box<dyn OptRule> + Send + Sync + 'static,
    {
        let registry = get_registry();
        let mut writer = registry.write().unwrap();
        writer.insert(rule, Arc::new(creator));
    }

    pub fn get(rule: OptimizationRule) -> Option<RuleCreator> {
        let registry = get_registry();
        let reader = registry.read().unwrap();
        reader.get(&rule).cloned()
    }

    pub fn create_instance(rule: OptimizationRule) -> Option<Box<dyn OptRule>> {
        ensure_rules_initialized();
        let registry = get_registry();
        let reader = registry.read().unwrap();
        reader.get(&rule).map(|creator| creator())
    }

    pub fn get_all_rules() -> Vec<OptimizationRule> {
        ensure_rules_initialized();
        let registry = get_registry();
        let reader = registry.read().unwrap();
        reader.keys().copied().collect()
    }

    pub fn get_rules_by_phase(phase: crate::query::optimizer::OptimizationPhase) -> Vec<OptimizationRule> {
        ensure_rules_initialized();
        let registry = get_registry();
        let reader = registry.read().unwrap();
        reader
            .keys()
            .filter(|r| r.phase() == phase)
            .copied()
            .collect()
    }

    pub fn is_registered(rule: OptimizationRule) -> bool {
        ensure_rules_initialized();
        let registry = get_registry();
        let reader = registry.read().unwrap();
        reader.contains_key(&rule)
    }

    pub fn count() -> usize {
        ensure_rules_initialized();
        let registry = get_registry();
        let reader = registry.read().unwrap();
        reader.len()
    }

    pub fn is_initialized() -> bool {
        RULES_INITIALIZED.get().copied().unwrap_or(false)
    }
}

fn get_registry() -> &'static RwLock<HashMap<OptimizationRule, RuleCreator>> {
    RULE_REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

fn ensure_rules_initialized() {
    if RULES_INITIALIZED.get().copied().unwrap_or(false) {
        return;
    }

    let mut registry = get_registry().write().unwrap();
    if registry.is_empty() {
        crate::query::optimizer::rule_registrar::register_all_rules();
    }
    RULES_INITIALIZED.set(true).ok();
}

#[macro_export]
macro_rules! register_rule {
    ($rule:expr, $creator:expr) => {
        $crate::query::optimizer::rule_registry::RuleRegistry::register($rule, $creator);
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::optimizer::{BaseOptRule, OptContext, OptGroupNode, OptimizerError, OptRule};
    use crate::query::optimizer::plan::Pattern;
    
    #[derive(Debug)]
    struct TestRule;
    
    impl OptRule for TestRule {
        fn name(&self) -> &str {
            "TestRule"
        }
        
        fn apply(
            &self,
            ctx: &mut OptContext,
            group_node: &OptGroupNode,
        ) -> Result<Option<OptGroupNode>, OptimizerError> {
            Ok(None)
        }
        
        fn pattern(&self) -> Pattern {
            Pattern::new("Test")
        }
    }
    
    impl BaseOptRule for TestRule {}
    
    #[test]
    fn test_register_and_get() {
        RuleRegistry::register(
            OptimizationRule::FilterPushDown,
            || Box::new(TestRule) as Box<dyn OptRule>,
        );
        
        assert!(RuleRegistry::is_registered(OptimizationRule::FilterPushDown));
        assert!(RuleRegistry::create_instance(OptimizationRule::FilterPushDown).is_some());
    }
    
    #[test]
    fn test_get_unregistered() {
        assert!(RuleRegistry::create_instance(OptimizationRule::TopN).is_none());
    }
    
    #[test]
    fn test_get_all_rules() {
        let rules = RuleRegistry::get_all_rules();
        assert!(!rules.is_empty());
    }
}
