//! 版本管理模块
//!
//! 提供变量历史版本管理功能，支持多版本变量存储和查询

use crate::core::Value;
use std::collections::HashMap;

/// 版本管理器
///
/// 管理变量的历史版本，支持版本查询
#[derive(Debug, Clone)]
pub struct VersionManager {
    /// 变量名 -> 历史版本列表（最新版本在前面）
    versions: HashMap<String, Vec<Value>>,
}

impl VersionManager {
    /// 创建新的版本管理器
    pub fn new() -> Self {
        Self {
            versions: HashMap::new(),
        }
    }

    /// 获取最新版本 (version = 0)
    pub fn get_latest(&self, name: &str) -> Option<&Value> {
        self.get_version(name, 0)
    }

    /// 获取指定版本
    ///
    /// version = 0: 最新版本
    /// version = -1: 前一个版本
    /// version = 1: 最老版本
    /// version = 2: 第二老的版本
    pub fn get_version(&self, name: &str, version: i64) -> Option<&Value> {
        let history = self.versions.get(name)?;
        let len = history.len();

        if len == 0 {
            return None;
        }

        let idx = if version <= 0 {
            // 负数或0：从最新开始
            let abs = version.abs() as usize;
            if abs >= len {
                None
            } else {
                Some(abs)
            }
        } else {
            // 正数：从最老开始
            let idx_from_end = len as i64 - version;
            if idx_from_end < 0 {
                None
            } else {
                Some(idx_from_end as usize)
            }
        };

        idx.and_then(|i| history.get(i))
    }

    /// 设置新版本（追加到历史）
    pub fn set_version(&mut self, name: String, value: Value) {
        self.versions.entry(name).or_insert_with(Vec::new).insert(0, value);
    }

    /// 设置指定版本（替换）
    pub fn replace_version(&mut self, name: String, value: Value, version: i64) -> bool {
        let history = self.versions.entry(name).or_insert_with(Vec::new);
        let len = history.len();

        if len == 0 {
            return false;
        }

        let idx = if version <= 0 {
            let abs = version.abs() as usize;
            if abs >= len {
                return false;
            }
            abs
        } else {
            let idx_from_end = len as i64 - version;
            if idx_from_end < 0 {
                return false;
            }
            idx_from_end as usize
        };

        if let Some(slot) = history.get_mut(idx) {
            *slot = value;
            true
        } else {
            false
        }
    }

    /// 获取版本数量
    pub fn version_count(&self, name: &str) -> usize {
        self.versions.get(name).map(|v| v.len()).unwrap_or(0)
    }

    /// 获取所有版本历史
    pub fn get_history(&self, name: &str) -> Option<&[Value]> {
        self.versions.get(name).map(|v| v.as_slice())
    }

    /// 清空指定变量的历史
    pub fn clear_history(&mut self, name: &str) {
        self.versions.remove(name);
    }

    /// 截断历史，只保留最后 N 个版本
    pub fn truncate_history(&mut self, name: &str, num_versions_to_keep: usize) {
        if let Some(history) = self.versions.get_mut(name) {
            if history.len() > num_versions_to_keep {
                history.truncate(num_versions_to_keep);
            }
        }
    }

    /// 检查变量是否存在
    pub fn exists(&self, name: &str) -> bool {
        self.versions.contains_key(name)
    }

    /// 获取所有变量名
    pub fn variable_names(&self) -> Vec<&str> {
        self.versions.keys().map(|k| k.as_str()).collect()
    }

    /// 移除变量
    pub fn remove(&mut self, name: &str) -> Option<Vec<Value>> {
        self.versions.remove(name)
    }

    /// 清空所有变量
    pub fn clear(&mut self) {
        self.versions.clear();
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
    fn test_version_manager_basic() {
        let mut vm = VersionManager::new();

        vm.set_version("x".to_string(), Value::Int(1));
        vm.set_version("x".to_string(), Value::Int(2));
        vm.set_version("x".to_string(), Value::Int(3));

        assert_eq!(vm.get_latest("x"), Some(&Value::Int(3)));
        assert_eq!(vm.get_version("x", 0), Some(&Value::Int(3)));
        assert_eq!(vm.get_version("x", -1), Some(&Value::Int(2)));
        assert_eq!(vm.get_version("x", -2), Some(&Value::Int(1)));
        assert_eq!(vm.get_version("x", 1), Some(&Value::Int(1)));
        assert_eq!(vm.get_version("x", 2), Some(&Value::Int(2)));
        assert_eq!(vm.version_count("x"), 3);
    }

    #[test]
    fn test_version_manager_truncate() {
        let mut vm = VersionManager::new();

        for i in 1..=10 {
            vm.set_version("x".to_string(), Value::Int(i));
        }

        vm.truncate_history("x", 3);
        assert_eq!(vm.version_count("x"), 3);
        assert_eq!(vm.get_latest("x"), Some(&Value::Int(10)));
    }

    #[test]
    fn test_version_manager_replace() {
        let mut vm = VersionManager::new();

        vm.set_version("x".to_string(), Value::Int(1));
        vm.set_version("x".to_string(), Value::Int(2));

        let replaced = vm.replace_version("x".to_string(), Value::Int(100), 0);
        assert!(replaced);
        assert_eq!(vm.get_latest("x"), Some(&Value::Int(100)));
    }

    #[test]
    fn test_version_manager_clear() {
        let mut vm = VersionManager::new();

        vm.set_version("x".to_string(), Value::Int(1));
        vm.set_version("y".to_string(), Value::Int(2));

        vm.clear_history("x");
        assert!(!vm.exists("x"));
        assert!(vm.exists("y"));

        vm.clear();
        assert!(!vm.exists("y"));
    }
}
