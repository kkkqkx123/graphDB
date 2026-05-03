//! Column Store
//!
//! Columnar storage for vertex properties.
//! Each column stores values of a single property type.

use crate::core::{DataType, StorageError, StorageResult, Value};
use crate::core::value::DateValue;

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub col_id: i32,
    pub data_type: DataType,
    pub nullable: bool,
    data: Vec<u8>,
    null_bitmap: Option<Vec<bool>>,
    row_count: usize,
}

impl Column {
    pub fn new(name: String, col_id: i32, data_type: DataType, nullable: bool) -> Self {
        Self {
            name,
            col_id,
            data_type,
            nullable,
            data: Vec::new(),
            null_bitmap: if nullable { Some(BitVec::new()) } else { None },
            row_count: 0,
        }
    }

    pub fn with_capacity(name: String, col_id: i32, data_type: DataType, nullable: bool, capacity: usize) -> Self {
        let element_size = Self::element_size(&data_type);
        Self {
            name,
            col_id,
            data_type,
            nullable,
            data: Vec::with_capacity(capacity * element_size),
            null_bitmap: if nullable { Some(BitVec::with_capacity(capacity)) } else { None },
            row_count: 0,
        }
    }

    fn element_size(data_type: &DataType) -> usize {
        match data_type {
            DataType::Bool => 1,
            DataType::SmallInt => 2,
            DataType::Int => 4,
            DataType::BigInt => 8,
            DataType::Float => 4,
            DataType::Double => 8,
            DataType::Date => 12,
            DataType::Time => 8,
            DataType::Uuid => 16,
            _ => 8,
        }
    }

    pub fn set(&mut self, row_idx: usize, value: Option<&Value>) -> StorageResult<()> {
        let element_size = Self::element_size(&self.data_type);
        let offset = row_idx * element_size;

        if offset >= self.data.len() {
            self.data.resize(offset + element_size, 0);
        }

        match value {
            Some(v) => {
                self.write_value(offset, v)?;
                if let Some(ref mut bitmap) = self.null_bitmap {
                    if row_idx >= bitmap.len() {
                        bitmap.resize(row_idx + 1, false);
                    }
                    bitmap.set(row_idx, false);
                }
            }
            None => {
                if !self.nullable {
                    return Err(StorageError::NullValueNotAllowed(self.name.clone()));
                }
                if let Some(ref mut bitmap) = self.null_bitmap {
                    if row_idx >= bitmap.len() {
                        bitmap.resize(row_idx + 1, false);
                    }
                    bitmap.set(row_idx, true);
                }
            }
        }

        if row_idx >= self.row_count {
            self.row_count = row_idx + 1;
        }

        Ok(())
    }

    pub fn get(&self, row_idx: usize) -> Option<Value> {
        let element_size = Self::element_size(&self.data_type);
        let offset = row_idx * element_size;

        if let Some(ref bitmap) = self.null_bitmap {
            if row_idx < bitmap.len() && bitmap[row_idx] {
                return None;
            }
        }

        if offset + element_size > self.data.len() {
            return None;
        }

        self.read_value(offset)
    }

    fn write_value(&mut self, offset: usize, value: &Value) -> StorageResult<()> {
        match (&self.data_type, value) {
            (DataType::Bool, Value::Bool(b)) => {
                self.data[offset] = if *b { 1 } else { 0 };
            }
            (DataType::SmallInt, Value::SmallInt(i)) => {
                self.data[offset..offset + 2].copy_from_slice(&i.to_le_bytes());
            }
            (DataType::Int, Value::Int(i)) => {
                self.data[offset..offset + 4].copy_from_slice(&i.to_le_bytes());
            }
            (DataType::BigInt, Value::BigInt(i)) => {
                self.data[offset..offset + 8].copy_from_slice(&i.to_le_bytes());
            }
            (DataType::Float, Value::Float(f)) => {
                self.data[offset..offset + 4].copy_from_slice(&f.to_le_bytes());
            }
            (DataType::Double, Value::Double(d)) => {
                self.data[offset..offset + 8].copy_from_slice(&d.to_le_bytes());
            }
            (DataType::Date, Value::Date(d)) => {
                self.data[offset..offset + 4].copy_from_slice(&d.year.to_le_bytes());
                self.data[offset + 4..offset + 8].copy_from_slice(&d.month.to_le_bytes());
                self.data[offset + 8..offset + 12].copy_from_slice(&d.day.to_le_bytes());
            }
            (DataType::String, Value::String(s)) => {
                let bytes = s.as_bytes();
                let len = bytes.len() as u64;
                let start = self.data.len();
                self.data.extend_from_slice(&len.to_le_bytes());
                self.data.extend_from_slice(bytes);
            }
            _ => {
                return Err(StorageError::TypeMismatch {
                    expected: self.data_type.clone(),
                    actual: value.data_type(),
                });
            }
        }
        Ok(())
    }

    fn read_value(&self, offset: usize) -> Option<Value> {
        let element_size = Self::element_size(&self.data_type);
        if offset + element_size > self.data.len() {
            return None;
        }

        match &self.data_type {
            DataType::Bool => Some(Value::Bool(self.data[offset] != 0)),
            DataType::SmallInt => {
                let bytes: [u8; 2] = self.data[offset..offset + 2].try_into().ok()?;
                Some(Value::SmallInt(i16::from_le_bytes(bytes)))
            }
            DataType::Int => {
                let bytes: [u8; 4] = self.data[offset..offset + 4].try_into().ok()?;
                Some(Value::Int(i32::from_le_bytes(bytes)))
            }
            DataType::BigInt => {
                let bytes: [u8; 8] = self.data[offset..offset + 8].try_into().ok()?;
                Some(Value::BigInt(i64::from_le_bytes(bytes)))
            }
            DataType::Float => {
                let bytes: [u8; 4] = self.data[offset..offset + 4].try_into().ok()?;
                Some(Value::Float(f32::from_le_bytes(bytes)))
            }
            DataType::Double => {
                let bytes: [u8; 8] = self.data[offset..offset + 8].try_into().ok()?;
                Some(Value::Double(f64::from_le_bytes(bytes)))
            }
            DataType::Date => {
                let year_bytes: [u8; 4] = self.data[offset..offset + 4].try_into().ok()?;
                let month_bytes: [u8; 4] = self.data[offset + 4..offset + 8].try_into().ok()?;
                let day_bytes: [u8; 4] = self.data[offset + 8..offset + 12].try_into().ok()?;
                Some(Value::Date(DateValue {
                    year: i32::from_le_bytes(year_bytes),
                    month: u32::from_le_bytes(month_bytes),
                    day: u32::from_le_bytes(day_bytes),
                }))
            }
            _ => None,
        }
    }

    pub fn is_null(&self, row_idx: usize) -> bool {
        self.null_bitmap
            .as_ref()
            .map(|b| row_idx < b.len() && b[row_idx])
            .unwrap_or(false)
    }

    pub fn len(&self) -> usize {
        self.row_count
    }

    pub fn is_empty(&self) -> bool {
        self.row_count == 0
    }

    pub fn data_size(&self) -> usize {
        self.data.len()
    }

    pub fn clear(&mut self) {
        self.data.clear();
        if let Some(ref mut bitmap) = self.null_bitmap {
            bitmap.clear();
        }
        self.row_count = 0;
    }

    pub fn resize(&mut self, new_count: usize) {
        let element_size = Self::element_size(&self.data_type);
        self.data.resize(new_count * element_size, 0);
        if let Some(ref mut bitmap) = self.null_bitmap {
            bitmap.resize(new_count, false);
        }
        self.row_count = new_count;
    }
}

#[derive(Debug, Clone)]
pub struct ColumnStore {
    columns: Vec<Column>,
    name_to_index: std::collections::HashMap<String, usize>,
}

impl ColumnStore {
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            name_to_index: std::collections::HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            columns: Vec::with_capacity(capacity),
            name_to_index: std::collections::HashMap::with_capacity(capacity),
        }
    }

    pub fn add_column(&mut self, name: String, data_type: DataType, nullable: bool) -> i32 {
        let col_id = self.columns.len() as i32;
        let column = Column::new(name.clone(), col_id, data_type, nullable);
        self.name_to_index.insert(name, self.columns.len());
        self.columns.push(column);
        col_id
    }

    pub fn get_column(&self, name: &str) -> Option<&Column> {
        self.name_to_index.get(name).and_then(|&idx| self.columns.get(idx))
    }

    pub fn get_column_mut(&mut self, name: &str) -> Option<&mut Column> {
        self.name_to_index.get(name).and_then(|&idx| self.columns.get_mut(idx))
    }

    pub fn get_column_by_id(&self, col_id: i32) -> Option<&Column> {
        self.columns.get(col_id as usize)
    }

    pub fn get_column_by_id_mut(&mut self, col_id: i32) -> Option<&mut Column> {
        self.columns.get_mut(col_id as usize)
    }

    pub fn set(&mut self, row_idx: usize, values: &[(String, Value)]) -> StorageResult<()> {
        for (name, value) in values {
            if let Some(col) = self.get_column_mut(name) {
                col.set(row_idx, Some(value))?;
            }
        }
        Ok(())
    }

    pub fn get(&self, row_idx: usize) -> Vec<(String, Option<Value>)> {
        self.columns
            .iter()
            .map(|col| (col.name.clone(), col.get(row_idx)))
            .collect()
    }

    pub fn get_property(&self, row_idx: usize, col_name: &str) -> Option<Value> {
        self.get_column(col_name)?.get(row_idx)
    }

    pub fn set_property(&mut self, row_idx: usize, col_name: &str, value: Option<&Value>) -> StorageResult<()> {
        let col = self.get_column_mut(col_name)
            .ok_or_else(|| StorageError::ColumnNotFound(col_name.to_string()))?;
        col.set(row_idx, value)
    }

    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    pub fn row_count(&self) -> usize {
        self.columns.first().map(|c| c.len()).unwrap_or(0)
    }

    pub fn clear(&mut self) {
        for col in &mut self.columns {
            col.clear();
        }
    }

    pub fn resize(&mut self, new_count: usize) {
        for col in &mut self.columns {
            col.resize(new_count);
        }
    }

    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    pub fn column_names(&self) -> Vec<&str> {
        self.columns.iter().map(|c| c.name.as_str()).collect()
    }
}

impl Default for ColumnStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_basic() {
        let mut col = Column::new("age".to_string(), 0, DataType::Int, true);

        col.set(0, Some(&Value::Int(25))).unwrap();
        col.set(1, Some(&Value::Int(30))).unwrap();
        col.set(2, None).unwrap();

        assert_eq!(col.get(0), Some(Value::Int(25)));
        assert_eq!(col.get(1), Some(Value::Int(30)));
        assert!(col.is_null(2));
        assert_eq!(col.len(), 3);
    }

    #[test]
    fn test_column_store() {
        let mut store = ColumnStore::new();

        store.add_column("name".to_string(), DataType::String, false);
        store.add_column("age".to_string(), DataType::Int, true);

        store.set(0, &[
            ("name".to_string(), Value::String("Alice".to_string())),
            ("age".to_string(), Value::Int(30)),
        ]).unwrap();

        store.set(1, &[
            ("name".to_string(), Value::String("Bob".to_string())),
            ("age".to_string(), Value::Int(25)),
        ]).unwrap();

        assert_eq!(store.get_property(0, "age"), Some(Value::Int(30)));
        assert_eq!(store.get_property(1, "name"), Some(Value::String("Bob".to_string())));
    }
}
