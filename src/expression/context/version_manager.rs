//! 变量管理模块
//!
//! 提供简单的变量存储功能

use crate::core::Value;
use std::collections::HashMap;

/// 变量管理器
///
/// 简单的变量存储，只保留最新值
#[derive(Debug, Clone)]
pub struct VersionManager {
    /// 变量名 -> 变量值
    variables: HashMap<String, Value>,
}

impl VersionManager {
    /// 创建新的变量管理器
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// 获取变量值
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    /// 设置变量值（直接替换）
    pub fn set(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    /// 检查变量是否存在
    pub fn exists(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// 获取所有变量名
    pub fn variable_names(&self) -> Vec<&str> {
        self.variables.keys().map(|k| k.as_str()).collect()
    }

    /// 移除变量
    pub fn remove(&mut self, name: &str) -> Option<Value> {
        self.variables.remove(name)
    }

    /// 清空所有变量
    pub fn clear(&mut self) {
        self.variables.clear();
    }

    /// 获取变量数量
    pub fn len(&self) -> usize {
        self.variables.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }
}

impl Default for VersionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_manager_basic() {
        let mut vm = VersionManager::new();

        vm.set("x".to_string(), Value::Int(1));
        assert_eq!(vm.get("x"), Some(&Value::Int(1)));

        vm.set("x".to_string(), Value::Int(2));
        assert_eq!(vm.get("x"), Some(&Value::Int(2)));

        vm.set("y".to_string(), Value::String("hello".to_string()));
        assert_eq!(vm.get("y"), Some(&Value::String("hello".to_string())));
    }

    #[test]
    fn test_variable_manager_remove() {
        let mut vm = VersionManager::new();

        vm.set("x".to_string(), Value::Int(1));
        assert!(vm.exists("x"));

        let removed = vm.remove("x");
        assert_eq!(removed, Some(Value::Int(1)));
        assert!(!vm.exists("x"));
    }

    #[test]
    fn test_variable_manager_clear() {
        let mut vm = VersionManager::new();

        vm.set("x".to_string(), Value::Int(1));
        vm.set("y".to_string(), Value::Int(2));

        vm.clear();
        assert!(vm.is_empty());
        assert_eq!(vm.len(), 0);
    }

    #[test]
    fn test_variable_manager_names() {
        let mut vm = VersionManager::new();

        vm.set("x".to_string(), Value::Int(1));
        vm.set("y".to_string(), Value::Int(2));

        let names = vm.variable_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"x"));
        assert!(names.contains(&"y"));
    }
}
