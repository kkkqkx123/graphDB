//! 缓存管理器模块
//!
//! 管理表达式求值过程中的各种缓存，包括正则表达式缓存、表达式解析缓存、日期时间解析缓存等

use regex::Regex;
use std::collections::HashMap;
use crate::core::types::expression::ExpressionMeta;
use crate::core::value::{DateValue, TimeValue, DateTimeValue};

/// 缓存管理器
///
/// 管理表达式求值过程中的各种缓存
#[derive(Debug, Clone)]
pub struct CacheManager {
    /// 正则表达式缓存
    regex_cache: HashMap<String, Regex>,
    /// 表达式解析缓存（表达式字符串 -> ExpressionMeta）
    expression_cache: HashMap<String, ExpressionMeta>,
    /// 日期解析缓存（日期字符串 -> DateValue）
    date_cache: HashMap<String, DateValue>,
    /// 时间解析缓存（时间字符串 -> TimeValue）
    time_cache: HashMap<String, TimeValue>,
    /// 日期时间解析缓存（日期时间字符串 -> DateTimeValue）
    datetime_cache: HashMap<String, DateTimeValue>,
}

impl CacheManager {
    /// 创建新的缓存管理器
    pub fn new() -> Self {
        Self {
            regex_cache: HashMap::new(),
            expression_cache: HashMap::new(),
            date_cache: HashMap::new(),
            time_cache: HashMap::new(),
            datetime_cache: HashMap::new(),
        }
    }

    // ==================== 正则表达式缓存 ====================

    /// 获取或编译正则表达式
    pub fn get_regex(&mut self, pattern: &str) -> Option<&Regex> {
        if !self.regex_cache.contains_key(pattern) {
            if let Ok(regex) = Regex::new(pattern) {
                self.regex_cache.insert(pattern.to_string(), regex);
            } else {
                return None;
            }
        }
        self.regex_cache.get(pattern)
    }

    /// 获取正则表达式缓存数量
    pub fn regex_count(&self) -> usize {
        self.regex_cache.len()
    }

    /// 清空正则表达式缓存
    pub fn clear_regex(&mut self) {
        self.regex_cache.clear();
    }

    // ==================== 表达式解析缓存 ====================

    /// 获取缓存的表达式
    pub fn get_expression(&self, expr_str: &str) -> Option<&ExpressionMeta> {
        self.expression_cache.get(expr_str)
    }

    /// 缓存表达式
    pub fn set_expression(&mut self, expr_str: String, expr: ExpressionMeta) {
        self.expression_cache.insert(expr_str, expr);
    }

    /// 检查表达式是否已缓存
    pub fn has_expression(&self, expr_str: &str) -> bool {
        self.expression_cache.contains_key(expr_str)
    }

    /// 获取表达式缓存数量
    pub fn expression_count(&self) -> usize {
        self.expression_cache.len()
    }

    /// 清空表达式缓存
    pub fn clear_expression(&mut self) {
        self.expression_cache.clear();
    }

    // ==================== 日期解析缓存 ====================

    /// 获取缓存的日期
    pub fn get_date(&self, date_str: &str) -> Option<&DateValue> {
        self.date_cache.get(date_str)
    }

    /// 缓存日期
    pub fn set_date(&mut self, date_str: String, date: DateValue) {
        self.date_cache.insert(date_str, date);
    }

    /// 获取日期缓存数量
    pub fn date_count(&self) -> usize {
        self.date_cache.len()
    }

    /// 清空日期缓存
    pub fn clear_date(&mut self) {
        self.date_cache.clear();
    }

    // ==================== 时间解析缓存 ====================

    /// 获取缓存的时间
    pub fn get_time(&self, time_str: &str) -> Option<&TimeValue> {
        self.time_cache.get(time_str)
    }

    /// 缓存时间
    pub fn set_time(&mut self, time_str: String, time: TimeValue) {
        self.time_cache.insert(time_str, time);
    }

    /// 获取时间缓存数量
    pub fn time_count(&self) -> usize {
        self.time_cache.len()
    }

    /// 清空时间缓存
    pub fn clear_time(&mut self) {
        self.time_cache.clear();
    }

    // ==================== 日期时间解析缓存 ====================

    /// 获取缓存的日期时间
    pub fn get_datetime(&self, datetime_str: &str) -> Option<&DateTimeValue> {
        self.datetime_cache.get(datetime_str)
    }

    /// 缓存日期时间
    pub fn set_datetime(&mut self, datetime_str: String, datetime: DateTimeValue) {
        self.datetime_cache.insert(datetime_str, datetime);
    }

    /// 获取日期时间缓存数量
    pub fn datetime_count(&self) -> usize {
        self.datetime_cache.len()
    }

    /// 清空日期时间缓存
    pub fn clear_datetime(&mut self) {
        self.datetime_cache.clear();
    }

    // ==================== 通用操作 ====================

    /// 获取总缓存数量
    pub fn total_count(&self) -> usize {
        self.regex_cache.len()
            + self.expression_cache.len()
            + self.date_cache.len()
            + self.time_cache.len()
            + self.datetime_cache.len()
    }

    /// 清空所有缓存
    pub fn clear(&mut self) {
        self.regex_cache.clear();
        self.expression_cache.clear();
        self.date_cache.clear();
        self.time_cache.clear();
        self.datetime_cache.clear();
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
    fn test_regex_cache() {
        let mut cache = CacheManager::new();

        let regex = cache.get_regex(r"\d+");
        assert!(regex.is_some());
        assert!(regex.expect("Expected regex to be valid").is_match("123"));
        assert_eq!(cache.regex_count(), 1);

        // 第二次获取应该使用缓存
        let regex2 = cache.get_regex(r"\d+");
        assert!(regex2.is_some());
        assert_eq!(cache.regex_count(), 1);
    }

    #[test]
    fn test_regex_cache_invalid() {
        let mut cache = CacheManager::new();

        let regex = cache.get_regex(r"[invalid");
        assert!(regex.is_none());
        assert_eq!(cache.regex_count(), 0);
    }

    #[test]
    fn test_expression_cache() {
        let mut cache = CacheManager::new();

        // 创建简单的表达式元数据
        let expr = ExpressionMeta::new(crate::core::types::expression::Expression::literal(42));
        cache.set_expression("42".to_string(), expr.clone());

        assert!(cache.has_expression("42"));
        assert_eq!(cache.expression_count(), 1);

        let retrieved = cache.get_expression("42");
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_date_cache() {
        let mut cache = CacheManager::new();

        let date = DateValue { year: 2024, month: 1, day: 15 };
        cache.set_date("2024-01-15".to_string(), date);

        let retrieved = cache.get_date("2024-01-15");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().year, 2024);
        assert_eq!(cache.date_count(), 1);
    }

    #[test]
    fn test_time_cache() {
        let mut cache = CacheManager::new();

        let time = TimeValue { hour: 14, minute: 30, sec: 0, microsec: 0 };
        cache.set_time("14:30:00".to_string(), time);

        let retrieved = cache.get_time("14:30:00");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().hour, 14);
        assert_eq!(cache.time_count(), 1);
    }

    #[test]
    fn test_datetime_cache() {
        let mut cache = CacheManager::new();

        let datetime = DateTimeValue {
            year: 2024,
            month: 1,
            day: 15,
            hour: 14,
            minute: 30,
            sec: 0,
            microsec: 0,
        };
        cache.set_datetime("2024-01-15 14:30:00".to_string(), datetime);

        let retrieved = cache.get_datetime("2024-01-15 14:30:00");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().year, 2024);
        assert_eq!(cache.datetime_count(), 1);
    }

    #[test]
    fn test_clear_all() {
        let mut cache = CacheManager::new();

        cache.get_regex(r"\d+");
        cache.set_expression("test".to_string(), ExpressionMeta::new(crate::core::types::expression::Expression::literal(1)));
        cache.set_date("2024-01-15".to_string(), DateValue { year: 2024, month: 1, day: 15 });

        assert!(cache.total_count() > 0);
        cache.clear();
        assert_eq!(cache.total_count(), 0);
    }
}
