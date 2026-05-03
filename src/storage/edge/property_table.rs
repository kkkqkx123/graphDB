//! Property Table for Edges
//!
//! Stores edge properties in a row-oriented format.

use std::collections::HashMap;

use crate::core::{DataType, StorageError, StorageResult, Value};

#[derive(Debug, Clone)]
pub struct PropertySchema {
    pub name: String,
    pub prop_id: i32,
    pub data_type: DataType,
    pub nullable: bool,
    pub default_value: Option<Value>,
}

impl PropertySchema {
    pub fn new(name: String, prop_id: i32, data_type: DataType) -> Self {
        Self {
            name,
            prop_id,
            data_type,
            nullable: false,
            default_value: None,
        }
    }

    pub fn nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }

    pub fn default(mut self, value: Value) -> Self {
        self.default_value = Some(value);
        self
    }
}

#[derive(Debug, Clone)]
pub struct PropertyRow {
    values: Vec<Option<Value>>,
}

impl PropertyRow {
    pub fn new(column_count: usize) -> Self {
        Self {
            values: vec![None; column_count],
        }
    }

    pub fn set(&mut self, col_idx: usize, value: Option<Value>) {
        if col_idx < self.values.len() {
            self.values[col_idx] = value;
        }
    }

    pub fn get(&self, col_idx: usize) -> Option<&Value> {
        self.values.get(col_idx).and_then(|v| v.as_ref())
    }

    pub fn values(&self) -> &[Option<Value>] {
        &self.values
    }
}

pub struct PropertyTable {
    schema: Vec<PropertySchema>,
    name_to_index: HashMap<String, usize>,
    rows: Vec<PropertyRow>,
    free_list: Vec<u32>,
    next_offset: u32,
}

impl PropertyTable {
    pub fn new() -> Self {
        Self {
            schema: Vec::new(),
            name_to_index: HashMap::new(),
            rows: Vec::new(),
            free_list: Vec::new(),
            next_offset: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            schema: Vec::new(),
            name_to_index: HashMap::new(),
            rows: Vec::with_capacity(capacity),
            free_list: Vec::new(),
            next_offset: 0,
        }
    }

    pub fn add_property(&mut self, name: String, data_type: DataType, nullable: bool) -> i32 {
        let prop_id = self.schema.len() as i32;
        let schema = PropertySchema::new(name.clone(), prop_id, data_type).nullable(nullable);
        self.name_to_index.insert(name, self.schema.len());
        self.schema.push(schema);

        for row in &mut self.rows {
            row.values.push(None);
        }

        prop_id
    }

    pub fn insert(&mut self, values: &[(String, Value)]) -> StorageResult<u32> {
        let offset = if let Some(free) = self.free_list.pop() {
            free
        } else {
            let offset = self.next_offset;
            self.next_offset += 1;
            self.rows.push(PropertyRow::new(self.schema.len()));
            offset
        };

        self.update(offset, values)?;
        Ok(offset)
    }

    pub fn update(&mut self, offset: u32, values: &[(String, Value)]) -> StorageResult<()> {
        let offset_idx = offset as usize;
        if offset_idx >= self.rows.len() {
            return Err(StorageError::InvalidOffset(offset));
        }

        let row = &mut self.rows[offset_idx];
        for (name, value) in values {
            if let Some(&col_idx) = self.name_to_index.get(name) {
                row.set(col_idx, Some(value.clone()));
            }
        }

        Ok(())
    }

    pub fn get(&self, offset: u32) -> Option<Vec<(String, Option<Value>)>> {
        let offset_idx = offset as usize;
        if offset_idx >= self.rows.len() {
            return None;
        }

        let row = &self.rows[offset_idx];
        Some(
            self.schema
                .iter()
                .enumerate()
                .map(|(i, s)| (s.name.clone(), row.values.get(i).and_then(|v| v.clone())))
                .collect(),
        )
    }

    pub fn get_property(&self, offset: u32, name: &str) -> Option<Value> {
        let col_idx = *self.name_to_index.get(name)?;
        let offset_idx = offset as usize;

        if offset_idx >= self.rows.len() {
            return None;
        }

        self.rows[offset_idx].get(col_idx).cloned()
    }

    pub fn set_property(&mut self, offset: u32, name: &str, value: Option<Value>) -> StorageResult<()> {
        let col_idx = *self.name_to_index
            .get(name)
            .ok_or_else(|| StorageError::ColumnNotFound(name.to_string()))?;

        let offset_idx = offset as usize;
        if offset_idx >= self.rows.len() {
            return Err(StorageError::InvalidOffset(offset));
        }

        self.rows[offset_idx].set(col_idx, value);
        Ok(())
    }

    pub fn delete(&mut self, offset: u32) -> bool {
        let offset_idx = offset as usize;
        if offset_idx >= self.rows.len() {
            return false;
        }

        self.rows[offset_idx] = PropertyRow::new(self.schema.len());
        self.free_list.push(offset);
        true
    }

    pub fn row_count(&self) -> usize {
        self.rows.len() - self.free_list.len()
    }

    pub fn property_count(&self) -> usize {
        self.schema.len()
    }

    pub fn schema(&self) -> &[PropertySchema] {
        &self.schema
    }

    pub fn property_names(&self) -> Vec<&str> {
        self.schema.iter().map(|s| s.name.as_str()).collect()
    }

    pub fn clear(&mut self) {
        self.rows.clear();
        self.free_list.clear();
        self.next_offset = 0;
    }

    pub fn has_property(&self, name: &str) -> bool {
        self.name_to_index.contains_key(name)
    }

    pub fn get_schema(&self, name: &str) -> Option<&PropertySchema> {
        self.name_to_index.get(name).and_then(|&idx| self.schema.get(idx))
    }
}

impl Default for PropertyTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut table = PropertyTable::new();

        table.add_property("weight".to_string(), DataType::Double, false);
        table.add_property("since".to_string(), DataType::Int, true);

        let offset = table.insert(&[
            ("weight".to_string(), Value::Double(1.5)),
            ("since".to_string(), Value::Int(2020)),
        ]).unwrap();

        let props = table.get(offset).unwrap();
        assert_eq!(props.len(), 2);

        assert_eq!(table.get_property(offset, "weight"), Some(Value::Double(1.5)));
        assert_eq!(table.get_property(offset, "since"), Some(Value::Int(2020)));
    }

    #[test]
    fn test_update() {
        let mut table = PropertyTable::new();
        table.add_property("weight".to_string(), DataType::Double, false);

        let offset = table.insert(&[("weight".to_string(), Value::Double(1.0))]).unwrap();

        table.update(offset, &[("weight".to_string(), Value::Double(2.0))]).unwrap();

        assert_eq!(table.get_property(offset, "weight"), Some(Value::Double(2.0)));
    }

    #[test]
    fn test_delete() {
        let mut table = PropertyTable::new();
        table.add_property("weight".to_string(), DataType::Double, false);

        let offset1 = table.insert(&[("weight".to_string(), Value::Double(1.0))]).unwrap();
        let offset2 = table.insert(&[("weight".to_string(), Value::Double(2.0))]).unwrap();

        assert!(table.delete(offset1));
        assert_eq!(table.row_count(), 1);

        let offset3 = table.insert(&[("weight".to_string(), Value::Double(3.0))]).unwrap();
        assert_eq!(offset3, offset1);
    }
}
