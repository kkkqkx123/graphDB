//! 规则注册机制
//! 提供静态规则注册表，支持规则发现和注册
//!
//! 参考 nebula-graph 的实现，使用静态初始化避免运行时锁竞争

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use crate::query::optimizer::{OptRule, OptimizationRule};

type RuleCreator = Arc<dyn Fn() -> Box<dyn OptRule> + Send + Sync>;

static RULE_REGISTRY: OnceLock<Mutex<HashMap<OptimizationRule, RuleCreator>>> = OnceLock::new();
static INITIALIZED: OnceLock<bool> = OnceLock::new();

pub struct RuleRegistry;

impl RuleRegistry {
    pub fn register<F>(rule: OptimizationRule, creator: F)
    where
        F: Fn() -> Box<dyn OptRule> + Send + Sync + 'static,
    {
        let registry = RULE_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()));
        let mut writer = registry.lock().unwrap();
        writer.insert(rule, Arc::new(creator));
    }

    pub fn get(rule: OptimizationRule) -> Option<RuleCreator> {
        let registry = RULE_REGISTRY.get()?;
        let reader = registry.lock().unwrap();
        reader.get(&rule).cloned()
    }

    pub fn create_instance(rule: OptimizationRule) -> Option<Box<dyn OptRule>> {
        Self::get(rule).map(|creator| creator())
    }

    pub fn get_all_rules() -> Vec<OptimizationRule> {
        if let Some(registry) = RULE_REGISTRY.get() {
            let reader = registry.lock().unwrap();
            return reader.keys().copied().collect();
        }
        Vec::new()
    }

    pub fn get_rules_by_phase(phase: crate::query::optimizer::OptimizationPhase) -> Vec<OptimizationRule> {
        if let Some(registry) = RULE_REGISTRY.get() {
            let reader = registry.lock().unwrap();
            return reader
                .keys()
                .filter(|r| r.phase() == phase)
                .copied()
                .collect();
        }
        Vec::new()
    }

    pub fn is_registered(rule: OptimizationRule) -> bool {
        if let Some(registry) = RULE_REGISTRY.get() {
            let reader = registry.lock().unwrap();
            return reader.contains_key(&rule);
        }
        false
    }

    pub fn count() -> usize {
        if let Some(registry) = RULE_REGISTRY.get() {
            let reader = registry.lock().unwrap();
            return reader.len();
        }
        0
    }

    pub fn is_initialized() -> bool {
        INITIALIZED.get().copied().unwrap_or(false)
    }

    pub fn initialize() {
        if INITIALIZED.get().copied().unwrap_or(false) {
            return;
        }

        let registry = RULE_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()));
        let writer = registry.lock().unwrap();

        if writer.is_empty() {
            drop(writer);
            crate::query::optimizer::rule_registrar::register_all_rules();
        }

        INITIALIZED.set(true).ok();
    }
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
    use crate::query::optimizer::{BaseOptRule, OptContext, OptGroupNode, OptimizerError, OptRule, TransformResult};
    use crate::query::optimizer::plan::Pattern;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::result::Result as StdResult;

    #[derive(Debug)]
    struct TestRule;

    impl OptRule for TestRule {
        fn name(&self) -> &str {
            "TestRule"
        }

        fn apply(
            &self,
            _ctx: &mut OptContext,
            _group_node: &Rc<RefCell<OptGroupNode>>,
        ) -> StdResult<Option<TransformResult>, OptimizerError> {
            Ok(Some(TransformResult::unchanged()))
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
        assert!(RuleRegistry::create_instance(OptimizationRule::ConstantFolding).is_none());
    }

    #[test]
    fn test_get_all_rules() {
        let rules = RuleRegistry::get_all_rules();
        assert!(!rules.is_empty());
    }
}
