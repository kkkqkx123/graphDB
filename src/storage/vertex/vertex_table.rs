//! Vertex Table
//!
//! Main vertex storage with columnar layout.
//! Combines ID indexing, column storage, and timestamp tracking.

use std::path::Path;

use super::{
    ColumnStore, IdIndexer, LabelId, PropertyDef, Timestamp, VertexId, VertexRecord, VertexSchema,
    VertexTimestamp,
};
use crate::core::{StorageError, StorageResult, Value};

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

#[derive(Debug)]
pub struct VertexTable {
    label: LabelId,
    label_name: String,
    schema: VertexSchema,
    id_indexer: IdIndexer<String>,
    columns: ColumnStore,
    timestamps: VertexTimestamp,
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
            is_open: true,
        }
    }

    pub fn open<P: AsRef<Path>>(
        &mut self,
        _path: P,
        _memory_level: MemoryLevel,
    ) -> StorageResult<()> {
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
            return Err(StorageError::storage_not_open());
        }

        if self.id_indexer.contains(&external_id.to_string()) {
            let internal_id = self
                .id_indexer
                .get_index(&external_id.to_string())
                .ok_or(StorageError::vertex_not_found())?;

            if self.timestamps.is_valid(internal_id, ts) {
                return Err(StorageError::vertex_already_exists(external_id.to_string()));
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

        let _external_id = self.id_indexer.get_key(internal_id)?;
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
            return Err(StorageError::storage_not_open());
        }

        if !self.timestamps.is_valid(internal_id, ts) {
            return Err(StorageError::vertex_not_found());
        }

        self.columns
            .set_property(internal_id as usize, col_name, Some(value))
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
            .get_column_by_id_mut(col_id)
            .ok_or_else(|| StorageError::column_not_found(format!("col_id={}", col_id)))?;
        col.set(internal_id as usize, Some(value))
    }

    pub fn delete(&mut self, external_id: &str, ts: Timestamp) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        let internal_id = self
            .id_indexer
            .get_index(&external_id.to_string())
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

    pub fn batch_insert(
        &mut self,
        vertices: &[(String, Vec<(String, Value)>)],
        ts: Timestamp,
    ) -> StorageResult<Vec<u32>> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        let count = vertices.len();
        self.ensure_capacity(self.total_count() + count);

        let mut internal_ids = Vec::with_capacity(count);

        for (external_id, properties) in vertices {
            let internal_id = self.insert(external_id, properties, ts)?;
            internal_ids.push(internal_id);
        }

        Ok(internal_ids)
    }

    pub fn batch_delete(
        &mut self,
        external_ids: &[String],
        ts: Timestamp,
    ) -> StorageResult<usize> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        let mut deleted_count = 0;

        for external_id in external_ids {
            if let Some(internal_id) = self.id_indexer.get_index(external_id) {
                self.timestamps.remove(internal_id, ts);
                deleted_count += 1;
            }
        }

        Ok(deleted_count)
    }

    pub fn batch_get(
        &self,
        external_ids: &[String],
        ts: Timestamp,
    ) -> Vec<Option<VertexRecord>> {
        if !self.is_open {
            return vec![None; external_ids.len()];
        }

        external_ids
            .iter()
            .map(|id| self.get(id, ts))
            .collect()
    }

    pub fn batch_update(
        &mut self,
        updates: &[(String, Vec<(String, Value)>)],
        ts: Timestamp,
    ) -> StorageResult<usize> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        let mut updated_count = 0;

        for (external_id, properties) in updates {
            if let Some(internal_id) = self.id_indexer.get_index(external_id) {
                if self.timestamps.is_valid(internal_id, ts) {
                    for (col_name, value) in properties {
                        let _ = self.columns.set_property(
                            internal_id as usize,
                            col_name,
                            Some(value),
                        );
                    }
                    updated_count += 1;
                }
            }
        }

        Ok(updated_count)
    }

    pub fn revert_delete(&mut self, internal_id: u32, ts: Timestamp) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
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
        self.id_indexer.len()
    }

    pub fn scan(&self, ts: Timestamp) -> VertexIterator<'_> {
        VertexIterator::new(self, ts)
    }

    pub fn add_property(&mut self, prop: PropertyDef) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        if self.columns.get_column(&prop.name).is_some() {
            return Err(StorageError::column_already_exists(prop.name.clone()));
        }

        self.schema.properties.push(prop.clone());
        self.columns
            .add_column(prop.name, prop.data_type, prop.nullable);

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
        let id_mapping = self.id_indexer.compact().unwrap_or_default();

        if id_mapping.is_empty() {
            self.timestamps.compact();
            return;
        }

        self.remap_columns(&id_mapping);
        self.remap_timestamps(&id_mapping);
    }

    fn remap_columns(&mut self, id_mapping: &std::collections::HashMap<u32, u32>) {
        if id_mapping.is_empty() {
            return;
        }

        let max_old_id = id_mapping.keys().max().copied().unwrap_or(0) as usize;
        if max_old_id >= self.columns.row_count() {
            return;
        }

        let mut new_columns = ColumnStore::with_capacity(self.id_indexer.len());
        for prop in &self.schema.properties {
            new_columns.add_column(prop.name.clone(), prop.data_type.clone(), prop.nullable);
        }

        for (old_id, new_id) in id_mapping {
            let old_idx = *old_id as usize;
            let new_idx = *new_id as usize;

            let values = self.columns.get(old_idx);
            let pairs: Vec<(String, Value)> = values
                .into_iter()
                .filter_map(|(name, opt_val)| opt_val.map(|v| (name, v)))
                .collect();

            if !pairs.is_empty() {
                let _ = new_columns.set(new_idx, &pairs);
            }
        }

        self.columns = new_columns;
    }

    fn remap_timestamps(&mut self, id_mapping: &std::collections::HashMap<u32, u32>) {
        if id_mapping.is_empty() {
            return;
        }

        let max_new_id = id_mapping.values().max().copied().unwrap_or(0) as usize;
        let mut new_timestamps = VertexTimestamp::with_capacity(max_new_id + 1);

        for (old_id, new_id) in id_mapping {
            if let Some(start_ts) = self.timestamps.get_start_ts(*old_id) {
                new_timestamps.insert(*new_id, start_ts);
                if let Some(end_ts) = self.timestamps.get_end_ts(*old_id) {
                    if end_ts < super::MAX_TIMESTAMP {
                        new_timestamps.remove(*new_id, end_ts);
                    }
                }
            }
        }

        self.timestamps = new_timestamps;
    }

    pub fn fragmentation_ratio(&self) -> f64 {
        let total_slots = self.id_indexer.total_slots();
        let active_count = self.id_indexer.len();

        if total_slots == 0 {
            return 0.0;
        }

        (total_slots - active_count) as f64 / total_slots as f64
    }

    pub fn deleted_count(&self) -> usize {
        self.timestamps.size() - self.timestamps.valid_count(super::MAX_TIMESTAMP - 1)
    }

    pub fn flush<P: AsRef<Path>>(&self, path: P) -> StorageResult<()> {
        use std::fs::{self, File};
        use std::io::Write;

        let path = path.as_ref();
        fs::create_dir_all(path)?;

        let meta_path = path.join("meta.bin");
        let mut meta_file = File::create(&meta_path)?;

        let label_bytes = self.label.to_le_bytes();
        let label_name_bytes = self.label_name.as_bytes();
        let label_name_len = label_name_bytes.len() as u32;

        meta_file.write_all(&label_bytes)?;
        meta_file.write_all(&label_name_len.to_le_bytes())?;
        meta_file.write_all(label_name_bytes)?;

        let schema_json = serde_json::to_string(&self.schema)
            .map_err(|e| StorageError::serialize_error(e.to_string()))?;
        let schema_bytes = schema_json.as_bytes();
        meta_file.write_all(&(schema_bytes.len() as u32).to_le_bytes())?;
        meta_file.write_all(schema_bytes)?;

        let id_indexer_path = path.join("id_indexer.bin");
        self.flush_id_indexer(&id_indexer_path)?;

        let columns_path = path.join("columns.bin");
        self.flush_columns(&columns_path)?;

        let timestamps_path = path.join("timestamps.bin");
        self.flush_timestamps(&timestamps_path)?;

        Ok(())
    }

    fn flush_id_indexer(&self, path: &Path) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(path)?;

        let keys: Vec<&String> = self.id_indexer.keys().collect();
        let count = keys.len() as u32;
        file.write_all(&count.to_le_bytes())?;

        for key in keys {
            let key_bytes = key.as_bytes();
            file.write_all(&(key_bytes.len() as u32).to_le_bytes())?;
            file.write_all(key_bytes)?;
        }

        Ok(())
    }

    fn flush_columns(&self, path: &Path) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(path)?;

        let column_count = self.columns.column_count() as u32;
        file.write_all(&column_count.to_le_bytes())?;

        for col in self.columns.columns() {
            let name_bytes = col.name.as_bytes();
            file.write_all(&(name_bytes.len() as u32).to_le_bytes())?;
            file.write_all(name_bytes)?;

            let data = col.data();
            file.write_all(&(data.len() as u32).to_le_bytes())?;
            file.write_all(data)?;

            if let Some(bitmap) = col.null_bitmap() {
                file.write_all(&[1u8])?;
                let bitmap_bytes = bitmap.as_raw_slice();
                let bitmap_bit_len = bitmap.len() as u32;
                file.write_all(&bitmap_bit_len.to_le_bytes())?;
                file.write_all(&(bitmap_bytes.len() as u32).to_le_bytes())?;
                file.write_all(bitmap_bytes)?;
            } else {
                file.write_all(&[0u8])?;
            }
        }

        Ok(())
    }

    fn flush_timestamps(&self, path: &Path) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(path)?;

        let timestamps = self.timestamps.dump();
        let count = timestamps.len() as u32;
        file.write_all(&count.to_le_bytes())?;

        for ts in timestamps {
            file.write_all(&ts.to_le_bytes())?;
        }

        Ok(())
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Read;

        let path = path.as_ref();

        let meta_path = path.join("meta.bin");
        let mut meta_file = File::open(&meta_path)?;

        let mut label_bytes = [0u8; 4];
        meta_file.read_exact(&mut label_bytes)?;
        self.label = u32::from_le_bytes(label_bytes);

        let mut label_name_len_bytes = [0u8; 4];
        meta_file.read_exact(&mut label_name_len_bytes)?;
        let label_name_len = u32::from_le_bytes(label_name_len_bytes) as usize;

        let mut label_name_bytes = vec![0u8; label_name_len];
        meta_file.read_exact(&mut label_name_bytes)?;
        self.label_name = String::from_utf8(label_name_bytes)
            .map_err(|e| StorageError::deserialize_error(e.to_string()))?;

        let mut schema_len_bytes = [0u8; 4];
        meta_file.read_exact(&mut schema_len_bytes)?;
        let schema_len = u32::from_le_bytes(schema_len_bytes) as usize;

        let mut schema_bytes = vec![0u8; schema_len];
        meta_file.read_exact(&mut schema_bytes)?;
        let schema_json = String::from_utf8(schema_bytes)
            .map_err(|e| StorageError::deserialize_error(e.to_string()))?;
        self.schema = serde_json::from_str(&schema_json)
            .map_err(|e| StorageError::deserialize_error(e.to_string()))?;

        let id_indexer_path = path.join("id_indexer.bin");
        self.load_id_indexer(&id_indexer_path)?;

        let columns_path = path.join("columns.bin");
        self.load_columns(&columns_path)?;

        let timestamps_path = path.join("timestamps.bin");
        self.load_timestamps(&timestamps_path)?;

        self.is_open = true;
        Ok(())
    }

    fn load_id_indexer(&mut self, path: &Path) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(path)?;

        let mut count_bytes = [0u8; 4];
        file.read_exact(&mut count_bytes)?;
        let count = u32::from_le_bytes(count_bytes) as usize;

        self.id_indexer.clear();

        for _ in 0..count {
            let mut key_len_bytes = [0u8; 4];
            file.read_exact(&mut key_len_bytes)?;
            let key_len = u32::from_le_bytes(key_len_bytes) as usize;

            let mut key_bytes = vec![0u8; key_len];
            file.read_exact(&mut key_bytes)?;
            let key = String::from_utf8(key_bytes)
                .map_err(|e| StorageError::deserialize_error(e.to_string()))?;

            self.id_indexer.insert(key)?;
        }

        Ok(())
    }

    fn load_columns(&mut self, path: &Path) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(path)?;

        let mut column_count_bytes = [0u8; 4];
        file.read_exact(&mut column_count_bytes)?;
        let column_count = u32::from_le_bytes(column_count_bytes) as usize;

        self.columns.clear();

        for _ in 0..column_count {
            let mut name_len_bytes = [0u8; 4];
            file.read_exact(&mut name_len_bytes)?;
            let name_len = u32::from_le_bytes(name_len_bytes) as usize;

            let mut name_bytes = vec![0u8; name_len];
            file.read_exact(&mut name_bytes)?;
            let name = String::from_utf8(name_bytes)
                .map_err(|e| StorageError::deserialize_error(e.to_string()))?;

            let mut data_len_bytes = [0u8; 4];
            file.read_exact(&mut data_len_bytes)?;
            let data_len = u32::from_le_bytes(data_len_bytes) as usize;

            let mut data = vec![0u8; data_len];
            file.read_exact(&mut data)?;

            let mut has_bitmap_bytes = [0u8; 1];
            file.read_exact(&mut has_bitmap_bytes)?;
            let has_bitmap = has_bitmap_bytes[0] == 1;

            let (null_bitmap_raw, bitmap_bit_len) = if has_bitmap {
                let mut bitmap_bit_len_bytes = [0u8; 4];
                file.read_exact(&mut bitmap_bit_len_bytes)?;
                let bitmap_bit_len = u32::from_le_bytes(bitmap_bit_len_bytes) as usize;

                let mut bitmap_bytes_len_bytes = [0u8; 4];
                file.read_exact(&mut bitmap_bytes_len_bytes)?;
                let bitmap_bytes_len = u32::from_le_bytes(bitmap_bytes_len_bytes) as usize;

                let mut bitmap_bytes = vec![0u8; bitmap_bytes_len];
                file.read_exact(&mut bitmap_bytes)?;

                (Some(bitmap_bytes), bitmap_bit_len)
            } else {
                (None, 0)
            };

            self.columns.load_column_from_raw(&name, data, null_bitmap_raw, bitmap_bit_len)?;
        }

        Ok(())
    }

    fn load_timestamps(&mut self, path: &Path) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(path)?;

        let mut count_bytes = [0u8; 4];
        file.read_exact(&mut count_bytes)?;
        let count = u32::from_le_bytes(count_bytes) as usize;

        let mut timestamps = Vec::with_capacity(count);
        for _ in 0..count {
            let mut ts_bytes = [0u8; 4];
            file.read_exact(&mut ts_bytes)?;
            timestamps.push(u32::from_le_bytes(ts_bytes));
        }

        self.timestamps.load(&timestamps);

        Ok(())
    }

    pub fn compact_with_ts(&mut self, ts: Timestamp) -> usize {
        self.compact_with_ts_collect(ts).len()
    }

    pub fn compact_with_ts_collect(&mut self, ts: Timestamp) -> Vec<String> {
        let deleted_ids: Vec<u32> = self.timestamps
            .iter_deleted(ts)
            .collect();

        let mut removed_keys = Vec::with_capacity(deleted_ids.len());

        for id in &deleted_ids {
            if let Some(key) = self.id_indexer.get_key(*id).cloned() {
                self.id_indexer.remove(&key);
                removed_keys.push(key);
            }
        }

        self.compact();

        removed_keys
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

        total += self.timestamps.valid_count(super::MAX_TIMESTAMP - 1) * std::mem::size_of::<Timestamp>();

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
    use crate::storage::vertex::{PropertyDef, VertexSchema};

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

        let record = table.get("v1", 100).unwrap();
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

        assert!(table.get("v1", 150).is_some());
        assert!(table.get("v1", 250).is_none());
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
}
