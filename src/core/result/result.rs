use crate::core::value::Value;
use crate::core::result::iterator::r#Iterator;
use std::sync::Arc;

/// Result 状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultState {
    NotStarted,
    InProgress,
    Completed,
    Failed,
}

/// Result 元数据
#[derive(Debug, Clone)]
pub struct ResultMeta {
    pub row_count: usize,
    pub col_count: usize,
    pub state: ResultState,
    pub memory_usage: u64,
}

impl Default for ResultMeta {
    fn default() -> Self {
        Self {
            row_count: 0,
            col_count: 0,
            state: ResultState::NotStarted,
            memory_usage: 0,
        }
    }
}

/// Result 结构体
/// 
/// 基于 Nebula-Graph 的 Result 设计，使用 Rust 的类型系统和内存安全特性
/// 
/// # 特性
/// - 零成本抽象：编译时优化，无运行时开销
/// - 类型安全：编译时类型检查
/// - 内存安全：Rust 所有权系统保证
/// - 高效迭代：支持多种迭代器类型
#[derive(Debug, Clone)]
pub struct Result {
    rows: Vec<Vec<Value>>,
    col_names: Vec<String>,
    meta: ResultMeta,
    iterator: Option<Arc<dyn Iterator>>,
}

impl Result {
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            col_names: Vec::new(),
            meta: ResultMeta::default(),
            iterator: None,
        }
    }

    pub fn with_capacity(row_capacity: usize, col_capacity: usize) -> Self {
        Self {
            rows: Vec::with_capacity(row_capacity),
            col_names: Vec::with_capacity(col_capacity),
            meta: ResultMeta::default(),
            iterator: None,
        }
    }

    pub fn with_col_names(col_names: Vec<String>) -> Self {
        let col_count = col_names.len();
        Self {
            rows: Vec::new(),
            col_names,
            meta: ResultMeta {
                col_count,
                ..Default::default()
            },
            iterator: None,
        }
    }

    pub fn col_names(&self) -> &[String] {
        &self.col_names
    }

    pub fn set_col_names(&mut self, col_names: Vec<String>) {
        self.col_names = col_names;
        self.meta.col_count = self.col_names.len();
    }

    pub fn row_count(&self) -> usize {
        self.meta.row_count
    }

    pub fn col_count(&self) -> usize {
        self.meta.col_count
    }

    pub fn state(&self) -> ResultState {
        self.meta.state
    }

    pub fn set_state(&mut self, state: ResultState) {
        self.meta.state = state;
    }

    pub fn memory_usage(&self) -> u64 {
        self.meta.memory_usage
    }

    pub fn add_row(&mut self, row: Vec<Value>) {
        self.rows.push(row);
        self.meta.row_count = self.rows.len();
    }

    pub fn rows(&self) -> &[Vec<Value>] {
        &self.rows
    }

    pub fn rows_mut(&mut self) -> &mut Vec<Vec<Value>> {
        &mut self.rows
    }

    pub fn get_row(&self, index: usize) -> Option<&Vec<Value>> {
        self.rows.get(index)
    }

    pub fn get_row_mut(&mut self, index: usize) -> Option<&mut Vec<Value>> {
        self.rows.get_mut(index)
    }

    pub fn get_value(&self, row: usize, col: usize) -> Option<&Value> {
        self.rows.get(row).and_then(|r| r.get(col))
    }

    pub fn set_value(&mut self, row: usize, col: usize, value: Value) {
        if let Some(r) = self.rows.get_mut(row) {
            if col < r.len() {
                r[col] = value;
            }
        }
    }

    pub fn set_iterator(&mut self, iterator: Arc<dyn Iterator>) {
        self.iterator = Some(iterator);
    }

    pub fn iterator(&self) -> Option<&Arc<dyn Iterator>> {
        self.iterator.as_ref()
    }

    pub fn take_iterator(&mut self) -> Option<Arc<dyn Iterator>> {
        self.iterator.take()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn clear(&mut self) {
        self.rows.clear();
        self.meta.row_count = 0;
        self.meta.state = ResultState::NotStarted;
        self.iterator = None;
    }

    pub fn shrink_to_fit(&mut self) {
        self.rows.shrink_to_fit();
        self.col_names.shrink_to_fit();
    }

    pub fn reserve(&mut self, additional: usize) {
        self.rows.reserve(additional);
    }

    pub fn estimate_memory_usage(&self) -> u64 {
        let mut total = 0u64;
        
        total += (self.rows.len() * std::mem::size_of::<Vec<Value>>()) as u64;
        
        for row in &self.rows {
            total += (row.len() * std::mem::size_of::<Value>()) as u64;
        }
        
        total += (self.col_names.len() * std::mem::size_of::<String>()) as u64;
        
        for name in &self.col_names {
            total += name.capacity() as u64;
        }
        
        total
    }

    pub fn update_memory_usage(&mut self) {
        self.meta.memory_usage = self.estimate_memory_usage();
    }

    pub fn meta(&self) -> &ResultMeta {
        &self.meta
    }

    pub fn meta_mut(&mut self) -> &mut ResultMeta {
        &mut self.meta
    }

    pub fn from_rows(rows: Vec<Vec<Value>>, col_names: Vec<String>) -> Self {
        let row_count = rows.len();
        let col_count = col_names.len();
        
        let mut result = Self {
            rows,
            col_names,
            meta: ResultMeta {
                row_count,
                col_count,
                state: ResultState::Completed,
                ..Default::default()
            },
            iterator: None,
        };
        
        result.update_memory_usage();
        result
    }

    pub fn merge(&mut self, other: Result) {
        self.rows.extend(other.rows);
        self.meta.row_count = self.rows.len();
        self.update_memory_usage();
    }

    pub fn split_at(&mut self, mid: usize) -> Result {
        let (left_rows, right_rows) = self.rows.split_at(mid);
        let right_rows = right_rows.to_vec();
        let right_len = right_rows.len();
        self.rows = left_rows.to_vec();
        self.meta.row_count = self.rows.len();
        
        let mut result = Self {
            rows: right_rows,
            col_names: self.col_names.clone(),
            meta: ResultMeta {
                row_count: right_len,
                col_count: self.meta.col_count,
                state: self.meta.state,
                ..Default::default()
            },
            iterator: None,
        };
        
        result.update_memory_usage();
        self.update_memory_usage();
        result
    }
}

impl Default for Result {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for Result {
    type Item = Vec<Value>;
    type IntoIter = std::vec::IntoIter<Vec<Value>>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows.into_iter()
    }
}

impl<'a> IntoIterator for &'a Result {
    type Item = &'a Vec<Value>;
    type IntoIter = std::slice::Iter<'a, Vec<Value>>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_new() {
        let result = Result::new();
        assert_eq!(result.row_count(), 0);
        assert_eq!(result.col_count(), 0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_result_with_capacity() {
        let result = Result::with_capacity(10, 5);
        assert_eq!(result.row_count(), 0);
        assert_eq!(result.col_count(), 0);
    }

    #[test]
    fn test_result_with_col_names() {
        let col_names = vec!["id".to_string(), "name".to_string()];
        let result = Result::with_col_names(col_names.clone());
        assert_eq!(result.col_names(), &col_names);
        assert_eq!(result.col_count(), 2);
    }

    #[test]
    fn test_result_add_row() {
        let mut result = Result::new();
        result.add_row(vec![Value::Int(1), Value::String("Alice".to_string())]);
        result.add_row(vec![Value::Int(2), Value::String("Bob".to_string())]);
        
        assert_eq!(result.row_count(), 2);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_result_get_row() {
        let mut result = Result::new();
        result.add_row(vec![Value::Int(1), Value::String("Alice".to_string())]);
        
        let row = result.get_row(0);
        assert!(row.is_some());
        assert_eq!(row.unwrap()[0], Value::Int(1));
    }

    #[test]
    fn test_result_get_value() {
        let mut result = Result::new();
        result.add_row(vec![Value::Int(1), Value::String("Alice".to_string())]);
        
        let value = result.get_value(0, 1);
        assert!(value.is_some());
        assert_eq!(value.unwrap(), &Value::String("Alice".to_string()));
    }

    #[test]
    fn test_result_set_value() {
        let mut result = Result::new();
        result.add_row(vec![Value::Int(1), Value::String("Alice".to_string())]);
        
        result.set_value(0, 1, Value::String("Bob".to_string()));
        let value = result.get_value(0, 1);
        assert_eq!(value.unwrap(), &Value::String("Bob".to_string()));
    }

    #[test]
    fn test_result_clear() {
        let mut result = Result::new();
        result.add_row(vec![Value::Int(1), Value::String("Alice".to_string())]);
        result.set_state(ResultState::Completed);
        
        result.clear();
        
        assert_eq!(result.row_count(), 0);
        assert!(result.is_empty());
        assert_eq!(result.state(), ResultState::NotStarted);
    }

    #[test]
    fn test_result_from_rows() {
        let rows = vec![
            vec![Value::Int(1), Value::String("Alice".to_string())],
            vec![Value::Int(2), Value::String("Bob".to_string())],
        ];
        let col_names = vec!["id".to_string(), "name".to_string()];
        
        let result = Result::from_rows(rows, col_names);
        
        assert_eq!(result.row_count(), 2);
        assert_eq!(result.col_count(), 2);
        assert_eq!(result.state(), ResultState::Completed);
    }

    #[test]
    fn test_result_merge() {
        let mut result1 = Result::new();
        result1.add_row(vec![Value::Int(1)]);
        
        let mut result2 = Result::new();
        result2.add_row(vec![Value::Int(2)]);
        result2.add_row(vec![Value::Int(3)]);
        
        result1.merge(result2);
        
        assert_eq!(result1.row_count(), 3);
    }

    #[test]
    fn test_result_split_at() {
        let mut result = Result::new();
        result.add_row(vec![Value::Int(1)]);
        result.add_row(vec![Value::Int(2)]);
        result.add_row(vec![Value::Int(3)]);
        
        let result2 = result.split_at(1);
        
        assert_eq!(result.row_count(), 1);
        assert_eq!(result2.row_count(), 2);
    }

    #[test]
    fn test_result_into_iterator() {
        let mut result = Result::new();
        result.add_row(vec![Value::Int(1)]);
        result.add_row(vec![Value::Int(2)]);
        
        let rows: Vec<_> = result.into_iter().collect();
        assert_eq!(rows.len(), 2);
    }
}
