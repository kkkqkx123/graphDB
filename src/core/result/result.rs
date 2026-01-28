use crate::core::value::Value;
use crate::core::result::result_iterator::ResultIterator;
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
    iterator: Option<Arc<dyn ResultIterator<'static, Vec<Value>, Row = Vec<Value>>>>,
}

impl Result {
    /// 创建新的空 Result
    ///
    /// # 示例
    ///
    /// ```rust
    /// use graphdb::core::result::Result;
    ///
    /// let result = Result::new();
    /// assert!(result.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            col_names: Vec::new(),
            meta: ResultMeta::default(),
            iterator: None,
        }
    }

    /// 内部构造函数，供 ResultBuilder 使用
    pub(crate) fn from_builder(
        rows: Vec<Vec<Value>>,
        col_names: Vec<String>,
        state: ResultState,
        iterator: Option<Arc<dyn ResultIterator<'static, Vec<Value>, Row = Vec<Value>>>>,
    ) -> Self {
        let row_count = rows.len();
        let col_count = col_names.len();

        Self {
            rows,
            col_names,
            meta: ResultMeta {
                row_count,
                col_count,
                state,
                memory_usage: 0,
            },
            iterator,
        }
    }

    /// 从行集合和列名创建 Result
    ///
    /// 此方法是创建 Result 的推荐方式，自动设置状态为 Completed
    /// 并计算内存使用量
    pub fn from_rows(rows: Vec<Vec<Value>>, col_names: Vec<String>) -> Self {
        let row_count = rows.len();
        let col_count = col_names.len();

        Self {
            rows,
            col_names,
            meta: ResultMeta {
                row_count,
                col_count,
                state: ResultState::Completed,
                ..Default::default()
            },
            iterator: None,
        }
    }

    /// 创建空结果集（带有指定的列名）
    pub fn empty(col_names: Vec<String>) -> Self {
        let col_count = col_names.len();
        Self {
            rows: Vec::new(),
            col_names,
            meta: ResultMeta {
                row_count: 0,
                col_count,
                state: ResultState::Completed,
                ..Default::default()
            },
            iterator: None,
        }
    }

    pub fn col_names(&self) -> &[String] {
        &self.col_names
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

    pub fn get_row(&self, index: usize) -> Option<&Vec<Value>> {
        self.rows.get(index)
    }

    pub fn get_value(&self, row: usize, col: usize) -> Option<&Value> {
        self.rows.get(row).and_then(|r| r.get(col))
    }

    pub fn iterator(&self) -> Option<&Arc<dyn ResultIterator<'static, Vec<Value>, Row = Vec<Value>>>> {
        self.iterator.as_ref()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn meta(&self) -> &ResultMeta {
        &self.meta
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
    fn test_result_empty() {
        let col_names = vec!["id".to_string()];
        let result = Result::empty(col_names.clone());
        
        assert_eq!(result.col_names(), &col_names);
        assert_eq!(result.row_count(), 0);
        assert!(result.is_empty());
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
