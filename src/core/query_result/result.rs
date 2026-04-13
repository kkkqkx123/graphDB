use crate::core::value::Value;
use crate::utils::output::{Format, Result as OutputResult};

/// Result Status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultState {
    NotStarted,
    InProgress,
    Completed,
    Failed,
}

/// Result Metadata
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

/// Result struct
///
/// Nebula-Graph-based Result design, using Rust's type system and memory safety features
///
/// # Characteristics
/// - Zero-cost abstraction: compile-time optimizations, no runtime overheads
/// - Type safety: compile-time type checking
/// - Memory Safety: Rust Ownership System Guarantees
/// - Efficient Iteration: Support for multiple iterator types via rows() method
#[derive(Debug, Clone)]
pub struct Result {
    rows: Vec<Vec<Value>>,
    col_names: Vec<String>,
    meta: ResultMeta,
}

impl Result {
    /// Create a new empty Result
    ///
    /// # Examples
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
        }
    }

    /// Creating Results from Row Sets and Column Names
    ///
    /// This method is the recommended way to create a Result and automatically sets the status to Completed.
    /// and calculate the memory usage
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
        }
    }

    /// Creating an empty result set (with the specified column names)
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

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn meta(&self) -> &ResultMeta {
        &self.meta
    }

    /// Convert result to output string based on format
    pub fn to_output(&self, format: Format) -> OutputResult<String> {
        match format {
            Format::Plain => self.to_plain_string(),
            Format::Json => self.to_json_string(),
            Format::Table => self.to_table_string(),
        }
    }

    /// Convert result to plain string format
    fn to_plain_string(&self) -> OutputResult<String> {
        let mut output = String::new();

        // Column names
        if !self.col_names.is_empty() {
            output.push_str(&self.col_names.join("\t"));
            output.push('\n');
        }

        // Rows
        for row in &self.rows {
            let row_str: Vec<String> = row.iter().map(|v| format!("{}", v)).collect();
            output.push_str(&row_str.join("\t"));
            output.push('\n');
        }

        Ok(output)
    }

    /// Convert result to JSON string
    fn to_json_string(&self) -> OutputResult<String> {
        use serde::Serialize;
        use std::collections::HashMap;

        #[derive(Serialize)]
        struct RowData {
            #[serde(flatten)]
            data: HashMap<String, Value>,
        }

        #[derive(Serialize)]
        struct ResultData {
            columns: Vec<String>,
            rows: Vec<RowData>,
            row_count: usize,
        }

        let rows: Vec<RowData> = self
            .rows
            .iter()
            .map(|row| {
                let mut data = HashMap::new();
                for (i, value) in row.iter().enumerate() {
                    if let Some(col_name) = self.col_names.get(i) {
                        data.insert(col_name.clone(), value.clone());
                    }
                }
                RowData { data }
            })
            .collect();

        let result_data = ResultData {
            columns: self.col_names.clone(),
            rows,
            row_count: self.rows.len(),
        };

        crate::utils::output::to_json_string(&result_data)
    }

    /// Convert result to table format
    fn to_table_string(&self) -> OutputResult<String> {
        use crate::utils::output::TableFormatter;

        let mut formatter = TableFormatter::new();

        if !self.col_names.is_empty() {
            let col_names: Vec<&str> = self.col_names.iter().map(|s| s.as_str()).collect();
            formatter.set_headers(&col_names);
        }

        for row in &self.rows {
            let row_str: Vec<String> = row.iter().map(|v| format!("{}", v)).collect();
            formatter.add_row_strings(row_str);
        }

        formatter.render_to_string()
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
        assert_eq!(row.expect("Expected row to exist")[0], Value::Int(1));
    }

    #[test]
    fn test_result_get_value() {
        let mut result = Result::new();
        result.add_row(vec![Value::Int(1), Value::String("Alice".to_string())]);

        let value = result.get_value(0, 1);
        assert!(value.is_some());
        assert_eq!(
            value.expect("Expected value to exist"),
            &Value::String("Alice".to_string())
        );
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
