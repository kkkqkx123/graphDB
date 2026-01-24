//! 函数注册表模块
//!
//! 管理内置函数和自定义函数的注册与查找

use crate::expression::functions::{
    BuiltinFunction, CustomFunction, ExpressionFunction, FunctionRef,
};
use std::collections::HashMap;

/// 函数注册表
///
/// 管理内置函数和自定义函数的注册与查找
#[derive(Debug, Clone)]
pub struct FunctionRegistry {
    /// 内置函数注册表
    builtin_functions: HashMap<String, BuiltinFunction>,
    /// 自定义函数注册表
    custom_functions: HashMap<String, CustomFunction>,
}

impl FunctionRegistry {
    /// 创建新的函数注册表
    pub fn new() -> Self {
        Self {
            builtin_functions: HashMap::new(),
            custom_functions: HashMap::new(),
        }
    }

    /// 注册内置函数
    pub fn register_builtin(&mut self, function: BuiltinFunction) {
        self.builtin_functions.insert(function.name().to_string(), function);
    }

    /// 注册自定义函数
    pub fn register_custom(&mut self, function: CustomFunction) {
        self.custom_functions.insert(function.name.clone(), function);
    }

    /// 获取函数
    pub fn get(&self, name: &str) -> Option<FunctionRef> {
        if let Some(function) = self.builtin_functions.get(name) {
            return Some(FunctionRef::Builtin(function));
        }

        if let Some(function) = self.custom_functions.get(name) {
            return Some(FunctionRef::Custom(function));
        }

        None
    }

    /// 获取内置函数
    pub fn get_builtin(&self, name: &str) -> Option<&BuiltinFunction> {
        self.builtin_functions.get(name)
    }

    /// 获取自定义函数
    pub fn get_custom(&self, name: &str) -> Option<&CustomFunction> {
        self.custom_functions.get(name)
    }

    /// 检查函数是否存在
    pub fn exists(&self, name: &str) -> bool {
        self.builtin_functions.contains_key(name) || self.custom_functions.contains_key(name)
    }

    /// 检查是否为内置函数
    pub fn is_builtin(&self, name: &str) -> bool {
        self.builtin_functions.contains_key(name)
    }

    /// 检查是否为自定义函数
    pub fn is_custom(&self, name: &str) -> bool {
        self.custom_functions.contains_key(name)
    }

    /// 移除函数
    pub fn remove(&mut self, name: &str) -> bool {
        self.builtin_functions.remove(name).is_some() || self.custom_functions.remove(name).is_some()
    }

    /// 获取所有函数名
    pub fn function_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.builtin_functions.keys().map(|k| k.as_str()).collect();
        names.extend(self.custom_functions.keys().map(|k| k.as_str()));
        names.sort();
        names.dedup();
        names
    }

    /// 获取内置函数数量
    pub fn builtin_count(&self) -> usize {
        self.builtin_functions.len()
    }

    /// 获取自定义函数数量
    pub fn custom_count(&self) -> usize {
        self.custom_functions.len()
    }

    /// 获取总函数数量
    pub fn total_count(&self) -> usize {
        self.builtin_count() + self.custom_count()
    }

    /// 清空所有函数
    pub fn clear(&mut self) {
        self.builtin_functions.clear();
        self.custom_functions.clear();
    }

    /// 清空内置函数
    pub fn clear_builtin(&mut self) {
        self.builtin_functions.clear();
    }

    /// 清空自定义函数
    pub fn clear_custom(&mut self) {
        self.custom_functions.clear();
    }

    /// 合并另一个注册表
    pub fn merge(&mut self, other: FunctionRegistry) {
        for (name, func) in other.builtin_functions {
            self.builtin_functions.entry(name).or_insert(func);
        }
        for (name, func) in other.custom_functions {
            self.custom_functions.entry(name).or_insert(func);
        }
    }

    /// 获取所有内置函数
    pub fn get_all_builtins(&self) -> Vec<&BuiltinFunction> {
        self.builtin_functions.values().collect()
    }

    /// 获取所有自定义函数
    pub fn get_all_customs(&self) -> Vec<&CustomFunction> {
        self.custom_functions.values().collect()
    }
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_registry_basic() {
        let mut registry = FunctionRegistry::new();

        let builtin = BuiltinFunction::Math(crate::expression::functions::MathFunction::Abs);
        registry.register_builtin(builtin);

        assert!(registry.exists("abs"));
        assert!(registry.is_builtin("abs"));
        assert_eq!(registry.builtin_count(), 1);
    }

    #[test]
    fn test_function_registry_custom() {
        let mut registry = FunctionRegistry::new();

        let custom = CustomFunction {
            name: "my_func".to_string(),
            arity: 0,
            is_variadic: false,
            description: "测试函数".to_string(),
            function_id: 1,
        };
        registry.register_custom(custom);

        assert!(registry.exists("my_func"));
        assert!(registry.is_custom("my_func"));
        assert_eq!(registry.custom_count(), 1);
    }

    #[test]
    fn test_function_registry_remove() {
        let mut registry = FunctionRegistry::new();

        let builtin = BuiltinFunction::Math(crate::expression::functions::MathFunction::Abs);
        registry.register_builtin(builtin);

        let removed = registry.remove("abs");
        assert!(removed);
        assert!(!registry.exists("abs"));
    }

    #[test]
    fn test_function_registry_merge() {
        let mut registry1 = FunctionRegistry::new();
        let mut registry2 = FunctionRegistry::new();

        let builtin1 = BuiltinFunction::Math(crate::expression::functions::MathFunction::Abs);
        let builtin2 = BuiltinFunction::Math(crate::expression::functions::MathFunction::Sqrt);

        registry1.register_builtin(builtin1);
        registry2.register_builtin(builtin2);

        registry1.merge(registry2);

        assert!(registry1.exists("abs"));
        assert!(registry1.exists("sqrt"));
        assert_eq!(registry1.total_count(), 2);
    }

    #[test]
    fn test_function_registry_clear() {
        let mut registry = FunctionRegistry::new();

        let builtin = BuiltinFunction::Math(crate::expression::functions::MathFunction::Abs);
        let custom = CustomFunction {
            name: "my_func".to_string(),
            arity: 0,
            is_variadic: false,
            description: "测试函数".to_string(),
            function_id: 1,
        };

        registry.register_builtin(builtin);
        registry.register_custom(custom);

        registry.clear_builtin();
        assert!(!registry.exists("abs"));
        assert!(registry.exists("my_func"));

        registry.clear_custom();
        assert!(!registry.exists("my_func"));
    }
}
