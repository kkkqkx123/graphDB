//! 结果迭代器模块 - 为Result提供迭代器接口

use crate::core::Value;
use std::sync::Arc;

/// 结果迭代器trait
pub trait ResultIterator: Send + Sync {
    /// 获取值
    fn value_ptr(&self) -> Arc<Value>;

    /// 是否有效
    fn is_valid(&self) -> bool;

    /// 移动到下一个
    fn next(&mut self);

    /// 当前位置的行
    fn current_row(&self) -> Option<&Vec<Value>>;

    /// 大小
    fn size(&self) -> usize;

    /// 是否为空
    fn is_empty(&self) -> bool {
        self.size() == 0
    }

    /// 重置位置
    fn reset(&mut self);

    /// 清空
    fn clear(&mut self);

    /// 获取列值
    fn get_column(&self, col_name: &str) -> Option<&Value>;

    /// 获取列值（通过索引）
    fn get_column_by_index(&self, index: usize) -> Option<&Value>;

    /// 获取列名
    fn get_col_names(&self) -> Vec<String>;
}

/// 顺序结果迭代器
#[derive(Debug)]
pub struct SequentialResultIterator {
    data: Arc<Value>,
    rows: Vec<Vec<Value>>,
    col_names: Vec<String>,
    current_pos: usize,
}

impl SequentialResultIterator {
    pub fn new(data: Arc<Value>) -> Result<Self, String> {
        match &*data {
            Value::DataSet(dataset) => {
                let col_names = dataset.col_names.clone();
                let rows = dataset.rows.clone();
                Ok(Self {
                    data,
                    rows,
                    col_names,
                    current_pos: 0,
                })
            }
            _ => Err("SequentialResultIterator 只支持 DataSet".to_string()),
        }
    }
}

impl ResultIterator for SequentialResultIterator {
    fn value_ptr(&self) -> Arc<Value> {
        if self.current_pos < self.rows.len() {
            if let Some(row) = &self.rows.get(self.current_pos) {
                if let Some(first_value) = row.first() {
                    return Arc::new(first_value.clone());
                }
            }
        }
        Arc::new(Value::Null(crate::core::NullType::Null))
    }

    fn is_valid(&self) -> bool {
        self.current_pos < self.rows.len()
    }

    fn next(&mut self) {
        if self.current_pos < self.rows.len() {
            self.current_pos += 1;
        }
    }

    fn current_row(&self) -> Option<&Vec<Value>> {
        if self.current_pos < self.rows.len() {
            Some(&self.rows[self.current_pos])
        } else {
            None
        }
    }

    fn size(&self) -> usize {
        self.rows.len()
    }

    fn reset(&mut self) {
        self.current_pos = 0;
    }

    fn clear(&mut self) {
        self.rows.clear();
        self.current_pos = 0;
    }

    fn get_column(&self, col_name: &str) -> Option<&Value> {
        if let Some(col_idx) = self.col_names.iter().position(|c| c == col_name) {
            if let Some(row) = self.current_row() {
                return row.get(col_idx);
            }
        }
        None
    }

    fn get_column_by_index(&self, index: usize) -> Option<&Value> {
        if let Some(row) = self.current_row() {
            row.get(index)
        } else {
            None
        }
    }

    fn get_col_names(&self) -> Vec<String> {
        self.col_names.clone()
    }
}

/// 默认结果迭代器（用于单个值）
pub struct DefaultResultIterator {
    value: Arc<Value>,
    is_valid: bool,
}

impl DefaultResultIterator {
    pub fn new(value: Arc<Value>) -> Self {
        Self {
            value,
            is_valid: true,
        }
    }
}

impl ResultIterator for DefaultResultIterator {
    fn value_ptr(&self) -> Arc<Value> {
        self.value.clone()
    }

    fn is_valid(&self) -> bool {
        self.is_valid
    }

    fn next(&mut self) {
        self.is_valid = false;
    }

    fn current_row(&self) -> Option<&Vec<Value>> {
        None
    }

    fn size(&self) -> usize {
        1
    }

    fn reset(&mut self) {
        self.is_valid = true;
    }

    fn clear(&mut self) {
        self.is_valid = false;
    }

    fn get_column(&self, _col_name: &str) -> Option<&Value> {
        Some(&self.value)
    }

    fn get_column_by_index(&self, _index: usize) -> Option<&Value> {
        Some(&self.value)
    }

    fn get_col_names(&self) -> Vec<String> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::DataSet;

    #[test]
    fn test_default_result_iterator() {
        let value = Arc::new(Value::Int(42));
        let mut iter = DefaultResultIterator::new(value.clone());

        assert!(iter.is_valid());
        assert_eq!(iter.size(), 1);
        assert_eq!(iter.value_ptr(), value);

        iter.next();
        assert!(!iter.is_valid());

        iter.reset();
        assert!(iter.is_valid());
    }

    #[test]
    fn test_sequential_result_iterator() {
        let mut dataset = DataSet::new();
        dataset.col_names = vec!["name".to_string(), "age".to_string()];
        dataset.rows = vec![
            vec![Value::String("Alice".to_string()), Value::Int(25)],
            vec![Value::String("Bob".to_string()), Value::Int(30)],
        ];
        let data = Arc::new(Value::DataSet(dataset));

        let mut iter = SequentialResultIterator::new(data)
            .expect("SequentialResultIterator should be created successfully with valid DataSet");

        assert!(iter.is_valid());
        assert_eq!(iter.size(), 2);

        // 测试第一行
        if let Some(row) = iter.current_row() {
            assert_eq!(row.len(), 2);
            assert_eq!(row[0], Value::String("Alice".to_string()));
            assert_eq!(row[1], Value::Int(25));
        }

        // 测试列访问
        assert_eq!(
            iter.get_column("name"),
            Some(&Value::String("Alice".to_string()))
        );
        assert_eq!(iter.get_column_by_index(1), Some(&Value::Int(25)));

        iter.next();
        assert!(iter.is_valid());

        // 测试第二行
        if let Some(row) = iter.current_row() {
            assert_eq!(row[0], Value::String("Bob".to_string()));
        }

        iter.next();
        assert!(!iter.is_valid());

        iter.reset();
        assert!(iter.is_valid());
    }

    #[test]
    fn test_sequential_result_iterator_invalid_data() {
        let data = Arc::new(Value::Int(42));
        let result = SequentialResultIterator::new(data);
        match result {
            Err(e) => assert_eq!(e, "SequentialResultIterator 只支持 DataSet"),
            Ok(_) => panic!("Expected error but got Ok"),
        }
    }
}
