//! Vertex Table
//!
//! Main vertex storage with columnar layout.
//! Combines ID indexing, column storage, and timestamp tracking.

use std::path::Path;

use super::{
    ColumnStore, IdIndexer, IdKey, LabelId, Timestamp, VertexId, VertexRecord, VertexSchema,
    VertexTimestamp,
};
use crate::core::{StorageError, StorageResult, Value};
use crate::storage::encoding::EncodingType;
use crate::storage::persistence::{read_header, section, write_header_to, HEADER_SIZE};
use crate::storage::types::StoragePropertyDef;

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
    label: LabelId,
    label_name: String,
    schema: VertexSchema,
    id_indexer: IdIndexer,
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

    pub fn add_property(&mut self, prop: StoragePropertyDef) -> StorageResult<()> {
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

    pub fn remove_property(&mut self, prop_name: &str) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        let index = self
            .schema
            .properties
            .iter()
            .position(|prop| prop.name == prop_name)
            .ok_or_else(|| StorageError::column_not_found(prop_name.to_string()))?;

        if index == self.schema.primary_key_index {
            return Err(StorageError::not_supported(
                "Removing the primary key property is not supported".to_string(),
            ));
        }

        self.schema.properties.remove(index);
        if index < self.schema.primary_key_index {
            self.schema.primary_key_index -= 1;
        }

        self.columns.remove_column(prop_name)?;
        Ok(())
    }

    pub fn rename_property(&mut self, old_name: &str, new_name: &str) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        if self
            .schema
            .properties
            .iter()
            .any(|prop| prop.name == new_name)
        {
            return Err(StorageError::column_already_exists(new_name.to_string()));
        }

        let index = self
            .schema
            .properties
            .iter()
            .position(|prop| prop.name == old_name)
            .ok_or_else(|| StorageError::column_not_found(old_name.to_string()))?;

        self.schema.properties[index].name = new_name.to_string();
        self.columns.rename_column(old_name, new_name.to_string())?;
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

    pub fn set_schema(&mut self, schema: VertexSchema) {
        self.schema = schema;
    }

    pub fn compact(&mut self) {
        let id_mapping = self.id_indexer.compact().unwrap_or_default();
        if id_mapping.is_empty() {
            let old_count = self.timestamps.size();
            self.timestamps.compact();
            let new_count = self.timestamps.size();
            if new_count < old_count && new_count < self.columns.row_count() {
                self.columns.resize(new_count);
            }
            return;
        }
        self.remap_columns(&id_mapping);
        self.remap_timestamps(&id_mapping);
        let _ = self.columns.auto_apply_encodings(None);
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

    pub fn flush<P: AsRef<Path>>(
        &self,
        path: P,
        compression: super::super::compression::CompressionType,
    ) -> StorageResult<()> {
        use std::fs::{self, File};
        use std::io::Write;

        let path = path.as_ref();
        fs::create_dir_all(path)?;

        let meta_path = path.join("meta.bin");
        let mut meta_file = File::create(&meta_path)?;
        write_header_to(&mut meta_file, section::VERTEX_META)
            .map_err(|e| StorageError::io_error(format!("Failed to write meta header: {}", e)))?;

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

        drop(meta_file);
        super::super::compression::compress_file_inplace(&meta_path, compression)?;

        let id_indexer_path = path.join("id_indexer.bin");
        self.flush_id_indexer(&id_indexer_path)?;
        super::super::compression::compress_file_inplace(&id_indexer_path, compression)?;

        let columns_path = path.join("columns.bin");
        self.flush_columns(&columns_path)?;
        super::super::compression::compress_file_inplace(&columns_path, compression)?;

        let timestamps_path = path.join("timestamps.bin");
        self.flush_timestamps(&timestamps_path)?;
        super::super::compression::compress_file_inplace(&timestamps_path, compression)?;

        Ok(())
    }

    fn flush_id_indexer(&self, path: &Path) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(path)?;
        write_header_to(&mut file, section::VERTEX_ID_INDEXER).map_err(|e| {
            StorageError::io_error(format!("Failed to write id_indexer header: {}", e))
        })?;

        let count = self.id_indexer.len() as u32;
        file.write_all(&count.to_le_bytes())?;

        let mut key_buf = Vec::new();
        for (key, id) in self.id_indexer.iter() {
            file.write_all(&id.to_le_bytes())?;
            key.write_to(&mut key_buf);
            file.write_all(&(key_buf.len() as u32).to_le_bytes())?;
            file.write_all(&key_buf)?;
        }

        Ok(())
    }

    fn flush_columns(&self, path: &Path) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(path)?;
        write_header_to(&mut file, section::VERTEX_COLUMNS).map_err(|e| {
            StorageError::io_error(format!("Failed to write columns header: {}", e))
        })?;

        let column_count = self.columns.column_count() as u32;
        file.write_all(&column_count.to_le_bytes())?;

        for col in self.columns.columns() {
            let name_bytes = col.name.as_bytes();
            file.write_all(&(name_bytes.len() as u32).to_le_bytes())?;
            file.write_all(name_bytes)?;

            let (data, offsets, bitmap) = col.get_flush_data();

            let row_count = offsets
                .len()
                .max(if data.is_empty() { 0 } else { col.len() });
            file.write_all(&(row_count as u32).to_le_bytes())?;

            file.write_all(&(data.len() as u32).to_le_bytes())?;
            file.write_all(&data)?;

            let offsets_count = offsets.len() as u32;
            file.write_all(&offsets_count.to_le_bytes())?;
            for &off in &offsets {
                file.write_all(&off.to_le_bytes())?;
            }

            if let Some(bitmap) = bitmap {
                file.write_all(&[1u8])?;
                let bitmap_bytes = bitmap.as_raw_slice();
                let bitmap_bit_len = bitmap.len() as u32;
                file.write_all(&bitmap_bit_len.to_le_bytes())?;
                file.write_all(&(bitmap_bytes.len() as u32).to_le_bytes())?;
                file.write_all(bitmap_bytes)?;
            } else {
                file.write_all(&[0u8])?;
            }

            let encoding_type = col.encoding_type().to_u8();
            file.write_all(&[encoding_type])?;
        }

        Ok(())
    }

    fn flush_timestamps(&self, path: &Path) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(path)?;
        write_header_to(&mut file, section::VERTEX_TIMESTAMPS).map_err(|e| {
            StorageError::io_error(format!("Failed to write timestamps header: {}", e))
        })?;

        let timestamps = self.timestamps.dump();
        let count = timestamps.len() as u32;
        file.write_all(&count.to_le_bytes())?;

        for ts in timestamps {
            file.write_all(&ts.to_le_bytes())?;
        }

        Ok(())
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> StorageResult<()> {
        use std::io::Read;

        let path = path.as_ref();

        let meta_path = path.join("meta.bin");
        let meta_data = super::super::compression::read_decompressed(&meta_path)?;
        let mut meta_cursor = &meta_data[..];
        let mut header_buf = [0u8; HEADER_SIZE];
        meta_cursor.read_exact(&mut header_buf)?;
        {
            let mut slice = &header_buf[..];
            let (_version, sid) = read_header(&mut slice)?;
            if sid != section::VERTEX_META {
                return Err(StorageError::deserialize_error(format!(
                    "unexpected section id in vertex meta: expected {:#06x}, got {:#06x}",
                    section::VERTEX_META,
                    sid
                )));
            }
        }

        let mut label_bytes = [0u8; 4];
        meta_cursor.read_exact(&mut label_bytes)?;
        self.label = u32::from_le_bytes(label_bytes);

        let mut label_name_len_bytes = [0u8; 4];
        meta_cursor.read_exact(&mut label_name_len_bytes)?;
        let label_name_len = u32::from_le_bytes(label_name_len_bytes) as usize;

        let mut label_name_bytes = vec![0u8; label_name_len];
        meta_cursor.read_exact(&mut label_name_bytes)?;
        self.label_name = String::from_utf8(label_name_bytes)
            .map_err(|e| StorageError::deserialize_error(e.to_string()))?;

        let mut schema_len_bytes = [0u8; 4];
        meta_cursor.read_exact(&mut schema_len_bytes)?;
        let schema_len = u32::from_le_bytes(schema_len_bytes) as usize;

        let mut schema_bytes = vec![0u8; schema_len];
        meta_cursor.read_exact(&mut schema_bytes)?;
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
        use std::io::Read;

        let data = super::super::compression::read_decompressed(path)?;
        let mut cursor = &data[..];
        let mut header_buf = [0u8; HEADER_SIZE];
        cursor.read_exact(&mut header_buf)?;
        {
            let mut slice = &header_buf[..];
            let (_version, sid) = read_header(&mut slice)?;
            if sid != section::VERTEX_ID_INDEXER {
                return Err(StorageError::deserialize_error(format!(
                    "unexpected section id in vertex id_indexer: expected {:#06x}, got {:#06x}",
                    section::VERTEX_ID_INDEXER,
                    sid
                )));
            }
        }

        let mut count_bytes = [0u8; 4];
        cursor.read_exact(&mut count_bytes)?;
        let count = u32::from_le_bytes(count_bytes) as usize;

        self.id_indexer.clear();

        for _ in 0..count {
            let mut id_bytes = [0u8; 4];
            cursor.read_exact(&mut id_bytes)?;
            let internal_id = u32::from_le_bytes(id_bytes);

            let mut key_len_bytes = [0u8; 4];
            cursor.read_exact(&mut key_len_bytes)?;
            let key_len = u32::from_le_bytes(key_len_bytes) as usize;

            let mut key_bytes = vec![0u8; key_len];
            cursor.read_exact(&mut key_bytes)?;
            let key = IdKey::from_bytes(&key_bytes)?;

            self.id_indexer.set_at(internal_id, key);
        }

        Ok(())
    }

    fn load_columns(&mut self, path: &Path) -> StorageResult<()> {
        use std::io::Read;

        let data = super::super::compression::read_decompressed(path)?;
        let mut cursor = &data[..];
        let mut header_buf = [0u8; HEADER_SIZE];
        cursor.read_exact(&mut header_buf)?;
        {
            let mut slice = &header_buf[..];
            let (_version, sid) = read_header(&mut slice)?;
            if sid != section::VERTEX_COLUMNS {
                return Err(StorageError::deserialize_error(format!(
                    "unexpected section id in vertex columns: expected {:#06x}, got {:#06x}",
                    section::VERTEX_COLUMNS,
                    sid
                )));
            }
        }

        let mut column_count_bytes = [0u8; 4];
        cursor.read_exact(&mut column_count_bytes)?;
        let column_count = u32::from_le_bytes(column_count_bytes) as usize;

        self.columns.clear();

        for _ in 0..column_count {
            let mut name_len_bytes = [0u8; 4];
            cursor.read_exact(&mut name_len_bytes)?;
            let name_len = u32::from_le_bytes(name_len_bytes) as usize;

            let mut name_bytes = vec![0u8; name_len];
            cursor.read_exact(&mut name_bytes)?;
            let name = String::from_utf8(name_bytes)
                .map_err(|e| StorageError::deserialize_error(e.to_string()))?;

            let mut row_count_bytes = [0u8; 4];
            cursor.read_exact(&mut row_count_bytes)?;
            let _row_count = u32::from_le_bytes(row_count_bytes) as usize;

            let mut data_len_bytes = [0u8; 4];
            cursor.read_exact(&mut data_len_bytes)?;
            let data_len = u32::from_le_bytes(data_len_bytes) as usize;

            let mut data = vec![0u8; data_len];
            cursor.read_exact(&mut data)?;

            let mut offsets_count_bytes = [0u8; 4];
            cursor.read_exact(&mut offsets_count_bytes)?;
            let offsets_count = u32::from_le_bytes(offsets_count_bytes) as usize;

            let mut offsets = Vec::with_capacity(offsets_count);
            for _ in 0..offsets_count {
                let mut off_bytes = [0u8; 8];
                cursor.read_exact(&mut off_bytes)?;
                offsets.push(u64::from_le_bytes(off_bytes));
            }

            let mut has_bitmap_bytes = [0u8; 1];
            cursor.read_exact(&mut has_bitmap_bytes)?;
            let has_bitmap = has_bitmap_bytes[0] == 1;

            let (null_bitmap_raw, bitmap_bit_len) = if has_bitmap {
                let mut bitmap_bit_len_bytes = [0u8; 4];
                cursor.read_exact(&mut bitmap_bit_len_bytes)?;
                let bitmap_bit_len = u32::from_le_bytes(bitmap_bit_len_bytes) as usize;

                let mut bitmap_bytes_len_bytes = [0u8; 4];
                cursor.read_exact(&mut bitmap_bytes_len_bytes)?;
                let bitmap_bytes_len = u32::from_le_bytes(bitmap_bytes_len_bytes) as usize;

                let mut bitmap_bytes = vec![0u8; bitmap_bytes_len];
                cursor.read_exact(&mut bitmap_bytes)?;

                (Some(bitmap_bytes), bitmap_bit_len)
            } else {
                (None, 0)
            };

            self.columns.load_column_from_raw(
                &name,
                data,
                offsets,
                null_bitmap_raw,
                bitmap_bit_len,
            )?;

            let mut encoding_byte_bytes = [0u8; 1];
            cursor.read_exact(&mut encoding_byte_bytes)?;
            let encoding_type = EncodingType::from_u8(encoding_byte_bytes[0]);
            if encoding_type != EncodingType::None {
                self.columns
                    .apply_encoding_to_column(&name, encoding_type)?;
            }
        }

        Ok(())
    }

    fn load_timestamps(&mut self, path: &Path) -> StorageResult<()> {
        use std::io::Read;

        let data = super::super::compression::read_decompressed(path)?;
        let mut cursor = &data[..];
        let mut header_buf = [0u8; HEADER_SIZE];
        cursor.read_exact(&mut header_buf)?;
        {
            let mut slice = &header_buf[..];
            let (_version, sid) = read_header(&mut slice)?;
            if sid != section::VERTEX_TIMESTAMPS {
                return Err(StorageError::deserialize_error(format!(
                    "unexpected section id in vertex timestamps: expected {:#06x}, got {:#06x}",
                    section::VERTEX_TIMESTAMPS,
                    sid
                )));
            }
        }

        let mut count_bytes = [0u8; 4];
        cursor.read_exact(&mut count_bytes)?;
        let count = u32::from_le_bytes(count_bytes) as usize;

        let mut timestamps = Vec::with_capacity(count);
        for _ in 0..count {
            let mut ts_bytes = [0u8; 4];
            cursor.read_exact(&mut ts_bytes)?;
            timestamps.push(u32::from_le_bytes(ts_bytes));
        }

        self.timestamps.load(&timestamps);

        Ok(())
    }

    pub fn compact_with_ts_collect(&mut self, ts: Timestamp) -> Vec<IdKey> {
        let deleted_ids: Vec<u32> = self.timestamps.iter_deleted(ts).collect();

        let mut removed_keys = Vec::with_capacity(deleted_ids.len());

        for id in &deleted_ids {
            if let Some(key) = self.id_indexer.get_key(*id) {
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

        total += self.timestamps.valid_count(super::MAX_TIMESTAMP - 1)
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
    use crate::storage::vertex::VertexSchema;

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
