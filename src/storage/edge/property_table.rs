//! Property Table for Edges
//!
//! Stores edge properties in a row-oriented format.

use std::collections::HashMap;

use crate::core::{DataType, StorageError, StorageResult, Value};

/// Sentinel value meaning "no properties"
pub const PROP_OFFSET_NONE: u32 = 0;

/// Convert a property offset to a row index
/// Offset 0 is the sentinel for "no properties", so row index = offset - 1
pub fn prop_offset_to_index(offset: u32) -> Option<usize> {
    if offset == PROP_OFFSET_NONE {
        return None;
    }
    Some((offset - 1) as usize)
}

/// Convert a row index to a property offset
/// Row index 0 corresponds to offset 1 (since offset 0 is the sentinel)
pub fn prop_index_to_offset(index: usize) -> u32 {
    (index + 1) as u32
}

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

#[derive(Debug)]
pub struct PropertyTable {
    schema: Vec<PropertySchema>,
    name_to_index: HashMap<String, usize>,
    rows: Vec<PropertyRow>,
    free_list: Vec<u32>,
}

impl PropertyTable {
    pub fn new() -> Self {
        Self {
            schema: Vec::new(),
            name_to_index: HashMap::new(),
            rows: Vec::new(),
            free_list: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            schema: Vec::new(),
            name_to_index: HashMap::new(),
            rows: Vec::with_capacity(capacity),
            free_list: Vec::new(),
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
            self.rows.push(PropertyRow::new(self.schema.len()));
            prop_index_to_offset(self.rows.len() - 1)
        };

        self.update(offset, values)?;
        Ok(offset)
    }

    pub fn update(&mut self, offset: u32, values: &[(String, Value)]) -> StorageResult<()> {
        let row_idx = prop_offset_to_index(offset).ok_or_else(|| StorageError::invalid_offset(offset))?;
        if row_idx >= self.rows.len() {
            return Err(StorageError::invalid_offset(offset));
        }

        let row = &mut self.rows[row_idx];
        for (name, value) in values {
            if let Some(&col_idx) = self.name_to_index.get(name) {
                row.set(col_idx, Some(value.clone()));
            }
        }

        Ok(())
    }

    pub fn get(&self, offset: u32) -> Option<Vec<(String, Option<Value>)>> {
        let row_idx = prop_offset_to_index(offset)?;
        if row_idx >= self.rows.len() {
            return None;
        }

        let row = &self.rows[row_idx];
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
        let row_idx = prop_offset_to_index(offset)?;
        if row_idx >= self.rows.len() {
            return None;
        }

        self.rows[row_idx].get(col_idx).cloned()
    }

    pub fn set_property(
        &mut self,
        offset: u32,
        name: &str,
        value: Option<Value>,
    ) -> StorageResult<()> {
        let col_idx = *self
            .name_to_index
            .get(name)
            .ok_or_else(|| StorageError::column_not_found(name.to_string()))?;

        let row_idx = prop_offset_to_index(offset).ok_or_else(|| StorageError::invalid_offset(offset))?;
        if row_idx >= self.rows.len() {
            return Err(StorageError::invalid_offset(offset));
        }

        self.rows[row_idx].set(col_idx, value);
        Ok(())
    }

    pub fn set_property_by_id(
        &mut self,
        offset: u32,
        prop_id: i32,
        value: Option<Value>,
    ) -> StorageResult<()> {
        let col_idx = prop_id as usize;
        let row_idx = prop_offset_to_index(offset).ok_or_else(|| StorageError::invalid_offset(offset))?;
        if row_idx >= self.rows.len() {
            return Err(StorageError::invalid_offset(offset));
        }

        if col_idx >= self.schema.len() {
            return Err(StorageError::column_not_found(format!("prop_id={}", prop_id)));
        }

        self.rows[row_idx].set(col_idx, value);
        Ok(())
    }

    pub fn get_property_by_id(&self, offset: u32, prop_id: i32) -> Option<Value> {
        let col_idx = prop_id as usize;
        let row_idx = prop_offset_to_index(offset)?;
        if row_idx >= self.rows.len() {
            return None;
        }

        if col_idx >= self.schema.len() {
            return None;
        }

        self.rows[row_idx].get(col_idx).cloned()
    }

    pub fn get_property_type(&self, prop_id: i32) -> Option<DataType> {
        self.schema.get(prop_id as usize).map(|s| s.data_type.clone())
    }

    pub fn delete(&mut self, offset: u32) -> bool {
        let row_idx = match prop_offset_to_index(offset) {
            Some(idx) => idx,
            None => return false,
        };
        if row_idx >= self.rows.len() {
            return false;
        }

        self.rows[row_idx] = PropertyRow::new(self.schema.len());
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
    }

    pub fn has_property(&self, name: &str) -> bool {
        self.name_to_index.contains_key(name)
    }

    pub fn get_schema(&self, name: &str) -> Option<&PropertySchema> {
        self.name_to_index
            .get(name)
            .and_then(|&idx| self.schema.get(idx))
    }

    pub fn dump(&self) -> Vec<u8> {
        let mut result = Vec::new();

        result.extend_from_slice(&(self.schema.len() as u32).to_le_bytes());
        for prop in &self.schema {
            let name_bytes = prop.name.as_bytes();
            result.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
            result.extend_from_slice(name_bytes);
            result.extend_from_slice(&prop.prop_id.to_le_bytes());
            result.push(prop.data_type.as_u8());
            result.push(if prop.nullable { 1 } else { 0 });
        }

        result.extend_from_slice(&(self.rows.len() as u32).to_le_bytes());
        for row in &self.rows {
            result.extend_from_slice(&(row.values.len() as u32).to_le_bytes());
            for value in &row.values {
                if let Some(v) = value {
                    result.push(1);
                    result.extend_from_slice(&v.to_bytes());
                } else {
                    result.push(0);
                }
            }
        }

        result.extend_from_slice(&(self.free_list.len() as u32).to_le_bytes());
        for offset in &self.free_list {
            result.extend_from_slice(&offset.to_le_bytes());
        }

        result
    }

    pub fn load(&mut self, data: &[u8]) {
        if data.is_empty() {
            return;
        }

        let mut offset = 0;

        if offset + 4 > data.len() {
            return;
        }
        let schema_len =
            u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap_or([0; 4])) as usize;
        offset += 4;

        self.schema.clear();
        self.name_to_index.clear();

        for _ in 0..schema_len {
            if offset + 4 > data.len() {
                break;
            }
            let name_len =
                u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap_or([0; 4])) as usize;
            offset += 4;

            if offset + name_len > data.len() {
                break;
            }
            let name = String::from_utf8_lossy(&data[offset..offset + name_len]).to_string();
            offset += name_len;

            if offset + 5 > data.len() {
                break;
            }
            let prop_id = i32::from_le_bytes(data[offset..offset + 4].try_into().unwrap_or([0; 4]));
            offset += 4;
            let data_type = DataType::from_u8(data[offset]);
            offset += 1;
            let nullable = data[offset] == 1;
            offset += 1;

            let prop_schema =
                PropertySchema::new(name.clone(), prop_id, data_type).nullable(nullable);
            self.name_to_index.insert(name, self.schema.len());
            self.schema.push(prop_schema);
        }

        if offset + 4 > data.len() {
            return;
        }
        let rows_len =
            u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap_or([0; 4])) as usize;
        offset += 4;

        self.rows.clear();

        for _ in 0..rows_len {
            if offset + 4 > data.len() {
                break;
            }
            let values_len =
                u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap_or([0; 4])) as usize;
            offset += 4;

            let mut row = PropertyRow::new(values_len);
            for i in 0..values_len {
                if offset >= data.len() {
                    break;
                }
                let has_value = data[offset] == 1;
                offset += 1;

                if has_value {
                    if let Some((value, bytes_read)) = Value::from_bytes(&data[offset..]) {
                        row.values[i] = Some(value);
                        offset += bytes_read;
                    }
                }
            }
            self.rows.push(row);
        }

        if offset + 4 > data.len() {
            return;
        }
        let free_list_len =
            u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap_or([0; 4])) as usize;
        offset += 4;

        self.free_list.clear();
        for _ in 0..free_list_len {
            if offset + 4 > data.len() {
                break;
            }
            let free_offset =
                u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap_or([0; 4]));
            offset += 4;
            self.free_list.push(free_offset);
        }

        if offset + 4 <= data.len() {
            // Skip the old next_offset field for backward compatibility
            let _old_next_offset =
                u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap_or([0; 4]));
        }
    }

    pub fn compact(&mut self, valid_offsets: &std::collections::HashSet<u32>) {
        let mut new_rows = Vec::new();
        let mut offset_mapping = std::collections::HashMap::new();

        for (idx, row) in self.rows.iter().enumerate() {
            let old_offset = prop_index_to_offset(idx);
            if valid_offsets.contains(&old_offset) {
                let new_offset = prop_index_to_offset(new_rows.len());
                offset_mapping.insert(old_offset, new_offset);
                new_rows.push(row.clone());
            }
        }

        self.rows = new_rows;
        self.free_list.clear();
    }

    pub fn memory_size(&self) -> usize {
        let mut total = 0;

        total += self.schema.len() * std::mem::size_of::<PropertySchema>();
        total += self.rows.len() * std::mem::size_of::<PropertyRow>();
        total += self.free_list.len() * std::mem::size_of::<u32>();
        total += std::mem::size_of::<Self>();

        for row in &self.rows {
            total += row.values.len() * std::mem::size_of::<Option<Value>>();
        }

        total
    }

    pub fn used_memory_size(&self) -> usize {
        let active_count = self.rows.len() - self.free_list.len();
        let avg_row_size = self.schema.len() * std::mem::size_of::<Option<Value>>();
        active_count * avg_row_size + std::mem::size_of::<Self>()
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

        let offset = table
            .insert(&[
                ("weight".to_string(), Value::Double(1.5)),
                ("since".to_string(), Value::Int(2020)),
            ])
            .unwrap();

        let props = table.get(offset).unwrap();
        assert_eq!(props.len(), 2);

        assert_eq!(
            table.get_property(offset, "weight"),
            Some(Value::Double(1.5))
        );
        assert_eq!(table.get_property(offset, "since"), Some(Value::Int(2020)));
    }

    #[test]
    fn test_update() {
        let mut table = PropertyTable::new();
        table.add_property("weight".to_string(), DataType::Double, false);

        let offset = table
            .insert(&[("weight".to_string(), Value::Double(1.0))])
            .unwrap();

        table
            .update(offset, &[("weight".to_string(), Value::Double(2.0))])
            .unwrap();

        assert_eq!(
            table.get_property(offset, "weight"),
            Some(Value::Double(2.0))
        );
    }

    #[test]
    fn test_delete() {
        let mut table = PropertyTable::new();
        table.add_property("weight".to_string(), DataType::Double, false);

        let offset1 = table
            .insert(&[("weight".to_string(), Value::Double(1.0))])
            .unwrap();
        let _offset2 = table
            .insert(&[("weight".to_string(), Value::Double(2.0))])
            .unwrap();

        assert!(table.delete(offset1));
        assert_eq!(table.row_count(), 1);

        let offset3 = table
            .insert(&[("weight".to_string(), Value::Double(3.0))])
            .unwrap();
        assert_eq!(offset3, offset1);
    }
}
