//! 缓存管理器模块
//!
//! 管理表达式求值过程中的各种缓存，如正则表达式缓存等

use regex::Regex;
use std::collections::HashMap;

/// 缓存管理器
///
/// 管理表达式求值过程中的各种缓存
#[derive(Debug, Clone)]
pub struct CacheManager {
    /// 正则表达式缓存
    regex_cache: HashMap<String, Regex>,
    /// 其他缓存（泛型缓存）
    generic_cache: HashMap<String, String>,
}

impl CacheManager {
    /// 创建新的缓存管理器
    pub fn new() -> Self {
        Self {
            regex_cache: HashMap::new(),
            generic_cache: HashMap::new(),
        }
    }

    /// 获取或编译正则表达式（内部方法）
    pub fn get_regex_internal(&mut self, pattern: &str) -> Option<&Regex> {
        if !self.regex_cache.contains_key(pattern) {
            if let Ok(regex) = Regex::new(pattern) {
                self.regex_cache.insert(pattern.to_string(), regex);
            } else {
                return None;
            }
        }
        self.regex_cache.get(pattern)
    }

    /// 预编译正则表达式
    pub fn compile_regex(&mut self, pattern: String) -> Result<(), String> {
        if self.regex_cache.contains_key(&pattern) {
            return Ok(());
        }

        let regex = Regex::new(&pattern).map_err(|e| e.to_string())?;
        self.regex_cache.insert(pattern, regex);
        Ok(())
    }

    /// 检查正则表达式是否已缓存
    pub fn has_regex(&self, pattern: &str) -> bool {
        self.regex_cache.contains_key(pattern)
    }

    /// 移除正则表达式缓存
    pub fn remove_regex(&mut self, pattern: &str) -> Option<Regex> {
        self.regex_cache.remove(pattern)
    }

    /// 获取泛型缓存值
    pub fn get_generic(&self, key: &str) -> Option<&String> {
        self.generic_cache.get(key)
    }

    /// 设置泛型缓存值
    pub fn set_generic(&mut self, key: String, value: String) {
        self.generic_cache.insert(key, value);
    }

    /// 检查泛型缓存是否存在
    pub fn has_generic(&self, key: &str) -> bool {
        self.generic_cache.contains_key(key)
    }

    /// 移除泛型缓存
    pub fn remove_generic(&mut self, key: &str) -> Option<String> {
        self.generic_cache.remove(key)
    }

    /// 获取正则表达式缓存数量
    pub fn regex_count(&self) -> usize {
        self.regex_cache.len()
    }

    /// 获取泛型缓存数量
    pub fn generic_count(&self) -> usize {
        self.generic_cache.len()
    }

    /// 获取总缓存数量
    pub fn total_count(&self) -> usize {
        self.regex_cache.len() + self.generic_cache.len()
    }

    /// 清空正则表达式缓存
    pub fn clear_regex(&mut self) {
        self.regex_cache.clear();
    }

    /// 清空泛型缓存
    pub fn clear_generic(&mut self) {
        self.generic_cache.clear();
    }

    /// 清空所有缓存
    pub fn clear(&mut self) {
        self.regex_cache.clear();
        self.generic_cache.clear();
    }

    /// 获取所有缓存的正则表达式模式
    pub fn regex_patterns(&self) -> Vec<&str> {
        self.regex_cache.keys().map(|k| k.as_str()).collect()
    }

    /// 获取所有泛型缓存的键
    pub fn generic_keys(&self) -> Vec<&str> {
        self.generic_cache.keys().map(|k| k.as_str()).collect()
    }
}

impl crate::expression::context::traits::CacheContext for CacheManager {
    fn get_regex(&mut self, pattern: &str) -> Option<&regex::Regex> {
        self.get_regex_internal(pattern)
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_manager_regex() {
        let mut cache = CacheManager::new();

        let regex = cache.get_regex_internal(r"\d+");
        assert!(regex.is_some());
        assert!(regex.expect("Expected regex to be valid").is_match("123"));
        assert_eq!(cache.regex_count(), 1);

        let regex2 = cache.get_regex_internal(r"\d+");
        assert!(regex2.is_some());
        assert_eq!(cache.regex_count(), 1);
    }

    #[test]
    fn test_cache_manager_regex_invalid() {
        let mut cache = CacheManager::new();

        let regex = cache.get_regex_internal(r"[invalid");
        assert!(regex.is_none());
        assert_eq!(cache.regex_count(), 0);
    }

    #[test]
    fn test_cache_manager_generic() {
        let mut cache = CacheManager::new();

        cache.set_generic("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get_generic("key1"), Some(&"value1".to_string()));
        assert_eq!(cache.generic_count(), 1);

        let removed = cache.remove_generic("key1");
        assert_eq!(removed, Some("value1".to_string()));
        assert_eq!(cache.generic_count(), 0);
    }

    #[test]
    fn test_cache_manager_clear() {
        let mut cache = CacheManager::new();

        cache.get_regex_internal(r"\d+");
        cache.set_generic("key1".to_string(), "value1".to_string());

        assert_eq!(cache.total_count(), 2);

        cache.clear_regex();
        assert_eq!(cache.total_count(), 1);

        cache.clear_generic();
        assert_eq!(cache.total_count(), 0);

        cache.get_regex_internal(r"\d+");
        cache.set_generic("key1".to_string(), "value1".to_string());

        cache.clear();
        assert_eq!(cache.total_count(), 0);
    }

    #[test]
    fn test_cache_manager_patterns() {
        let mut cache = CacheManager::new();

        cache.get_regex_internal(r"\d+");
        cache.get_regex_internal(r"[a-z]+");
        cache.get_regex_internal(r"\w+");

        let patterns = cache.regex_patterns();
        assert_eq!(patterns.len(), 3);
        assert!(patterns.contains(&r"\d+"));
        assert!(patterns.contains(&r"[a-z]+"));
        assert!(patterns.contains(&r"\w+"));
    }
}
