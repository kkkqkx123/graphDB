//! 默认迭代器 - 用于单个值
//!
//! DefaultIter用于表示单个常量值，如数字、字符串等
//! 逻辑上只有一"行"（值本身），next()后无效

use super::{Iterator, IteratorKind, Row};
use crate::core::Value;
use std::sync::Arc;

/// 默认迭代器
///
/// 用于包装单个Value，使其可以通过迭代器接口访问
/// 始终有一个有效行，代表该值本身
#[derive(Debug, Clone)]
pub struct DefaultIter {
    value: Arc<Value>,
    counter: usize,
}

impl DefaultIter {
    /// 创建新的默认迭代器
    pub fn new(value: Arc<Value>) -> Self {
        Self { value, counter: 0 }
    }

    /// 创建新的默认迭代器（拥有所有权）
    pub fn from_value(value: Value) -> Self {
        Self::new(Arc::new(value))
    }
}

impl Iterator for DefaultIter {
    fn kind(&self) -> IteratorKind {
        IteratorKind::Default
    }

    fn valid(&self) -> bool {
        self.counter == 0
    }

    fn next(&mut self) {
        self.counter += 1;
    }

    fn erase(&mut self) {
        self.counter += 1;
    }

    fn unstable_erase(&mut self) {
        self.counter += 1;
    }

    fn clear(&mut self) {
        self.counter = 1;
    }

    fn reset(&mut self, pos: usize) {
        self.counter = pos;
    }

    fn size(&self) -> usize {
        if self.valid() {
            1
        } else {
            0
        }
    }

    fn row(&self) -> Option<&Row> {
        // DefaultIter 只有一行（整个值）
        // 但我们不能将 Value 转成 Row 的引用
        // 所以返回 None，用户应该通过 get_column 获取值
        None
    }

    fn move_row(&mut self) -> Option<Row> {
        // DefaultIter 不支持行级操作
        None
    }

    fn add_row(&mut self, _row: Row) {
        // DefaultIter 不支持添加行
    }

    fn select(&mut self, offset: usize, count: usize) {
        if offset > 0 || count == 0 {
            self.clear();
        }
    }

    fn sample(&mut self, count: i64) {
        if count == 0 {
            self.clear();
        }
    }

    fn erase_range(&mut self, _first: usize, _last: usize) {
        // DefaultIter 只有一行，任何范围删除都会清空
        self.clear();
    }

    fn get_column(&self, _col: &str) -> Option<&Value> {
        // DefaultIter 返回值本身作为列
        Some(&self.value)
    }

    fn get_column_by_index(&self, index: i32) -> Option<&Value> {
        // DefaultIter 只有索引 0
        if index == 0 || index == -1 {
            Some(&self.value)
        } else {
            None
        }
    }

    fn get_column_index(&self, _col: &str) -> Option<usize> {
        // DefaultIter 只有索引 0
        Some(0)
    }

    fn get_col_names(&self) -> Vec<String> {
        // DefaultIter 没有列名
        vec![]
    }

    fn copy(&self) -> Self {
        Self {
            value: self.value.clone(),
            counter: self.counter,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_iter_creation() {
        let value = Arc::new(Value::Int(42));
        let iter = DefaultIter::new(value.clone());

        assert_eq!(iter.kind(), IteratorKind::Default);
        assert!(iter.valid());
        assert_eq!(iter.size(), 1);
        assert!(!iter.is_empty());
    }

    #[test]
    fn test_default_iter_next() {
        let value = Arc::new(Value::Int(42));
        let mut iter = DefaultIter::new(value.clone());

        assert!(iter.valid());
        iter.next();
        assert!(!iter.valid());
    }

    #[test]
    fn test_default_iter_reset() {
        let value = Arc::new(Value::Int(42));
        let mut iter = DefaultIter::new(value);

        iter.next();
        assert!(!iter.valid());

        iter.reset(0);
        assert!(iter.valid());
    }

    #[test]
    fn test_default_iter_get_column() {
        let value = Arc::new(Value::String("hello".to_string()));
        let iter = DefaultIter::new(value.clone());

        // DefaultIter 返回值本身作为列
        assert_eq!(iter.get_column("any_name"), Some(&*value));
        assert_eq!(iter.get_column_by_index(0), Some(&*value));
        assert_eq!(iter.get_column_index("any_name"), Some(0));
    }

    #[test]
    fn test_default_iter_select() {
        let value = Arc::new(Value::Int(42));
        let mut iter = DefaultIter::new(value);

        // select(offset > 0, count) 会清空
        iter.select(1, 10);
        assert!(!iter.valid());
        assert_eq!(iter.size(), 0);

        // 重置后 select(0, 0) 也会清空
        iter.reset(0);
        iter.select(0, 0);
        assert!(!iter.valid());
        assert_eq!(iter.size(), 0);

        // 重置后 select(0, count > 0) 保留
        iter.reset(0);
        iter.select(0, 1);
        assert!(iter.valid());
        assert_eq!(iter.size(), 1);
    }

    #[test]
    fn test_default_iter_sample() {
        let value = Arc::new(Value::Int(42));
        let mut iter = DefaultIter::new(value);

        // sample(0) 清空
        iter.sample(0);
        assert!(!iter.valid());

        // 重置后 sample(1) 保留
        iter.reset(0);
        iter.sample(1);
        assert!(iter.valid());

        // 重置后 sample(count > 1) 也保留（只有1行）
        iter.reset(0);
        iter.sample(100);
        assert!(iter.valid());
    }

    #[test]
    fn test_default_iter_copy() {
        let value = Arc::new(Value::Int(42));
        let mut iter = DefaultIter::new(value.clone());

        iter.next();
        assert!(!iter.valid());

        let mut copy = iter.copy();
        assert!(!copy.valid()); // 深拷贝状态

        copy.reset(0);
        assert!(copy.valid());
    }

    #[test]
    fn test_default_iter_from_value() {
        let iter = DefaultIter::from_value(Value::Int(100));
        assert!(iter.valid());
        assert_eq!(iter.get_column("test"), Some(&Value::Int(100)));
    }
}
