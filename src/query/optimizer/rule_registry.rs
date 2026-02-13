//! 规则注册机制
//! 提供静态规则注册表，支持规则发现和注册
//!
//! 参考 nebula-graph 的实现，使用静态初始化避免运行时锁竞争

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use crate::query::optimizer::{OptRule, OptimizationRule};
use crate::core::error::{DBError, LockError};

type RuleCreator = Arc<dyn Fn() -> Box<dyn OptRule> + Send + Sync>;

static RULE_REGISTRY: OnceLock<Mutex<HashMap<OptimizationRule, RuleCreator>>> = OnceLock::new();
static INITIALIZED: OnceLock<bool> = OnceLock::new();

pub struct RuleRegistry;

impl RuleRegistry {
    pub fn register<F>(rule: OptimizationRule, creator: F) -> Result<(), DBError>
    where
        F: Fn() -> Box<dyn OptRule> + Send + Sync + 'static,
    {
        let registry = RULE_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()));
        let mut writer = registry.lock().map_err(|e| {
            DBError::Lock(LockError::MutexPoisoned { reason: e.to_string() })
        })?;
        writer.insert(rule, Arc::new(creator));
        Ok(())
    }

    pub fn get(rule: OptimizationRule) -> Result<Option<RuleCreator>, DBError> {
        let registry = RULE_REGISTRY.get().ok_or_else(|| {
            DBError::Internal("Rule registry not initialized".to_string())
        })?;
        let reader = registry.lock().map_err(|e| {
            DBError::Lock(LockError::MutexPoisoned { reason: e.to_string() })
        })?;
        Ok(reader.get(&rule).cloned())
    }

    pub fn create_instance(rule: OptimizationRule) -> Result<Option<Box<dyn OptRule>>, DBError> {
        Self::get(rule).map(|creator| creator.map(|c| c()))
    }

    pub fn get_all_rules() -> Result<Vec<OptimizationRule>, DBError> {
        if let Some(registry) = RULE_REGISTRY.get() {
            let reader = registry.lock().map_err(|e| {
                DBError::Lock(LockError::MutexPoisoned { reason: e.to_string() })
            })?;
            return Ok(reader.keys().copied().collect());
        }
        Ok(Vec::new())
    }

    pub fn get_rules_by_phase(phase: crate::query::optimizer::OptimizationPhase) -> Result<Vec<OptimizationRule>, DBError> {
        if let Some(registry) = RULE_REGISTRY.get() {
            let reader = registry.lock().map_err(|e| {
                DBError::Lock(LockError::MutexPoisoned { reason: e.to_string() })
            })?;
            return Ok(reader
                .keys()
                .filter(|r| r.phase() == phase)
                .copied()
                .collect());
        }
        Ok(Vec::new())
    }

    pub fn is_registered(rule: OptimizationRule) -> Result<bool, DBError> {
        if let Some(registry) = RULE_REGISTRY.get() {
            let reader = registry.lock().map_err(|e| {
                DBError::Lock(LockError::MutexPoisoned { reason: e.to_string() })
            })?;
            return Ok(reader.contains_key(&rule));
        }
        Ok(false)
    }

    pub fn count() -> Result<usize, DBError> {
        if let Some(registry) = RULE_REGISTRY.get() {
            let reader = registry.lock().map_err(|e| {
                DBError::Lock(LockError::MutexPoisoned { reason: e.to_string() })
            })?;
            return Ok(reader.len());
        }
        Ok(0)
    }

    pub fn is_initialized() -> bool {
        INITIALIZED.get().copied().unwrap_or(false)
    }

    pub fn initialize() -> Result<(), DBError> {
        if INITIALIZED.get().copied().unwrap_or(false) {
            return Ok(());
        }

        let registry = RULE_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()));
        let writer = registry.lock().map_err(|e| {
            DBError::Lock(LockError::MutexPoisoned { reason: e.to_string() })
        })?;

        if writer.is_empty() {
            drop(writer);
            crate::query::optimizer::rule_registrar::register_all_rules();
        }

        INITIALIZED.set(true).ok();
        Ok(())
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
            Ok(None)
        }

        fn pattern(&self) -> Pattern {
            Pattern::new()
        }
    }

    impl BaseOptRule for TestRule {}

    #[test]
    fn test_register_and_get() {
        let _ = RuleRegistry::register(
            OptimizationRule::ProjectionPushDown,
            || Box::new(TestRule) as Box<dyn OptRule>,
        );

        assert!(RuleRegistry::is_registered(OptimizationRule::ProjectionPushDown).unwrap_or(false));
        assert!(RuleRegistry::create_instance(OptimizationRule::ProjectionPushDown).unwrap_or(None).is_some());
    }

    #[test]
    fn test_get_unregistered() {
        // 由于测试并行运行且共享全局状态，我们测试 is_registered 方法的行为
        // 如果规则未注册，is_registered 应该返回 false
        // 注意：这个测试假设 PushFilterDownAllPaths 规则可能未被注册
        // 如果测试失败，说明该规则已被其他测试注册
        let rule = OptimizationRule::PushFilterDownAllPaths;
        if RuleRegistry::is_registered(rule).unwrap_or(false) {
            // 如果已注册，验证我们可以获取实例
            assert!(RuleRegistry::create_instance(rule).unwrap_or(None).is_some());
        } else {
            // 如果未注册，验证返回 None
            assert!(RuleRegistry::create_instance(rule).unwrap_or(None).is_none());
        }
    }

    #[test]
    fn test_get_all_rules() {
        let _ = RuleRegistry::initialize();
        let rules = RuleRegistry::get_all_rules();
        assert!(!rules.unwrap_or_default().is_empty());
    }
}
