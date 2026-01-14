//! 属性跟踪器
//!
//! 用于跟踪查询中使用的属性，支持属性修剪优化

use std::collections::{HashMap, HashSet};

/// 属性跟踪器
///
/// 跟踪查询中使用的属性，用于属性修剪优化
#[derive(Debug, Clone)]
pub struct PropertyTracker {
    /// 使用的属性映射：变量名 -> 属性名集合
    used_properties: HashMap<String, HashSet<String>>,
}

impl PropertyTracker {
    /// 创建新的属性跟踪器
    pub fn new() -> Self {
        Self {
            used_properties: HashMap::new(),
        }
    }

    /// 跟踪属性
    ///
    /// # 参数
    /// * `var` - 变量名
    /// * `prop` - 属性名
    pub fn track_property(&mut self, var: &str, prop: &str) {
        self.used_properties
            .entry(var.to_string())
            .or_insert_with(HashSet::new)
            .insert(prop.to_string());
    }

    /// 检查属性是否被使用
    ///
    /// # 参数
    /// * `var` - 变量名
    /// * `prop` - 属性名
    ///
    /// # 返回值
    /// 如果属性被使用，返回 true
    pub fn is_property_used(&self, var: &str, prop: &str) -> bool {
        if let Some(props) = self.used_properties.get(var) {
            props.contains(prop)
        } else {
            false
        }
    }

    /// 获取变量的所有使用属性
    ///
    /// # 参数
    /// * `var` - 变量名
    ///
    /// # 返回值
    /// 返回变量的所有使用属性
    pub fn get_used_properties(&self, var: &str) -> Option<&HashSet<String>> {
        self.used_properties.get(var)
    }

    /// 获取所有使用的属性
    ///
    /// # 返回值
    /// 返回所有使用的属性映射
    pub fn get_all_used_properties(&self) -> &HashMap<String, HashSet<String>> {
        &self.used_properties
    }

    /// 合并另一个属性跟踪器
    ///
    /// # 参数
    /// * `other` - 要合并的属性跟踪器
    pub fn merge(&mut self, other: &PropertyTracker) {
        for (var, props) in other.used_properties.iter() {
            for prop in props {
                self.track_property(var, prop);
            }
        }
    }

    /// 清除所有跟踪的属性
    pub fn clear(&mut self) {
        self.used_properties.clear();
    }

    /// 跟踪的变量数量
    ///
    /// # 返回值
    /// 返回跟踪的变量数量
    pub fn variable_count(&self) -> usize {
        self.used_properties.len()
    }

    /// 获取变量的属性数量
    ///
    /// # 参数
    /// * `var` - 变量名
    ///
    /// # 返回值
    /// 返回变量的属性数量
    pub fn property_count(&self, var: &str) -> usize {
        self.used_properties
            .get(var)
            .map(|props| props.len())
            .unwrap_or(0)
    }

    /// 检查变量是否有任何使用的属性
    ///
    /// # 参数
    /// * `var` - 变量名
    ///
    /// # 返回值
    /// 如果变量有任何使用的属性，返回 true
    pub fn has_any_property(&self, var: &str) -> bool {
        self.used_properties
            .get(var)
            .map(|props| !props.is_empty())
            .unwrap_or(false)
    }

    /// 移除变量的属性跟踪
    ///
    /// # 参数
    /// * `var` - 变量名
    pub fn remove_variable(&mut self, var: &str) {
        self.used_properties.remove(var);
    }

    /// 移除变量的特定属性跟踪
    ///
    /// # 参数
    /// * `var` - 变量名
    /// * `prop` - 属性名
    pub fn remove_property(&mut self, var: &str, prop: &str) {
        if let Some(props) = self.used_properties.get_mut(var) {
            props.remove(prop);
            if props.is_empty() {
                self.used_properties.remove(var);
            }
        }
    }

    /// 克隆并创建新的跟踪器
    pub fn clone_tracker(&self) -> PropertyTracker {
        Self {
            used_properties: self.used_properties.clone(),
        }
    }
}

impl Default for PropertyTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_property() {
        let mut tracker = PropertyTracker::new();
        tracker.track_property("v", "name");
        tracker.track_property("v", "age");
        tracker.track_property("e", "weight");

        assert!(tracker.is_property_used("v", "name"));
        assert!(tracker.is_property_used("v", "age"));
        assert!(tracker.is_property_used("e", "weight"));
        assert!(!tracker.is_property_used("v", "unknown"));
    }

    #[test]
    fn test_get_used_properties() {
        let mut tracker = PropertyTracker::new();
        tracker.track_property("v", "name");
        tracker.track_property("v", "age");

        let props = tracker.get_used_properties("v");
        assert!(props.is_some());
        let props = props.unwrap();
        assert_eq!(props.len(), 2);
        assert!(props.contains("name"));
        assert!(props.contains("age"));
    }

    #[test]
    fn test_merge() {
        let mut tracker1 = PropertyTracker::new();
        tracker1.track_property("v", "name");

        let mut tracker2 = PropertyTracker::new();
        tracker2.track_property("v", "age");
        tracker2.track_property("e", "weight");

        tracker1.merge(&tracker2);

        assert!(tracker1.is_property_used("v", "name"));
        assert!(tracker1.is_property_used("v", "age"));
        assert!(tracker1.is_property_used("e", "weight"));
    }

    #[test]
    fn test_clear() {
        let mut tracker = PropertyTracker::new();
        tracker.track_property("v", "name");
        tracker.track_property("v", "age");

        assert_eq!(tracker.variable_count(), 1);

        tracker.clear();

        assert_eq!(tracker.variable_count(), 0);
        assert!(!tracker.is_property_used("v", "name"));
    }

    #[test]
    fn test_remove_variable() {
        let mut tracker = PropertyTracker::new();
        tracker.track_property("v", "name");
        tracker.track_property("v", "age");

        tracker.remove_variable("v");

        assert!(!tracker.is_property_used("v", "name"));
        assert!(!tracker.is_property_used("v", "age"));
    }

    #[test]
    fn test_remove_property() {
        let mut tracker = PropertyTracker::new();
        tracker.track_property("v", "name");
        tracker.track_property("v", "age");

        tracker.remove_property("v", "name");

        assert!(!tracker.is_property_used("v", "name"));
        assert!(tracker.is_property_used("v", "age"));
    }

    #[test]
    fn test_has_any_property() {
        let mut tracker = PropertyTracker::new();
        tracker.track_property("v", "name");

        assert!(tracker.has_any_property("v"));
        assert!(!tracker.has_any_property("e"));
    }

    #[test]
    fn test_property_count() {
        let mut tracker = PropertyTracker::new();
        tracker.track_property("v", "name");
        tracker.track_property("v", "age");

        assert_eq!(tracker.property_count("v"), 2);
        assert_eq!(tracker.property_count("e"), 0);
    }

    #[test]
    fn test_clone_tracker() {
        let mut tracker1 = PropertyTracker::new();
        tracker1.track_property("v", "name");

        let tracker2 = tracker1.clone_tracker();

        assert!(tracker2.is_property_used("v", "name"));

        tracker1.remove_property("v", "name");

        assert!(!tracker1.is_property_used("v", "name"));
        assert!(tracker2.is_property_used("v", "name"));
    }
}
