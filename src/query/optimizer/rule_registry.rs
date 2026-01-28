//! 规则注册机制
//! 提供静态规则注册表，支持规则发现和注册

use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

use crate::query::optimizer::{OptRule, OptimizationRule};

type RuleCreator = Arc<dyn Fn() -> Box<dyn OptRule> + Send + Sync>;

static RULE_REGISTRY: OnceLock<RwLock<HashMap<OptimizationRule, RuleCreator>>> = OnceLock::new();

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
        let registry = get_registry();
        let reader = registry.read().unwrap();
        reader.get(&rule).map(|creator| creator())
    }
    
    pub fn get_all_rules() -> Vec<OptimizationRule> {
        let registry = get_registry();
        let reader = registry.read().unwrap();
        reader.keys().copied().collect()
    }
    
    pub fn get_rules_by_phase(phase: crate::query::optimizer::OptimizationPhase) -> Vec<OptimizationRule> {
        let registry = get_registry();
        let reader = registry.read().unwrap();
        reader
            .keys()
            .filter(|r| r.phase() == phase)
            .copied()
            .collect()
    }
    
    pub fn is_registered(rule: OptimizationRule) -> bool {
        let registry = get_registry();
        let reader = registry.read().unwrap();
        reader.contains_key(&rule)
    }
    
    pub fn count() -> usize {
        let registry = get_registry();
        let reader = registry.read().unwrap();
        reader.len()
    }
}

fn get_registry() -> &'static RwLock<HashMap<OptimizationRule, RuleCreator>> {
    RULE_REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
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
    use crate::query::optimizer::{BaseOptRule, OptRule};
    
    struct TestRule;
    
    impl BaseOptRule for TestRule {
        fn name(&self) -> &str {
            "TestRule"
        }
        
        fn transform(
            &self,
            ctx: &mut dyn crate::query::optimizer::OptContext,
        ) -> Option<crate::query::optimizer::TransformResult> {
            None
        }
    }
    
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
        assert!(RuleRegistry::create_instance(OptimizationRule::RemoveUselessNode).is_none());
    }
    
    #[test]
    fn test_get_all_rules() {
        let rules = RuleRegistry::get_all_rules();
        assert!(!rules.is_empty());
    }
}
