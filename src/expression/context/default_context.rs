//! 默认表达式上下文实现
//!
//! 提供表达式求值过程中的上下文管理

use crate::core::Value;
use crate::expression::context::cache_manager::CacheManager;
use crate::expression::functions::global_registry_ref;
use std::collections::HashMap;

/// 表达式上下文
///
/// 提供表达式求值所需的上下文环境，包括：
/// - 变量存储
/// - 函数注册（使用全局函数注册表）
/// - 正则缓存
#[derive(Debug)]
pub struct DefaultExpressionContext {
    /// 变量存储
    variables: HashMap<String, Value>,
    /// 缓存管理器
    cache_manager: CacheManager,
}

impl DefaultExpressionContext {
    /// 创建新的上下文
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            cache_manager: CacheManager::new(),
        }
    }

    /// 添加变量
    pub fn add_variable(mut self, name: String, value: Value) -> Self {
        self.variables.insert(name, value);
        self
    }

    /// 批量添加变量
    pub fn with_variables<I>(mut self, variables: I) -> Self
    where
        I: IntoIterator<Item = (String, Value)>,
    {
        for (name, value) in variables {
            self.variables.insert(name, value);
        }
        self
    }
}

impl Default for DefaultExpressionContext {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::expression::evaluator::traits::ExpressionContext for DefaultExpressionContext {
    fn get_variable(&self, name: &str) -> Option<Value> {
        self.variables.get(name).cloned()
    }

    fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    fn get_function(&self, name: &str) -> Option<crate::expression::functions::FunctionRef> {
        let registry = global_registry_ref();
        registry.get_builtin(name)
            .map(|f| crate::expression::functions::FunctionRef::Builtin(f))
            .or_else(|| registry.get_custom(name).map(|f| crate::expression::functions::FunctionRef::Custom(f)))
    }

    fn supports_cache(&self) -> bool {
        true
    }

    fn get_cache(&mut self) -> Option<&mut CacheManager> {
        Some(&mut self.cache_manager)
    }
}
