//! Vertex Table
//!
//! Main vertex storage with columnar layout.
//! Combines ID indexing, column storage, and timestamp tracking.

use std::path::Path;
use std::sync::RwLock;

use super::{ColumnStore, IdIndexer, LabelId, PropertyDef, Timestamp, VertexId, VertexRecord, VertexSchema, VertexTimestamp, INVALID_TIMESTAMP};
use crate::core::{DataType, StorageError, StorageResult, Value};

#[derive(Debug, Clone)]
pub struct VertexTableConfig {
    pub initial_capacity: usize,
    pub memory_level: MemoryLevel,
}

impl Default for VertexTableConfig {
    fn default() -> Self {
        Self {
            initial_capacity: 4096,
            memory_level: MemoryLevel::InMemory,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryLevel {
    InMemory,
    SyncToFile,
    HugePagePreferred,
}

pub struct VertexTable {
    label: LabelId,
    label_name: String,
    schema: VertexSchema,
    id_indexer: IdIndexer<String>,
    columns: ColumnStore,
    timestamps: VertexTimestamp,
    config: VertexTableConfig,
    is_open: bool,
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
            config,
            is_open: true,
        }
    }

    pub fn open<P: AsRef<Path>>(&mut self, _path: P, _memory_level: MemoryLevel) -> StorageResult<()> {
        self.is_open = true;
        Ok(())
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn ensure_capacity(&mut self, capacity: usize) {
        self.id_indexer.reserve(capacity);
        self.timestamps.reserve(capacity);
        self.columns.resize(capacity);
    }

    pub fn insert(
        &mut self,
        external_id: &str,
        properties: &[(String, Value)],
        ts: Timestamp,
    ) -> StorageResult<u32> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }

        if self.id_indexer.contains(&external_id.to_string()) {
            let internal_id = self.id_indexer.get_index(&external_id.to_string())
                .ok_or(StorageError::VertexNotFound)?;

            if self.timestamps.is_valid(internal_id, ts) {
                return Err(StorageError::VertexAlreadyExists(external_id.to_string()));
            }

            self.timestamps.revert_remove(internal_id, ts);
            self.columns.set(internal_id as usize, properties)?;
            return Ok(internal_id);
        }

        let internal_id = self.id_indexer.insert(external_id.to_string())?;
        self.timestamps.insert(internal_id, ts);
        self.columns.set(internal_id as usize, properties)?;

        Ok(internal_id)
    }

    pub fn get(&self, external_id: &str, ts: Timestamp) -> Option<VertexRecord> {
        if !self.is_open {
            return None;
        }

        let internal_id = self.id_indexer.get_index(&external_id.to_string())?;

        if !self.timestamps.is_valid(internal_id, ts) {
            return None;
        }

        let props = self.columns.get(internal_id as usize);
        let properties: Vec<(String, Value)> = props
            .into_iter()
            .filter_map(|(name, opt_val)| opt_val.map(|v| (name, v)))
            .collect();

        Some(VertexRecord {
            vid: internal_id as VertexId,
            internal_id,
            properties,
        })
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

        Some(VertexRecord {
            vid: internal_id as VertexId,
            internal_id,
            properties,
        })
    }

    pub fn get_property(&self, internal_id: u32, col_name: &str, ts: Timestamp) -> Option<Value> {
        if !self.is_open || !self.timestamps.is_valid(internal_id, ts) {
            return None;
        }
        self.columns.get_property(internal_id as usize, col_name)
    }

    pub fn update_property(
        &mut self,
        internal_id: u32,
        col_name: &str,
        value: &Value,
        ts: Timestamp,
    ) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }

        if !self.timestamps.is_valid(internal_id, ts) {
            return Err(StorageError::VertexNotFound);
        }

        self.columns.set_property(internal_id as usize, col_name, Some(value))
    }

    pub fn delete(&mut self, external_id: &str, ts: Timestamp) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }

        let internal_id = self.id_indexer
            .get_index(&external_id.to_string())
            .ok_or(StorageError::VertexNotFound)?;

        self.timestamps.remove(internal_id, ts);
        Ok(())
    }

    pub fn delete_by_internal_id(&mut self, internal_id: u32, ts: Timestamp) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }

        self.timestamps.remove(internal_id, ts);
        Ok(())
    }

    pub fn revert_delete(&mut self, internal_id: u32, ts: Timestamp) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }

        self.timestamps.revert_remove(internal_id, ts);
        Ok(())
    }

    pub fn contains(&self, external_id: &str, ts: Timestamp) -> bool {
        if !self.is_open {
            return false;
        }

        self.id_indexer
            .get_index(&external_id.to_string())
            .map(|id| self.timestamps.is_valid(id, ts))
            .unwrap_or(false)
    }

    pub fn get_internal_id(&self, external_id: &str, ts: Timestamp) -> Option<u32> {
        if !self.is_open {
            return None;
        }

        let internal_id = self.id_indexer.get_index(&external_id.to_string())?;
        if self.timestamps.is_valid(internal_id, ts) {
            Some(internal_id)
        } else {
            None
        }
    }

    pub fn get_external_id(&self, internal_id: u32) -> Option<String> {
        self.id_indexer.get_key(internal_id).cloned()
    }

    pub fn vertex_count(&self, ts: Timestamp) -> usize {
        self.timestamps.valid_count(ts)
    }

    pub fn total_count(&self) -> usize {
        self.id_indexer.size()
    }

    pub fn scan(&self, ts: Timestamp) -> VertexIterator {
        VertexIterator::new(self, ts)
    }

    pub fn add_property(&mut self, prop: PropertyDef) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }

        if self.columns.get_column(&prop.name).is_some() {
            return Err(StorageError::ColumnAlreadyExists(prop.name.clone()));
        }

        self.schema.properties.push(prop.clone());
        self.columns.add_column(prop.name, prop.data_type, prop.nullable);

        Ok(())
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

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn capacity(&self) -> usize {
        self.id_indexer.capacity()
    }

    pub fn compact(&mut self) {
        self.timestamps.compact();
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

    fn create_test_schema() -> VertexSchema {
        VertexSchema {
            label_id: 0,
            label_name: "person".to_string(),
            properties: vec![
                PropertyDef::new("name".to_string(), DataType::String),
                PropertyDef::new("age".to_string(), DataType::Int).nullable(true),
            ],
            primary_key_index: 0,
        }
    }

    #[test]
    fn test_insert_and_get() {
        let schema = create_test_schema();
        let mut table = VertexTable::new(0, "person".to_string(), schema);

        let internal_id = table.insert(
            "v1",
            &[
                ("name".to_string(), Value::String("Alice".to_string())),
                ("age".to_string(), Value::Int(30)),
            ],
            100,
        ).unwrap();

        assert_eq!(internal_id, 0);

        let record = table.get("v1", 100).unwrap();
        assert_eq!(record.properties.len(), 2);
    }

    #[test]
    fn test_delete() {
        let schema = create_test_schema();
        let mut table = VertexTable::new(0, "person".to_string(), schema);

        table.insert(
            "v1",
            &[("name".to_string(), Value::String("Alice".to_string()))],
            100,
        ).unwrap();

        table.delete("v1", 200).unwrap();

        assert!(table.get("v1", 150).is_some());
        assert!(table.get("v1", 250).is_none());
    }

    #[test]
    fn test_iterator() {
        let schema = create_test_schema();
        let mut table = VertexTable::new(0, "person".to_string(), schema);

        table.insert("v1", &[("name".to_string(), Value::String("Alice".to_string()))], 100).unwrap();
        table.insert("v2", &[("name".to_string(), Value::String("Bob".to_string()))], 100).unwrap();
        table.insert("v3", &[("name".to_string(), Value::String("Charlie".to_string()))], 100).unwrap();

        let count = table.scan(100).count();
        assert_eq!(count, 3);
    }
}
