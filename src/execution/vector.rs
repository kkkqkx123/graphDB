//! Vectorized Execution Engine
//!
//! Provides batch processing capabilities for improved CPU cache locality
//! and SIMD optimization potential. Follows DuckDB's vectorized execution model.
//!
//! ## Arena Allocation
//!
//! For high-performance scenarios with many temporary allocations, use
//! `ArenaVectorBatch` which leverages `Arena` for efficient
//! batch memory management. This is particularly useful for:
//!
//! - Query execution with intermediate results
//! - Expression evaluation with temporary values
//! - Batch processing pipelines
//!
//! ## Example
//!
//! ```rust,ignore
//! use graphdb::execution::ArenaVectorBatch;
//! use graphdb::utils::Arena;
//!
//! // Create arena-backed batch for temporary allocations
//! let arena = Arena::new();
//! let mut batch = ArenaVectorBatch::new(&arena, 3);
//!
//! // All allocations come from the arena
//! batch.push_row(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
//!
//! // Arena can be reset for reuse
//! arena.reset();
//! ```

use crate::core::{DataType, StorageError, StorageResult, Value};
use crate::query::planning::plan::core::nodes::base::memory_estimation::MemoryEstimatable;
use crate::utils::Arena;
use bitvec::prelude::{BitVec, Lsb0};

pub const VECTOR_BATCH_SIZE: usize = 2048;

#[derive(Debug, Clone)]
pub struct VectorBatch {
    columns: Vec<VectorColumn>,
    row_count: usize,
    selection_vector: Option<Vec<usize>>,
}

impl VectorBatch {
    pub fn new(column_count: usize) -> Self {
        Self {
            columns: vec![VectorColumn::default(); column_count],
            row_count: 0,
            selection_vector: None,
        }
    }

    pub fn with_capacity(column_count: usize, capacity: usize) -> Self {
        Self {
            columns: (0..column_count)
                .map(|_| VectorColumn::with_capacity(capacity))
                .collect(),
            row_count: 0,
            selection_vector: None,
        }
    }

    pub fn from_columns(columns: Vec<VectorColumn>) -> Self {
        let row_count = columns.first().map_or(0, |c| c.len());
        Self {
            columns,
            row_count,
            selection_vector: None,
        }
    }

    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    pub fn row_count(&self) -> usize {
        self.row_count
    }

    pub fn column(&self, idx: usize) -> Option<&VectorColumn> {
        self.columns.get(idx)
    }

    pub fn column_mut(&mut self, idx: usize) -> Option<&mut VectorColumn> {
        self.columns.get_mut(idx)
    }

    pub fn columns(&self) -> &[VectorColumn] {
        &self.columns
    }

    pub fn columns_mut(&mut self) -> &mut [VectorColumn] {
        &mut self.columns
    }

    pub fn set_selection(&mut self, selection: Vec<usize>) {
        self.selection_vector = Some(selection);
    }

    pub fn clear_selection(&mut self) {
        self.selection_vector = None;
    }

    pub fn selection(&self) -> Option<&[usize]> {
        self.selection_vector.as_deref()
    }

    pub fn selected_count(&self) -> usize {
        self.selection_vector.as_ref().map_or(self.row_count, |s| s.len())
    }

    pub fn append_row(&mut self, values: Vec<Value>) -> StorageResult<()> {
        if values.len() != self.columns.len() {
            return Err(StorageError::invalid_operation(format!(
                "Expected {} values, got {}",
                self.columns.len(),
                values.len()
            )));
        }

        for (col, value) in self.columns.iter_mut().zip(values.into_iter()) {
            col.push(value);
        }

        self.row_count += 1;
        Ok(())
    }

    pub fn get_row(&self, row_idx: usize) -> Option<Vec<Value>> {
        if row_idx >= self.row_count {
            return None;
        }

        Some(self.columns.iter().map(|col| col.get(row_idx)).collect())
    }

    pub fn slice(&self, start: usize, count: usize) -> StorageResult<Self> {
        if start + count > self.row_count {
            return Err(StorageError::invalid_operation(
                "Slice out of bounds".to_string(),
            ));
        }

        let columns = self
            .columns
            .iter()
            .map(|col| col.slice(start, count))
            .collect();

        Ok(Self {
            columns,
            row_count: count,
            selection_vector: None,
        })
    }

    pub fn clear(&mut self) {
        for col in &mut self.columns {
            col.clear();
        }
        self.row_count = 0;
        self.selection_vector = None;
    }

    pub fn memory_usage(&self) -> usize {
        self.columns.iter().map(|c| c.memory_usage()).sum()
    }
}

impl Default for VectorBatch {
    fn default() -> Self {
        Self::new(0)
    }
}

#[derive(Debug, Clone)]
pub struct VectorColumn {
    data_type: DataType,
    values: Vec<Value>,
    null_mask: BitVec<u8, Lsb0>,
}

impl VectorColumn {
    pub fn new(data_type: DataType) -> Self {
        Self {
            data_type,
            values: Vec::new(),
            null_mask: BitVec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data_type: DataType::Empty,
            values: Vec::with_capacity(capacity),
            null_mask: BitVec::with_capacity(capacity),
        }
    }

    pub fn from_values(values: Vec<Value>) -> Self {
        let data_type = values.first().map_or(DataType::Empty, |v| v.get_type());
        let len = values.len();
        let mut null_mask = BitVec::repeat(false, len);

        for (i, value) in values.iter().enumerate() {
            if value.is_null() {
                null_mask.set(i, true);
            }
        }

        Self {
            data_type,
            values,
            null_mask,
        }
    }

    pub fn data_type(&self) -> &DataType {
        &self.data_type
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn push(&mut self, value: Value) {
        let is_null = value.is_null();
        self.values.push(value);
        self.null_mask.push(is_null);
    }

    pub fn get(&self, idx: usize) -> Value {
        self.values.get(idx).cloned().unwrap_or(Value::Empty)
    }

    pub fn set(&mut self, idx: usize, value: Value) -> StorageResult<()> {
        if idx >= self.values.len() {
            return Err(StorageError::invalid_operation("Index out of bounds".to_string()));
        }

        self.values[idx] = value.clone();
        self.null_mask.set(idx, value.is_null());
        Ok(())
    }

    pub fn is_null(&self, idx: usize) -> bool {
        self.null_mask.get(idx).is_none_or(|b| *b)
    }

    pub fn null_count(&self) -> usize {
        self.null_mask.count_ones()
    }

    pub fn null_mask(&self) -> &BitVec<u8, Lsb0> {
        &self.null_mask
    }

    pub fn values(&self) -> &[Value] {
        &self.values
    }

    pub fn values_mut(&mut self) -> &mut Vec<Value> {
        &mut self.values
    }

    pub fn slice(&self, start: usize, count: usize) -> Self {
        let end = (start + count).min(self.values.len());
        let values = self.values[start..end].to_vec();
        let mut null_mask = BitVec::new();
        for i in start..end {
            null_mask.push(self.null_mask.get(i).is_none_or(|b| *b));
        }

        Self {
            data_type: self.data_type.clone(),
            values,
            null_mask,
        }
    }

    pub fn clear(&mut self) {
        self.values.clear();
        self.null_mask.clear();
    }

    pub fn memory_usage(&self) -> usize {
        let values_size: usize = self.values.iter().map(|v| v.estimate_memory()).sum();
        let mask_size = self.null_mask.as_raw_slice().len();
        values_size + mask_size
    }

    pub fn get_int32_slice(&self) -> Option<Vec<i32>> {
        self.values
            .iter()
            .map(|v| {
                if let Value::Int(i) = v {
                    Some(*i)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_int64_slice(&self) -> Option<Vec<i64>> {
        self.values
            .iter()
            .map(|v| {
                if let Value::BigInt(i) = v {
                    Some(*i)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_float64_slice(&self) -> Option<Vec<f64>> {
        self.values
            .iter()
            .map(|v| {
                if let Value::Double(d) = v {
                    Some(*d)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_string_slice(&self) -> Option<Vec<&str>> {
        self.values
            .iter()
            .map(|v| {
                if let Value::String(s) = v {
                    Some(s.as_str())
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for VectorColumn {
    fn default() -> Self {
        Self::new(DataType::Empty)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    And,
    Or,
    Not,
}

pub struct VectorProcessor;

impl VectorProcessor {
    pub fn execute_binary_op(
        left: &VectorColumn,
        right: &VectorColumn,
        op: VectorOperation,
    ) -> StorageResult<VectorColumn> {
        if left.len() != right.len() {
            return Err(StorageError::invalid_operation(
                "Column length mismatch".to_string(),
            ));
        }

        let mut result = VectorColumn::with_capacity(left.len());

        for i in 0..left.len() {
            let left_val = left.get(i);
            let right_val = right.get(i);

            let result_val = Self::apply_binary_op(&left_val, &right_val, op)?;
            result.push(result_val);
        }

        Ok(result)
    }

    pub fn execute_unary_op(input: &VectorColumn, op: VectorOperation) -> StorageResult<VectorColumn> {
        let mut result = VectorColumn::with_capacity(input.len());

        for i in 0..input.len() {
            let val = input.get(i);
            let result_val = Self::apply_unary_op(&val, op)?;
            result.push(result_val);
        }

        Ok(result)
    }

    fn apply_binary_op(left: &Value, right: &Value, op: VectorOperation) -> StorageResult<Value> {
        use VectorOperation::*;

        match op {
            Add => Self::add_values(left, right),
            Subtract => Self::subtract_values(left, right),
            Multiply => Self::multiply_values(left, right),
            Divide => Self::divide_values(left, right),
            Modulo => Self::modulo_values(left, right),
            Equal => Ok(Value::Bool(left == right)),
            NotEqual => Ok(Value::Bool(left != right)),
            LessThan => Ok(Value::Bool(left < right)),
            LessThanEqual => Ok(Value::Bool(left <= right)),
            GreaterThan => Ok(Value::Bool(left > right)),
            GreaterThanEqual => Ok(Value::Bool(left >= right)),
            And => Self::and_values(left, right),
            Or => Self::or_values(left, right),
            Not => Self::not_values(left),
        }
    }

    fn apply_unary_op(value: &Value, op: VectorOperation) -> StorageResult<Value> {
        if op == VectorOperation::Not {
            Self::not_values(value)
        } else {
            Err(StorageError::invalid_operation(format!(
                "Invalid unary operation: {:?}",
                op
            )))
        }
    }

    fn add_values(left: &Value, right: &Value) -> StorageResult<Value> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::BigInt(a), Value::BigInt(b)) => Ok(Value::BigInt(a + b)),
            (Value::Double(a), Value::Double(b)) => Ok(Value::Double(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            _ => Err(StorageError::invalid_operation(format!(
                "Cannot add {:?} and {:?}",
                left.get_type(),
                right.get_type()
            ))),
        }
    }

    fn subtract_values(left: &Value, right: &Value) -> StorageResult<Value> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::BigInt(a), Value::BigInt(b)) => Ok(Value::BigInt(a - b)),
            (Value::Double(a), Value::Double(b)) => Ok(Value::Double(a - b)),
            _ => Err(StorageError::invalid_operation(format!(
                "Cannot subtract {:?} and {:?}",
                left.get_type(),
                right.get_type()
            ))),
        }
    }

    fn multiply_values(left: &Value, right: &Value) -> StorageResult<Value> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (Value::BigInt(a), Value::BigInt(b)) => Ok(Value::BigInt(a * b)),
            (Value::Double(a), Value::Double(b)) => Ok(Value::Double(a * b)),
            _ => Err(StorageError::invalid_operation(format!(
                "Cannot multiply {:?} and {:?}",
                left.get_type(),
                right.get_type()
            ))),
        }
    }

    fn divide_values(left: &Value, right: &Value) -> StorageResult<Value> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    return Err(StorageError::invalid_operation("Division by zero".to_string()));
                }
                Ok(Value::Int(a / b))
            }
            (Value::BigInt(a), Value::BigInt(b)) => {
                if *b == 0 {
                    return Err(StorageError::invalid_operation("Division by zero".to_string()));
                }
                Ok(Value::BigInt(a / b))
            }
            (Value::Double(a), Value::Double(b)) => {
                if *b == 0.0 {
                    return Err(StorageError::invalid_operation("Division by zero".to_string()));
                }
                Ok(Value::Double(a / b))
            }
            _ => Err(StorageError::invalid_operation(format!(
                "Cannot divide {:?} and {:?}",
                left.get_type(),
                right.get_type()
            ))),
        }
    }

    fn modulo_values(left: &Value, right: &Value) -> StorageResult<Value> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    return Err(StorageError::invalid_operation("Modulo by zero".to_string()));
                }
                Ok(Value::Int(a % b))
            }
            (Value::BigInt(a), Value::BigInt(b)) => {
                if *b == 0 {
                    return Err(StorageError::invalid_operation("Modulo by zero".to_string()));
                }
                Ok(Value::BigInt(a % b))
            }
            _ => Err(StorageError::invalid_operation(format!(
                "Cannot modulo {:?} and {:?}",
                left.get_type(),
                right.get_type()
            ))),
        }
    }

    fn and_values(left: &Value, right: &Value) -> StorageResult<Value> {
        match (left, right) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a && *b)),
            _ => Err(StorageError::invalid_operation(
                "AND operation requires boolean values".to_string(),
            )),
        }
    }

    fn or_values(left: &Value, right: &Value) -> StorageResult<Value> {
        match (left, right) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a || *b)),
            _ => Err(StorageError::invalid_operation(
                "OR operation requires boolean values".to_string(),
            )),
        }
    }

    fn not_values(value: &Value) -> StorageResult<Value> {
        match value {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            _ => Err(StorageError::invalid_operation(
                "NOT operation requires boolean value".to_string(),
            )),
        }
    }
}

pub struct VectorSelector;

impl VectorSelector {
    pub fn select_equals(column: &VectorColumn, value: &Value) -> Vec<usize> {
        column
            .values()
            .iter()
            .enumerate()
            .filter_map(|(i, v)| if v == value { Some(i) } else { None })
            .collect()
    }

    pub fn select_not_equals(column: &VectorColumn, value: &Value) -> Vec<usize> {
        column
            .values()
            .iter()
            .enumerate()
            .filter_map(|(i, v)| if v != value { Some(i) } else { None })
            .collect()
    }

    pub fn select_less_than(column: &VectorColumn, value: &Value) -> Vec<usize> {
        column
            .values()
            .iter()
            .enumerate()
            .filter_map(|(i, v)| if v < value { Some(i) } else { None })
            .collect()
    }

    pub fn select_less_than_equal(column: &VectorColumn, value: &Value) -> Vec<usize> {
        column
            .values()
            .iter()
            .enumerate()
            .filter_map(|(i, v)| if v <= value { Some(i) } else { None })
            .collect()
    }

    pub fn select_greater_than(column: &VectorColumn, value: &Value) -> Vec<usize> {
        column
            .values()
            .iter()
            .enumerate()
            .filter_map(|(i, v)| if v > value { Some(i) } else { None })
            .collect()
    }

    pub fn select_greater_than_equal(column: &VectorColumn, value: &Value) -> Vec<usize> {
        column
            .values()
            .iter()
            .enumerate()
            .filter_map(|(i, v)| if v >= value { Some(i) } else { None })
            .collect()
    }

    pub fn select_null(column: &VectorColumn) -> Vec<usize> {
        column
            .values()
            .iter()
            .enumerate()
            .filter_map(|(i, v)| if v.is_null() { Some(i) } else { None })
            .collect()
    }

    pub fn select_not_null(column: &VectorColumn) -> Vec<usize> {
        column
            .values()
            .iter()
            .enumerate()
            .filter_map(|(i, v)| if !v.is_null() { Some(i) } else { None })
            .collect()
    }

    pub fn select_in(column: &VectorColumn, values: &[Value]) -> Vec<usize> {
        column
            .values()
            .iter()
            .enumerate()
            .filter_map(|(i, v)| {
                if values.contains(v) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn select_range(column: &VectorColumn, min: &Value, max: &Value) -> Vec<usize> {
        column
            .values()
            .iter()
            .enumerate()
            .filter_map(|(i, v)| if v >= min && v <= max { Some(i) } else { None })
            .collect()
    }

    pub fn apply_selection(batch: &VectorBatch, selection: &[usize]) -> StorageResult<VectorBatch> {
        let mut columns = Vec::with_capacity(batch.column_count());

        for col in batch.columns() {
            let mut new_col = VectorColumn::with_capacity(selection.len());
            for &idx in selection {
                if idx < col.len() {
                    new_col.push(col.get(idx));
                }
            }
            columns.push(new_col);
        }

        Ok(VectorBatch::from_columns(columns))
    }
}

/// Arena-backed vector batch for high-performance temporary allocations.
///
/// This structure can be used with an arena for allocating temporary values,
/// while the column storage is managed normally. The arena reference is kept
/// for potential future use in allocating temporary computation results.
pub struct ArenaVectorBatch<'a> {
    arena: &'a Arena,
    columns: Vec<VectorColumn>,
    selection: Option<Vec<usize>>,
}

impl<'a> ArenaVectorBatch<'a> {
    pub fn new(arena: &'a Arena, column_count: usize) -> Self {
        let mut columns = Vec::with_capacity(column_count);
        for _ in 0..column_count {
            columns.push(VectorColumn::default());
        }

        Self {
            arena,
            columns,
            selection: None,
        }
    }

    pub fn with_capacity(arena: &'a Arena, column_count: usize, capacity: usize) -> Self {
        let mut columns = Vec::with_capacity(column_count);
        for _ in 0..column_count {
            columns.push(VectorColumn::with_capacity(capacity));
        }

        Self {
            arena,
            columns,
            selection: None,
        }
    }

    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    pub fn row_count(&self) -> usize {
        self.columns.first().map_or(0, |c| c.len())
    }

    pub fn push_row(&mut self, values: Vec<Value>) -> StorageResult<()> {
        if values.len() != self.columns.len() {
            return Err(StorageError::invalid_operation(format!(
                "Expected {} values, got {}",
                self.columns.len(),
                values.len()
            )));
        }

        for (col, value) in self.columns.iter_mut().zip(values.into_iter()) {
            col.push(value);
        }

        Ok(())
    }

    pub fn get_row(&self, row_idx: usize) -> Option<Vec<Value>> {
        let row_count = self.row_count();
        if row_idx >= row_count {
            return None;
        }

        Some(self.columns.iter().map(|col| col.get(row_idx)).collect())
    }

    pub fn column(&self, idx: usize) -> Option<&VectorColumn> {
        self.columns.get(idx)
    }

    pub fn column_mut(&mut self, idx: usize) -> Option<&mut VectorColumn> {
        self.columns.get_mut(idx)
    }

    pub fn set_selection(&mut self, selection: Vec<usize>) {
        self.selection = Some(selection);
    }

    pub fn selection(&self) -> Option<&[usize]> {
        self.selection.as_deref()
    }

    pub fn clear(&mut self) {
        for col in self.columns.iter_mut() {
            col.clear();
        }
        self.selection = None;
    }

    pub fn memory_usage(&self) -> usize {
        self.columns.iter().map(|c| c.memory_usage()).sum()
    }

    pub fn to_vector_batch(&self) -> VectorBatch {
        VectorBatch::from_columns(self.columns.clone())
    }

    pub fn arena(&self) -> &Arena {
        self.arena
    }
}

/// Arena-backed selection vector for efficient filtering operations.
pub struct ArenaSelectionVector<'a> {
    arena: &'a Arena,
    data: Vec<usize>,
}

impl<'a> ArenaSelectionVector<'a> {
    pub fn new(arena: &'a Arena, capacity: usize) -> Self {
        Self {
            arena,
            data: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, value: usize) {
        self.data.push(value);
    }

    pub fn as_slice(&self) -> &[usize] {
        &self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn extend_from_slice(&mut self, values: &[usize]) {
        self.data.extend_from_slice(values);
    }

    pub fn arena(&self) -> &Arena {
        self.arena
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_batch_creation() {
        let batch = VectorBatch::new(3);
        assert_eq!(batch.column_count(), 3);
        assert_eq!(batch.row_count(), 0);
    }

    #[test]
    fn test_vector_batch_append_row() {
        let mut batch = VectorBatch::new(2);

        batch
            .append_row(vec![Value::Int(1), Value::String("a".to_string())])
            .expect("Append failed");
        batch
            .append_row(vec![Value::Int(2), Value::String("b".to_string())])
            .expect("Append failed");

        assert_eq!(batch.row_count(), 2);

        let row = batch.get_row(0).expect("Get row failed");
        assert_eq!(row[0], Value::Int(1));
        assert_eq!(row[1], Value::String("a".to_string()));
    }

    #[test]
    fn test_vector_column_operations() {
        let mut col = VectorColumn::new(DataType::Int);

        col.push(Value::Int(1));
        col.push(Value::Int(2));
        col.push(Value::Int(3));

        assert_eq!(col.len(), 3);
        assert_eq!(col.get(1), Value::Int(2));
        assert_eq!(col.null_count(), 0);
    }

    #[test]
    fn test_vector_column_null_handling() {
        let mut col = VectorColumn::new(DataType::Int);

        col.push(Value::Int(1));
        col.push(Value::Null(crate::core::value::null::NullType::Null));
        col.push(Value::Int(3));

        assert_eq!(col.len(), 3);
        assert_eq!(col.null_count(), 1);
        assert!(!col.is_null(0));
        assert!(col.is_null(1));
        assert!(!col.is_null(2));
    }

    #[test]
    fn test_vector_processor_add() {
        let mut left = VectorColumn::new(DataType::Int);
        left.push(Value::Int(1));
        left.push(Value::Int(2));
        left.push(Value::Int(3));

        let mut right = VectorColumn::new(DataType::Int);
        right.push(Value::Int(10));
        right.push(Value::Int(20));
        right.push(Value::Int(30));

        let result = VectorProcessor::execute_binary_op(&left, &right, VectorOperation::Add)
            .expect("Operation failed");

        assert_eq!(result.get(0), Value::Int(11));
        assert_eq!(result.get(1), Value::Int(22));
        assert_eq!(result.get(2), Value::Int(33));
    }

    #[test]
    fn test_vector_processor_compare() {
        let mut col = VectorColumn::new(DataType::Int);
        col.push(Value::Int(1));
        col.push(Value::Int(2));
        col.push(Value::Int(3));

        let value = Value::Int(2);

        let result = VectorSelector::select_equals(&col, &value);
        assert_eq!(result, vec![1]);

        let result = VectorSelector::select_less_than(&col, &value);
        assert_eq!(result, vec![0]);

        let result = VectorSelector::select_greater_than(&col, &value);
        assert_eq!(result, vec![2]);
    }

    #[test]
    fn test_vector_batch_slice() {
        let mut batch = VectorBatch::new(2);

        for i in 0..10 {
            batch
                .append_row(vec![Value::Int(i), Value::String(format!("val_{}", i))])
                .expect("Append failed");
        }

        let sliced = batch.slice(2, 3).expect("Slice failed");

        assert_eq!(sliced.row_count(), 3);
        assert_eq!(sliced.get_row(0), Some(vec![Value::Int(2), Value::String("val_2".to_string())]));
    }

    #[test]
    fn test_vector_selector_apply() {
        let mut batch = VectorBatch::new(2);

        for i in 0..5 {
            batch
                .append_row(vec![Value::Int(i), Value::String(format!("val_{}", i))])
                .expect("Append failed");
        }

        let selection = vec![0, 2, 4];
        let filtered = VectorSelector::apply_selection(&batch, &selection).expect("Apply failed");

        assert_eq!(filtered.row_count(), 3);
        assert_eq!(filtered.get_row(0), Some(vec![Value::Int(0), Value::String("val_0".to_string())]));
        assert_eq!(filtered.get_row(1), Some(vec![Value::Int(2), Value::String("val_2".to_string())]));
        assert_eq!(filtered.get_row(2), Some(vec![Value::Int(4), Value::String("val_4".to_string())]));
    }

    #[test]
    fn test_vector_column_memory_usage() {
        let mut col = VectorColumn::new(DataType::Int);
        for i in 0..100 {
            col.push(Value::Int(i));
        }

        let usage = col.memory_usage();
        assert!(usage > 0);
    }

    #[test]
    fn test_vector_batch_selection() {
        let mut batch = VectorBatch::new(1);

        for i in 0..10 {
            batch.append_row(vec![Value::Int(i)]).expect("Append failed");
        }

        let selection = vec![1, 3, 5, 7];
        batch.set_selection(selection.clone());

        assert_eq!(batch.selected_count(), 4);
        assert_eq!(batch.selection(), Some(selection.as_slice()));
    }

    #[test]
    fn test_arena_vector_batch() {
        let arena = Arena::new();
        let mut batch = ArenaVectorBatch::new(&arena, 2);

        batch
            .push_row(vec![Value::Int(1), Value::String("a".to_string())])
            .expect("Push failed");
        batch
            .push_row(vec![Value::Int(2), Value::String("b".to_string())])
            .expect("Push failed");

        assert_eq!(batch.row_count(), 2);

        let row = batch.get_row(0).expect("Get row failed");
        assert_eq!(row[0], Value::Int(1));
        assert_eq!(row[1], Value::String("a".to_string()));
    }

    #[test]
    fn test_arena_selection_vector() {
        let arena = Arena::new();
        let mut sel = ArenaSelectionVector::new(&arena, 100);

        sel.push(1);
        sel.push(3);
        sel.push(5);

        assert_eq!(sel.len(), 3);
        assert_eq!(sel.as_slice(), &[1, 3, 5]);

        sel.clear();
        assert!(sel.is_empty());
    }

    #[test]
    fn test_arena_batch_to_vector_batch() {
        let arena = Arena::new();
        let mut arena_batch = ArenaVectorBatch::new(&arena, 2);

        arena_batch
            .push_row(vec![Value::Int(1), Value::Int(10)])
            .expect("Push failed");
        arena_batch
            .push_row(vec![Value::Int(2), Value::Int(20)])
            .expect("Push failed");

        let batch = arena_batch.to_vector_batch();
        assert_eq!(batch.row_count(), 2);
        assert_eq!(batch.column_count(), 2);
    }
}
