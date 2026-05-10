use std::collections::HashMap;

use crate::core::{StorageError, StorageResult, Value};
use crate::storage::engine::CacheManager;
use crate::storage::vertex::{
    LabelId, PropertyDef as VertexPropertyDef, Timestamp,
    VertexRecord, VertexSchema, VertexTable,
};

pub struct SchemaOps {
    pub vertex_tables: HashMap<LabelId, VertexTable>,
    pub vertex_label_names: HashMap<String, LabelId>,
    pub vertex_label_counter: LabelId,
}

impl Default for SchemaOps {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaOps {
    pub fn new() -> Self {
        Self {
            vertex_tables: HashMap::new(),
            vertex_label_names: HashMap::new(),
            vertex_label_counter: 0,
        }
    }

    pub fn create_vertex_type(
        &mut self,
        name: &str,
        properties: Vec<VertexPropertyDef>,
        primary_key: &str,
    ) -> StorageResult<LabelId> {
        if self.vertex_label_names.contains_key(name) {
            return Err(StorageError::label_already_exists(name.to_string()));
        }

        let label_id = self.vertex_label_counter;
        self.vertex_label_counter += 1;

        let primary_key_index = properties
            .iter()
            .position(|p| p.name == primary_key)
            .ok_or_else(|| StorageError::property_not_found(primary_key.to_string()))?;

        let schema = VertexSchema {
            label_id,
            label_name: name.to_string(),
            properties,
            primary_key_index,
        };

        let table = VertexTable::new(label_id, name.to_string(), schema);
        self.vertex_tables.insert(label_id, table);
        self.vertex_label_names.insert(name.to_string(), label_id);

        Ok(label_id)
    }

    pub fn drop_vertex_type(&mut self, name: &str) -> StorageResult<()> {
        let label_id = self
            .vertex_label_names
            .remove(name)
            .ok_or_else(|| StorageError::label_not_found(name.to_string()))?;

        self.vertex_tables.remove(&label_id);

        Ok(())
    }

    pub fn get_vertex_label_id(&self, name: &str) -> Option<LabelId> {
        self.vertex_label_names.get(name).copied()
    }

    pub fn vertex_label_names(&self) -> Vec<&str> {
        self.vertex_label_names.keys().map(|s| s.as_str()).collect()
    }

    pub fn get_vertex_table(&self, label: LabelId) -> Option<&VertexTable> {
        self.vertex_tables.get(&label)
    }

    pub fn vertex_tables(&self) -> &HashMap<LabelId, VertexTable> {
        &self.vertex_tables
    }

    pub fn vertex_tables_iter(&self) -> impl Iterator<Item = (&LabelId, &VertexTable)> {
        self.vertex_tables.iter()
    }

    pub fn insert_vertex(
        &mut self,
        label: LabelId,
        external_id: &str,
        properties: &[(String, Value)],
        ts: Timestamp,
    ) -> StorageResult<u32> {
        let table = self
            .vertex_tables
            .get_mut(&label)
            .ok_or_else(|| StorageError::label_not_found(format!("vertex label {}", label)))?;

        table.insert(external_id, properties, ts)
    }

    pub fn get_vertex(
        &self,
        label: LabelId,
        external_id: &str,
        ts: Timestamp,
        _cache_manager: &CacheManager,
    ) -> Option<VertexRecord> {
        self.vertex_tables.get(&label)?.get(external_id, ts)
    }

    pub fn scan_vertices(
        &self,
        label: LabelId,
        ts: Timestamp,
        _cache_manager: &CacheManager,
    ) -> Option<impl Iterator<Item = VertexRecord> + use<'_>> {
        let table = self.vertex_tables.get(&label)?;
        Some(table.scan(ts))
    }

    pub fn vertex_count(&self, label: LabelId) -> usize {
        self.vertex_tables.get(&label).map(|t| t.vertex_count(Timestamp::MAX)).unwrap_or(0)
    }

    pub fn get_vertex_internal_id(
        &self,
        label: LabelId,
        external_id: &str,
        ts: Timestamp,
    ) -> Option<u32> {
        self.vertex_tables
            .get(&label)?
            .get_internal_id(external_id, ts)
    }

    pub fn get_vertex_by_internal_id(
        &self,
        label: LabelId,
        internal_id: u32,
        ts: Timestamp,
    ) -> Option<VertexRecord> {
        self.vertex_tables
            .get(&label)?
            .get_by_internal_id(internal_id, ts)
    }

    pub fn delete_vertex(
        &mut self,
        label: LabelId,
        external_id: &str,
        ts: Timestamp,
    ) -> StorageResult<()> {
        let table = self
            .vertex_tables
            .get_mut(&label)
            .ok_or_else(|| StorageError::label_not_found(format!("vertex label {}", label)))?;

        table.delete(external_id, ts)
    }

    pub fn update_vertex_property(
        &mut self,
        label: LabelId,
        external_id: &str,
        property_name: &str,
        value: &Value,
        ts: Timestamp,
    ) -> StorageResult<()> {
        let table = self
            .vertex_tables
            .get_mut(&label)
            .ok_or_else(|| StorageError::label_not_found(format!("vertex label {}", label)))?;

        let internal_id = table
            .get_internal_id(external_id, ts)
            .ok_or(StorageError::vertex_not_found())?;

        table.update_property(internal_id, property_name, value, ts)
    }

    pub fn revert_delete_vertex_properties(
        &mut self,
        label_name: &str,
        prop_names: &[String],
    ) -> StorageResult<()> {
        let label_id = self
            .vertex_label_names
            .get(label_name)
            .copied()
            .ok_or_else(|| StorageError::label_not_found(label_name.to_string()))?;

        let table = self
            .vertex_tables
            .get_mut(&label_id)
            .ok_or_else(|| StorageError::label_not_found(label_name.to_string()))?;

        for prop_name in prop_names {
            if table.schema().properties.iter().any(|p| p.name == *prop_name) {
                continue;
            }

            let prop_def = VertexPropertyDef::new(
                prop_name.clone(),
                crate::core::DataType::String,
            );
            table.add_property(prop_def)?;
        }

        Ok(())
    }
}
