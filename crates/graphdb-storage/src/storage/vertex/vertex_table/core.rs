//! Vertex Table Core
//!
//! Main vertex storage with columnar layout.
//! Combines ID indexing, column storage, and timestamp tracking.

use std::path::Path;

use super::super::{ColumnStore, IdIndexer, IdKey, LabelId, Timestamp, VertexId, VertexRecord, VertexSchema, VertexTimestamp};
use crate::core::{StorageError, StorageResult, Value};

#[derive(Debug, Clone)]
pub struct VertexTableConfig {
    pub initial_capacity: usize,
}

impl Default for VertexTableConfig {
    fn default() -> Self {
        Self {
            initial_capacity: 4096,
        }
    }
}

#[derive(Debug)]
pub struct VertexTable {
    pub(super) label: LabelId,
    pub(super) label_name: String,
    pub(super) schema: VertexSchema,
    pub(super) id_indexer: IdIndexer,
    pub(super) columns: ColumnStore,
    pub(super) timestamps: VertexTimestamp,
    pub(super) is_open: bool,
    pub(super) deferred_encodings: std::collections::HashMap<String, crate::storage::encoding::EncodingType>,
}

impl VertexTable {
    pub fn new(label: LabelId, label_name: String, schema: VertexSchema) -> Self {
        Self::with_config(label, label_name, schema, VertexTableConfig::default())
    }

    pub fn with_config(
        label: LabelId,
        label_name: String,
        schema: VertexSchema,
        config: VertexTableConfig,
    ) -> Self {
        let mut columns = ColumnStore::with_capacity(schema.properties.len());

        for prop in &schema.properties {
            columns.add_column(prop.name.clone(), prop.data_type.clone(), prop.nullable);
        }

        Self {
            label,
            label_name,
            schema,
            id_indexer: IdIndexer::with_capacity(config.initial_capacity),
            columns,
            timestamps: VertexTimestamp::with_capacity(config.initial_capacity),
            is_open: true,
            deferred_encodings: std::collections::HashMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        external_id: &str,
        properties: &[(String, Value)],
        ts: Timestamp,
    ) -> StorageResult<u32> {
        self.insert_by_key(IdKey::Text(external_id.to_string()), properties, ts)
    }

    pub fn insert_by_i64(
        &mut self,
        external_id: i64,
        properties: &[(String, Value)],
        ts: Timestamp,
    ) -> StorageResult<u32> {
        self.insert_by_key(IdKey::Int(external_id), properties, ts)
    }

    fn insert_by_key(
        &mut self,
        key: IdKey,
        properties: &[(String, Value)],
        ts: Timestamp,
    ) -> StorageResult<u32> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        let mut converted: Vec<(String, Value)> = Vec::with_capacity(properties.len());
        for (name, value) in properties {
            let prop_def = self
                .schema
                .properties
                .iter()
                .find(|p| &p.name == name)
                .ok_or_else(|| StorageError::column_not_found(name.clone()))?;

            if value.data_type() != prop_def.data_type {
                let converted_val = value.try_cast_to(&prop_def.data_type)?;
                converted.push((name.clone(), converted_val));
            } else {
                converted.push((name.clone(), value.clone()));
            }
        }

        if self.id_indexer.contains(&key) {
            let internal_id = self
                .id_indexer
                .get_index(&key)
                .ok_or(StorageError::vertex_not_found())?;

            if self.timestamps.is_valid(internal_id, ts) {
                return Err(StorageError::vertex_already_exists(format!("{:?}", key)));
            }

            let _ = self.timestamps.revert_remove(internal_id, ts);
            self.columns.set(internal_id as usize, &converted)?;
            return Ok(internal_id);
        }

        let internal_id = self.id_indexer.insert(key)?;
        self.timestamps.insert(internal_id, ts);
        self.columns.set(internal_id as usize, &converted)?;

        Ok(internal_id)
    }

    pub fn get_by_internal_id(&self, internal_id: u32, ts: Timestamp) -> Option<VertexRecord> {
        if !self.is_open {
            return None;
        }

        if !self.timestamps.is_valid(internal_id, ts) {
            return None;
        }

        let external_id = self.id_indexer.get_key(internal_id)?;
        let props = self.columns.get(internal_id as usize);
        let properties: Vec<(String, Value)> = props
            .into_iter()
            .filter_map(|(name, opt_val)| opt_val.map(|v| (name, v)))
            .collect();

        let vid = match external_id {
            IdKey::Int(i) => VertexId::from_int64(i),
            IdKey::Text(s) => VertexId::from_string(&s),
        };

        Some(VertexRecord {
            vid,
            internal_id,
            properties,
        })
    }

    pub fn update_property(
        &mut self,
        internal_id: u32,
        col_name: &str,
        value: &Value,
        ts: Timestamp,
    ) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        if !self.timestamps.is_valid(internal_id, ts) {
            return Err(StorageError::vertex_not_found());
        }

        let prop_def = self
            .schema
            .properties
            .iter()
            .find(|p| p.name == col_name)
            .ok_or_else(|| StorageError::column_not_found(col_name.to_string()))?;

        let converted_value = if value.data_type() != prop_def.data_type {
            value.try_cast_to(&prop_def.data_type)?
        } else {
            value.clone()
        };

        self.columns
            .set_property(internal_id as usize, col_name, Some(&converted_value))
    }

    pub fn update_property_by_id(
        &mut self,
        internal_id: u32,
        col_id: i32,
        value: &Value,
        ts: Timestamp,
    ) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        if !self.timestamps.is_valid(internal_id, ts) {
            return Err(StorageError::vertex_not_found());
        }

        let col = self
            .columns
            .get_column_by_id(col_id)
            .ok_or_else(|| StorageError::column_not_found(format!("col_id={}", col_id)))?;

        let converted_value = if value.data_type() != col.data_type {
            value.try_cast_to(&col.data_type)?
        } else {
            value.clone()
        };

        let col = self
            .columns
            .get_column_by_id_mut(col_id)
            .ok_or_else(|| StorageError::column_not_found(format!("col_id={}", col_id)))?;
        col.set(internal_id as usize, Some(&converted_value))
    }

    pub fn delete(&mut self, external_id: &str, ts: Timestamp) -> StorageResult<()> {
        self.delete_by_key(&IdKey::Text(external_id.to_string()), ts)
    }

    pub fn delete_by_i64(&mut self, external_id: i64, ts: Timestamp) -> StorageResult<()> {
        self.delete_by_key(&IdKey::Int(external_id), ts)
    }

    fn delete_by_key(&mut self, key: &IdKey, ts: Timestamp) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        let internal_id = self
            .id_indexer
            .get_index(key)
            .ok_or(StorageError::vertex_not_found())?;

        self.timestamps.remove(internal_id, ts);
        Ok(())
    }

    pub fn delete_by_internal_id(&mut self, internal_id: u32, ts: Timestamp) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        self.timestamps.remove(internal_id, ts);
        Ok(())
    }

    pub fn revert_delete(&mut self, internal_id: u32, ts: Timestamp) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        if !self.timestamps.revert_remove(internal_id, ts) {
            return Err(StorageError::invalid_operation(format!(
                "Cannot revert deletion of vertex {}: invalid timestamp",
                internal_id
            )));
        }
        Ok(())
    }

    pub fn get_internal_id(&self, external_id: &str, ts: Timestamp) -> Option<u32> {
        if !self.is_open {
            return None;
        }

        let internal_id = self
            .id_indexer
            .get_index(&IdKey::Text(external_id.to_string()))?;
        if self.timestamps.is_valid(internal_id, ts) {
            Some(internal_id)
        } else {
            None
        }
    }

    pub fn get_internal_id_by_i64(&self, external_id: i64, ts: Timestamp) -> Option<u32> {
        if !self.is_open {
            return None;
        }

        let internal_id = self.id_indexer.get_index(&IdKey::Int(external_id))?;
        if self.timestamps.is_valid(internal_id, ts) {
            Some(internal_id)
        } else {
            None
        }
    }

    /// Lookup internal ID from external i64 without timestamp check.
    /// Returns Some(internal_id) even for deleted vertices.
    pub fn get_internal_id_by_i64_raw(&self, external_id: i64) -> Option<u32> {
        if !self.is_open {
            return None;
        }
        self.id_indexer.get_index(&IdKey::Int(external_id))
    }

    /// Lookup internal ID from external string without timestamp check.
    /// Returns Some(internal_id) even for deleted vertices.
    pub fn get_internal_id_raw(&self, external_id: &str) -> Option<u32> {
        if !self.is_open {
            return None;
        }
        self.id_indexer
            .get_index(&IdKey::Text(external_id.to_string()))
    }

    pub fn get_external_id(&self, internal_id: u32, ts: Timestamp) -> Option<IdKey> {
        if !self.is_open || !self.timestamps.is_valid(internal_id, ts) {
            return None;
        }
        self.id_indexer.get_key(internal_id)
    }

    /// Lookup external ID from internal ID without timestamp check.
    /// Returns the external ID even for deleted vertices.
    pub fn get_external_id_raw(&self, internal_id: u32) -> Option<IdKey> {
        if !self.is_open {
            return None;
        }
        self.id_indexer.get_key(internal_id)
    }

    pub fn total_count(&self) -> usize {
        self.id_indexer.len()
    }

    pub fn scan(&self, ts: Timestamp) -> VertexIterator<'_> {
        VertexIterator::new(self, ts)
    }

    pub fn label(&self) -> LabelId {
        self.label
    }

    pub fn label_name(&self) -> &str {
        &self.label_name
    }

    pub fn schema(&self) -> &VertexSchema {
        &self.schema
    }

    pub fn set_schema(&mut self, schema: VertexSchema) {
        self.schema = schema;
    }

    pub fn memory_size(&self) -> usize {
        let mut total = 0;

        total += self.id_indexer.memory_size();
        total += self.columns.memory_size();
        total += self.timestamps.memory_size();
        total += std::mem::size_of::<Self>();

        total
    }

    pub fn used_memory_size(&self) -> usize {
        let mut total = 0;

        let active_count = self.id_indexer.len();
        total += active_count * std::mem::size_of::<(String, u32)>();

        total += self.columns.used_memory_size();

        total += self.timestamps.valid_count(super::super::MAX_TIMESTAMP - 1)
            * std::mem::size_of::<Timestamp>();

        total
    }
}

pub struct VertexIterator<'a> {
    table: &'a VertexTable,
    ts: Timestamp,
    current: u32,
    end: u32,
}

impl<'a> VertexIterator<'a> {
    pub fn new(table: &'a VertexTable, ts: Timestamp) -> Self {
        Self {
            table,
            ts,
            current: 0,
            end: table.total_count() as u32,
        }
    }
}

impl<'a> Iterator for VertexIterator<'a> {
    type Item = VertexRecord;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current < self.end {
            let id = self.current;
            self.current += 1;

            if let Some(record) = self.table.get_by_internal_id(id, self.ts) {
                return Some(record);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::DataType;
    use crate::storage::types::StoragePropertyDef;

    fn create_test_schema() -> VertexSchema {
        VertexSchema {
            label_id: 0,
            label_name: "person".to_string(),
            properties: vec![
                StoragePropertyDef::new("name".to_string(), DataType::String),
                StoragePropertyDef {
                    name: "age".to_string(),
                    data_type: DataType::Int,
                    nullable: true,
                    default_value: None,
                },
            ],
            primary_key_index: 0,
        }
    }

    #[test]
    fn test_insert_and_get() {
        let schema = create_test_schema();
        let mut table = VertexTable::new(0, "person".to_string(), schema);

        let internal_id = table
            .insert(
                "v1",
                &[
                    ("name".to_string(), Value::String("Alice".to_string())),
                    ("age".to_string(), Value::Int(30)),
                ],
                100,
            )
            .unwrap();

        assert_eq!(internal_id, 0);

        let lookup_id = table.get_internal_id("v1", 100).unwrap();
        let record = table.get_by_internal_id(lookup_id, 100).unwrap();
        assert_eq!(record.properties.len(), 2);
    }

    #[test]
    fn test_delete() {
        let schema = create_test_schema();
        let mut table = VertexTable::new(0, "person".to_string(), schema);

        table
            .insert(
                "v1",
                &[("name".to_string(), Value::String("Alice".to_string()))],
                100,
            )
            .unwrap();

        table.delete("v1", 200).unwrap();

        let internal_id = table.get_internal_id("v1", 150).unwrap();
        assert!(table.get_by_internal_id(internal_id, 150).is_some());
        assert!(table.get_internal_id("v1", 250).is_none());
    }

    #[test]
    fn test_iterator() {
        let schema = create_test_schema();
        let mut table = VertexTable::new(0, "person".to_string(), schema);

        table
            .insert(
                "v1",
                &[("name".to_string(), Value::String("Alice".to_string()))],
                100,
            )
            .unwrap();
        table
            .insert(
                "v2",
                &[("name".to_string(), Value::String("Bob".to_string()))],
                100,
            )
            .unwrap();
        table
            .insert(
                "v3",
                &[("name".to_string(), Value::String("Charlie".to_string()))],
                100,
            )
            .unwrap();

        let count = table.scan(100).count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_rename_and_remove_property() {
        let schema = create_test_schema();
        let mut table = VertexTable::new(0, "person".to_string(), schema);

        table
            .add_property(StoragePropertyDef::new(
                "city".to_string(),
                DataType::String,
            ))
            .expect("add property should succeed");

        let internal_id = table
            .insert(
                "v1",
                &[
                    ("name".to_string(), Value::String("Alice".to_string())),
                    ("age".to_string(), Value::Int(30)),
                    ("city".to_string(), Value::String("Shanghai".to_string())),
                ],
                100,
            )
            .unwrap();

        table
            .rename_property("age", "years")
            .expect("rename should succeed");
        table
            .remove_property("city")
            .expect("remove should succeed");

        let record = table
            .get_by_internal_id(internal_id, 100)
            .expect("record should remain visible");

        assert_eq!(
            record
                .properties
                .iter()
                .find(|(name, _)| name == "years")
                .map(|(_, value)| value),
            Some(&Value::Int(30))
        );
        assert!(record.properties.iter().all(|(name, _)| name != "age"));
        assert!(record.properties.iter().all(|(name, _)| name != "city"));
        assert_eq!(
            table
                .schema()
                .properties
                .iter()
                .map(|prop| prop.name.as_str())
                .collect::<Vec<_>>(),
            vec!["name", "years"]
        );
    }
}
