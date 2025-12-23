//! 顺序迭代器 - 用于 DataSet 行级迭代
//!
//! SequentialIter用于遍历DataSet中的每一行，
//! 支持行级操作（添加、删除、修改）

use super::{Iterator, IteratorKind, Row};
use crate::core::Value;
use std::sync::Arc;

/// 顺序迭代器
///
/// 用于遍历DataSet行，支持：
/// - 行遍历：next、valid、reset
/// - 行删除：erase（有序）、unstable_erase（快速）
/// - 行级操作：select、erase_range、sample
/// - 列访问：get_column、get_column_by_index
#[derive(Debug, Clone)]
pub struct SequentialIter {
    data: Arc<Value>,       // 原始DataSet值的引用
    rows: Vec<Row>,         // 当前行数据（可能被修改）
    col_names: Vec<String>, // 列名
    curr_pos: usize,        // 当前位置
}

impl SequentialIter {
    /// 创建新的顺序迭代器
    ///
    /// # 参数
    /// - `data`: DataSet值的Arc指针
    ///
    /// # 返回
    /// - Ok(SequentialIter): 创建成功
    /// - Err(String): 如果data不是DataSet则返回错误
    pub fn new(data: Arc<Value>) -> Result<Self, String> {
        match &*data {
            Value::DataSet(dataset) => {
                let col_names = dataset.col_names.clone();
                let rows = dataset.rows.clone();
                Ok(Self {
                    data,
                    rows,
                    col_names,
                    curr_pos: 0,
                })
            }
            _ => Err("SequentialIter 只支持 DataSet".to_string()),
        }
    }

    /// 获取当前行的引用
    pub fn curr_row(&self) -> Option<&Row> {
        if self.curr_pos < self.rows.len() {
            Some(&self.rows[self.curr_pos])
        } else {
            None
        }
    }

    /// 获取当前行的可变引用
    pub fn curr_row_mut(&mut self) -> Option<&mut Row> {
        if self.curr_pos < self.rows.len() {
            Some(&mut self.rows[self.curr_pos])
        } else {
            None
        }
    }

    /// 获取列名列表的引用
    pub fn col_names(&self) -> &[String] {
        &self.col_names
    }

    /// 获取当前位置
    pub fn curr_pos(&self) -> usize {
        self.curr_pos
    }
}

impl Iterator for SequentialIter {
    fn kind(&self) -> IteratorKind {
        IteratorKind::Sequential
    }

    fn valid(&self) -> bool {
        self.curr_pos < self.rows.len()
    }

    fn next(&mut self) {
        if self.curr_pos < self.rows.len() {
            self.curr_pos += 1;
        }
    }

    fn erase(&mut self) {
        if self.curr_pos < self.rows.len() {
            self.rows.remove(self.curr_pos);
            // curr_pos 不变，指向下一行
        }
    }

    fn unstable_erase(&mut self) {
        if self.curr_pos < self.rows.len() {
            // 快速删除：交换最后一行到当前位置，然后 pop
            // 这不保持顺序，但速度更快
            let len = self.rows.len();
            self.rows.swap(self.curr_pos, len - 1);
            self.rows.pop();
            // curr_pos 不变，指向原来的最后一行
        }
    }

    fn clear(&mut self) {
        self.rows.clear();
        self.curr_pos = 0;
    }

    fn reset(&mut self, pos: usize) {
        self.curr_pos = if pos <= self.rows.len() {
            pos
        } else {
            self.rows.len()
        };
    }

    fn size(&self) -> usize {
        self.rows.len()
    }

    fn row(&self) -> Option<&Row> {
        self.curr_row()
    }

    fn move_row(&mut self) -> Option<Row> {
        if self.curr_pos < self.rows.len() {
            Some(self.rows[self.curr_pos].clone())
        } else {
            None
        }
    }

    fn select(&mut self, offset: usize, count: usize) {
        if offset >= self.rows.len() {
            self.rows.clear();
        } else {
            let end = std::cmp::min(offset + count, self.rows.len());
            let selected: Vec<_> = self.rows.drain(offset..end).collect();
            self.rows = selected;
            self.curr_pos = 0;
        }
    }

    fn sample(&mut self, count: i64) {
        if count <= 0 {
            self.clear();
        } else {
            let count = count as usize;
            if self.rows.len() > count {
                // 简单采样：保留前 count 行
                self.rows.truncate(count);
            }
            self.curr_pos = 0;
        }
    }

    fn erase_range(&mut self, first: usize, last: usize) {
        if first < self.rows.len() && first < last {
            let end = std::cmp::min(last, self.rows.len());
            self.rows.drain(first..end);
            // 调整当前位置
            if self.curr_pos >= first {
                self.curr_pos = first;
            }
        }
    }

    fn get_column(&self, col: &str) -> Option<&Value> {
        let col_idx = self.col_names.iter().position(|c| c == col)?;
        self.curr_row()?.get(col_idx)
    }

    fn get_column_by_index(&self, index: i32) -> Option<&Value> {
        let row = self.curr_row()?;
        let size = row.len() as i32;
        let idx = if index >= 0 {
            index as usize
        } else {
            let adjusted = (size + index) % size;
            if adjusted < 0 {
                return None;
            }
            adjusted as usize
        };
        row.get(idx)
    }

    fn get_column_index(&self, col: &str) -> Option<usize> {
        self.col_names.iter().position(|c| c == col)
    }

    fn get_col_names(&self) -> Vec<String> {
        self.col_names.clone()
    }

    fn copy(&self) -> Self {
        Self {
            data: self.data.clone(),
            rows: self.rows.clone(),
            col_names: self.col_names.clone(),
            curr_pos: self.curr_pos,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::DataSet;

    fn create_test_dataset() -> Value {
        let mut dataset = DataSet::new();
        dataset.col_names = vec!["name".to_string(), "age".to_string(), "city".to_string()];
        dataset.rows = vec![
            vec![
                Value::String("Alice".to_string()),
                Value::Int(28),
                Value::String("Beijing".to_string()),
            ],
            vec![
                Value::String("Bob".to_string()),
                Value::Int(25),
                Value::String("Shanghai".to_string()),
            ],
            vec![
                Value::String("Charlie".to_string()),
                Value::Int(30),
                Value::String("Shenzhen".to_string()),
            ],
        ];
        Value::DataSet(dataset)
    }

    #[test]
    fn test_sequential_iter_creation() {
        let data = Arc::new(create_test_dataset());
        let iter = SequentialIter::new(data);
        assert!(iter.is_ok());
        let iter = iter.expect("SequentialIter should be created successfully in test");
        assert_eq!(iter.size(), 3);
        assert!(iter.valid());
    }

    #[test]
    fn test_sequential_iter_navigation() {
        let data = Arc::new(create_test_dataset());
        let mut iter = SequentialIter::new(data)
            .expect("SequentialIter should be created successfully in test");

        // 检查初始状态
        assert_eq!(iter.curr_pos(), 0);
        assert!(iter.valid());

        // 移动到第二行
        iter.next();
        assert_eq!(iter.curr_pos(), 1);
        assert!(iter.valid());

        // 移动到第三行
        iter.next();
        assert_eq!(iter.curr_pos(), 2);
        assert!(iter.valid());

        // 移动到第四行（越界）
        iter.next();
        assert_eq!(iter.curr_pos(), 3);
        assert!(!iter.valid());
    }

    #[test]
    fn test_sequential_iter_get_column() {
        let data = Arc::new(create_test_dataset());
        let iter = SequentialIter::new(data)
            .expect("SequentialIter should be created successfully in test");

        // 获取第一行的列值
        assert_eq!(
            iter.get_column("name"),
            Some(&Value::String("Alice".to_string()))
        );
        assert_eq!(iter.get_column("age"), Some(&Value::Int(28)));
    }

    #[test]
    fn test_sequential_iter_erase() {
        let data = Arc::new(create_test_dataset());
        let mut iter = SequentialIter::new(data)
            .expect("SequentialIter should be created successfully in test");

        assert_eq!(iter.size(), 3);

        // 删除第一行
        iter.erase();
        assert_eq!(iter.size(), 2);
        // 当前位置仍然是0，但指向原来的第二行
        assert!(iter.valid());
        assert_eq!(
            iter.get_column("name"),
            Some(&Value::String("Bob".to_string()))
        );
    }

    #[test]
    fn test_sequential_iter_select() {
        let data = Arc::new(create_test_dataset());
        let mut iter = SequentialIter::new(data)
            .expect("SequentialIter should be created successfully in test");

        assert_eq!(iter.size(), 3);

        // 选择范围 [1, 3)
        iter.select(1, 2);
        assert_eq!(iter.size(), 2);
        assert_eq!(iter.curr_pos(), 0);
        assert!(iter.valid());
        assert_eq!(
            iter.get_column("name"),
            Some(&Value::String("Bob".to_string()))
        );
    }

    #[test]
    fn test_sequential_iter_erase_range() {
        let data = Arc::new(create_test_dataset());
        let mut iter = SequentialIter::new(data)
            .expect("SequentialIter should be created successfully in test");

        assert_eq!(iter.size(), 3);

        // 删除范围 [0, 2)
        iter.erase_range(0, 2);
        assert_eq!(iter.size(), 1);
        assert_eq!(
            iter.get_column("name"),
            Some(&Value::String("Charlie".to_string()))
        );
    }

    #[test]
    fn test_sequential_iter_reset() {
        let data = Arc::new(create_test_dataset());
        let mut iter = SequentialIter::new(data)
            .expect("SequentialIter should be created successfully in test");

        iter.next();
        iter.next();
        assert_eq!(iter.curr_pos(), 2);

        iter.reset(0);
        assert_eq!(iter.curr_pos(), 0);
        assert_eq!(
            iter.get_column("name"),
            Some(&Value::String("Alice".to_string()))
        );
    }

    #[test]
    fn test_sequential_iter_copy() {
        let data = Arc::new(create_test_dataset());
        let mut iter = SequentialIter::new(data)
            .expect("SequentialIter should be created successfully in test");

        iter.next();
        let copy = iter.copy();

        // 拷贝应该有相同的状态
        assert_eq!(copy.is_empty(), iter.is_empty());
        assert_eq!(copy.size(), iter.size());

        // 修改原迭代器不应该影响拷贝
        iter.next();
        assert_eq!(copy.size(), iter.size());
        assert_eq!(copy.is_empty(), iter.is_empty());
    }

    #[test]
    fn test_sequential_iter_sample() {
        let data = Arc::new(create_test_dataset());
        let mut iter = SequentialIter::new(data)
            .expect("SequentialIter should be created successfully in test");

        assert_eq!(iter.size(), 3);

        // 采样2行
        iter.sample(2);
        assert_eq!(iter.size(), 2);
        assert_eq!(iter.curr_pos(), 0);
    }

    #[test]
    fn test_sequential_iter_invalid_dataset() {
        let data = Arc::new(Value::Int(42));
        let result = SequentialIter::new(data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "SequentialIter 只支持 DataSet");
    }

    #[test]
    fn test_sequential_iter_get_column_index() {
        let data = Arc::new(create_test_dataset());
        let iter = SequentialIter::new(data)
            .expect("SequentialIter should be created successfully in test");

        assert_eq!(iter.get_column_index("name"), Some(0));
        assert_eq!(iter.get_column_index("age"), Some(1));
        assert_eq!(iter.get_column_index("city"), Some(2));
        assert_eq!(iter.get_column_index("unknown"), None);
    }

    #[test]
    fn test_sequential_iter_get_column_by_index() {
        let data = Arc::new(create_test_dataset());
        let iter = SequentialIter::new(data)
            .expect("SequentialIter should be created successfully in test");

        // 正索引
        assert_eq!(
            iter.get_column_by_index(0),
            Some(&Value::String("Alice".to_string()))
        );
        assert_eq!(iter.get_column_by_index(1), Some(&Value::Int(28)));

        // 负索引
        assert_eq!(
            iter.get_column_by_index(-1),
            Some(&Value::String("Beijing".to_string()))
        );
    }

    #[test]
    fn test_sequential_iter_unstable_erase() {
        let data = Arc::new(create_test_dataset());
        let mut iter = SequentialIter::new(data)
            .expect("SequentialIter should be created successfully in test");

        assert_eq!(iter.size(), 3);

        // 快速删除第一行
        iter.unstable_erase();
        assert_eq!(iter.size(), 2);

        // 由于是交换删除，顺序可能改变
        // 检查确实删除了一行
        assert_ne!(iter.size(), 3);
    }
}
