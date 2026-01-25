use crate::core::value::Value;
use crate::core::result::Result;
use crate::core::result::iterator::{r#Iterator, DefaultIterator, GetNeighborsIterator, PropIterator, IteratorType};
use std::sync::Arc;

/// ResultBuilder
/// 
/// 用于构建 Result 对象的构建器模式实现
/// 
/// # 特性
/// - 链式调用：支持流畅的 API 调用
/// - 类型安全：编译时类型检查
/// - 内存安全：Rust 所有权系统保证
/// - 灵活配置：支持多种配置选项
pub struct ResultBuilder {
    col_names: Vec<String>,
    rows: Vec<Vec<Value>>,
    iterator_type: Option<IteratorType>,
    capacity: Option<usize>,
}

impl Default for ResultBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ResultBuilder {
    pub fn new() -> Self {
        Self {
            col_names: Vec::new(),
            rows: Vec::new(),
            iterator_type: None,
            capacity: None,
        }
    }

    pub fn with_capacity(row_capacity: usize, col_capacity: usize) -> Self {
        Self {
            col_names: Vec::with_capacity(col_capacity),
            rows: Vec::with_capacity(row_capacity),
            iterator_type: None,
            capacity: Some(row_capacity),
        }
    }

    pub fn col_names(mut self, col_names: Vec<String>) -> Self {
        self.col_names = col_names;
        self
    }

    pub fn add_col_name(mut self, col_name: String) -> Self {
        self.col_names.push(col_name);
        self
    }

    pub fn rows(mut self, rows: Vec<Vec<Value>>) -> Self {
        self.rows = rows;
        self
    }

    pub fn add_row(mut self, row: Vec<Value>) -> Self {
        self.rows.push(row);
        self
    }

    pub fn iterator_type(mut self, iterator_type: IteratorType) -> Self {
        self.iterator_type = Some(iterator_type);
        self
    }

    pub fn capacity(mut self, capacity: usize) -> Self {
        self.capacity = Some(capacity);
        self
    }

    pub fn build(self) -> Result {
        let mut result = if let Some(cap) = self.capacity {
            Result::with_capacity(cap, self.col_names.len())
        } else {
            Result::new()
        };

        result.set_col_names(self.col_names);
        
        for row in self.rows {
            result.add_row(row);
        }

        result.set_state(crate::core::result::result::ResultState::Completed);

        if let Some(iter_type) = self.iterator_type {
            let iterator: Arc<dyn Iterator> = match iter_type {
                IteratorType::Default => Arc::new(DefaultIterator::new(result.rows().to_vec())),
                IteratorType::GetNeighbors => Arc::new(GetNeighborsIterator::new(vec![], vec![])),
                IteratorType::Prop => Arc::new(PropIterator::new(result.rows().to_vec())),
            };
            result.set_iterator(iterator);
        }

        result.update_memory_usage();
        result
    }

    pub fn build_with_iterator(self, iterator: Arc<dyn Iterator>) -> Result {
        let mut result = if let Some(cap) = self.capacity {
            Result::with_capacity(cap, self.col_names.len())
        } else {
            Result::new()
        };

        result.set_col_names(self.col_names);
        
        for row in self.rows {
            result.add_row(row);
        }

        result.set_state(crate::core::result::result::ResultState::Completed);
        result.set_iterator(iterator);
        result.update_memory_usage();
        result
    }

    pub fn build_empty(self) -> Result {
        let mut result = Result::new();
        result.set_col_names(self.col_names);
        result.set_state(crate::core::result::result::ResultState::Completed);
        result.update_memory_usage();
        result
    }

    pub fn build_from_rows(self) -> Result {
        Result::from_rows(self.rows, self.col_names)
    }

    pub fn col_count(&self) -> usize {
        self.col_names.len()
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn clear(mut self) -> Self {
        self.rows.clear();
        self.col_names.clear();
        self
    }

    pub fn reset(mut self) -> Self {
        self.rows.clear();
        self.col_names.clear();
        self.iterator_type = None;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_builder_new() {
        let builder = ResultBuilder::new();
        assert_eq!(builder.col_count(), 0);
        assert_eq!(builder.row_count(), 0);
        assert!(builder.is_empty());
    }

    #[test]
    fn test_result_builder_with_capacity() {
        let builder = ResultBuilder::with_capacity(10, 5);
        assert_eq!(builder.col_count(), 0);
        assert_eq!(builder.row_count(), 0);
    }

    #[test]
    fn test_result_builder_col_names() {
        let col_names = vec!["id".to_string(), "name".to_string()];
        let builder = ResultBuilder::new()
            .col_names(col_names.clone());
        
        assert_eq!(builder.col_count(), 2);
    }

    #[test]
    fn test_result_builder_add_col_name() {
        let builder = ResultBuilder::new()
            .add_col_name("id".to_string())
            .add_col_name("name".to_string());
        
        assert_eq!(builder.col_count(), 2);
    }

    #[test]
    fn test_result_builder_rows() {
        let rows = vec![
            vec![Value::Int(1), Value::String("Alice".to_string())],
            vec![Value::Int(2), Value::String("Bob".to_string())],
        ];
        
        let builder = ResultBuilder::new()
            .rows(rows.clone());
        
        assert_eq!(builder.row_count(), 2);
    }

    #[test]
    fn test_result_builder_add_row() {
        let builder = ResultBuilder::new()
            .add_row(vec![Value::Int(1)])
            .add_row(vec![Value::Int(2)]);
        
        assert_eq!(builder.row_count(), 2);
    }

    #[test]
    fn test_result_builder_build() {
        let col_names = vec!["id".to_string(), "name".to_string()];
        let rows = vec![
            vec![Value::Int(1), Value::String("Alice".to_string())],
        ];
        
        let result = ResultBuilder::new()
            .col_names(col_names)
            .rows(rows)
            .build();
        
        assert_eq!(result.col_count(), 2);
        assert_eq!(result.row_count(), 1);
        assert_eq!(result.state(), crate::core::result::result::ResultState::Completed);
    }

    #[test]
    fn test_result_builder_build_with_iterator() {
        let col_names = vec!["id".to_string()];
        let rows = vec![vec![Value::Int(1)]];
        let iterator = Arc::new(DefaultIterator::new(rows.clone()));
        
        let result = ResultBuilder::new()
            .col_names(col_names)
            .rows(rows)
            .build_with_iterator(iterator);
        
        assert!(result.iterator().is_some());
    }

    #[test]
    fn test_result_builder_build_empty() {
        let col_names = vec!["id".to_string()];
        
        let result = ResultBuilder::new()
            .col_names(col_names)
            .build_empty();
        
        assert_eq!(result.col_count(), 1);
        assert_eq!(result.row_count(), 0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_result_builder_build_from_rows() {
        let col_names = vec!["id".to_string()];
        let rows = vec![vec![Value::Int(1)]];
        
        let result = ResultBuilder::new()
            .col_names(col_names)
            .rows(rows)
            .build_from_rows();
        
        assert_eq!(result.row_count(), 1);
    }

    #[test]
    fn test_result_builder_clear() {
        let builder = ResultBuilder::new()
            .add_col_name("id".to_string())
            .add_row(vec![Value::Int(1)])
            .clear();
        
        assert_eq!(builder.col_count(), 0);
        assert_eq!(builder.row_count(), 0);
        assert!(builder.is_empty());
    }

    #[test]
    fn test_result_builder_reset() {
        let builder = ResultBuilder::new()
            .add_col_name("id".to_string())
            .add_row(vec![Value::Int(1)])
            .iterator_type(IteratorType::Default)
            .reset();
        
        assert_eq!(builder.col_count(), 0);
        assert_eq!(builder.row_count(), 0);
        assert!(builder.is_empty());
    }

    #[test]
    fn test_result_builder_chain() {
        let result = ResultBuilder::new()
            .col_names(vec!["id".to_string(), "name".to_string()])
            .add_row(vec![Value::Int(1), Value::String("Alice".to_string())])
            .add_row(vec![Value::Int(2), Value::String("Bob".to_string())])
            .iterator_type(IteratorType::Default)
            .build();
        
        assert_eq!(result.row_count(), 2);
        assert_eq!(result.col_count(), 2);
        assert!(result.iterator().is_some());
    }

    #[test]
    fn test_result_builder_default() {
        let builder = ResultBuilder::default();
        assert_eq!(builder.col_count(), 0);
        assert_eq!(builder.row_count(), 0);
    }
}
