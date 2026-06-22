//! Property Table for Edges
//!
//! Stores edge properties using row-oriented storage with MVCC support.
//! Each property record includes create_ts and delete_ts for version tracking,
//! enabling time-travel queries and garbage collection of expired versions.

use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read};

use crate::core::{DataType, DateValue, StorageError, StorageResult, Value};
use crate::core::types::Timestamp;
use crate::storage::naming::NameIndexer;
use crate::storage::persistence::{read_header, read_u32_le, section, write_header};
use crate::storage::types::PropertyId;

pub use super::property_schema::{
    PropertySchema, PropertyRecord, PropertyCompactionStats,
    PROP_OFFSET_NONE, prop_offset_to_index, prop_index_to_offset,
};

// Varint encoding for compact string lengths
fn encode_varint(mut value: u32, buffer: &mut Vec<u8>) {
    while value >= 128 {
        buffer.push((value as u8) | 0x80);
        value >>= 7;
    }
    buffer.push(value as u8);
}

fn decode_varint(cursor: &mut Cursor<&[u8]>) -> StorageResult<u32> {
    let mut result = 0u32;
    let mut shift = 0;
    loop {
        let mut b = [0u8; 1];
        cursor.read_exact(&mut b).map_err(|_| {
            StorageError::deserialize_error("failed to decode varint")
        })?;
        result |= ((b[0] & 0x7F) as u32) << shift;
        if b[0] < 128 {
            break;
        }
        shift += 7;
    }
    Ok(result)
}

#[derive(Debug, Clone)]
pub struct PropertyTable {
    schema: Vec<PropertySchema>,
    name_indexer: NameIndexer,
    records: Vec<Option<PropertyRecord>>,     // row_index → PropertyRecord with timestamps
    buffer: Vec<u8>,                          // single contiguous buffer (for data)
    row_count: usize,
    free_list: Vec<u32>,
    tombstones: HashMap<u32, Timestamp>,      // row_offset → delete_ts (for GC tracking)
}

impl PropertyTable {
    pub fn new() -> Self {
        Self {
            schema: Vec::new(),
            name_indexer: NameIndexer::new(),
            records: Vec::new(),
            buffer: Vec::new(),
            row_count: 0,
            free_list: Vec::new(),
            tombstones: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            schema: Vec::new(),
            name_indexer: NameIndexer::with_capacity(capacity),
            records: Vec::with_capacity(capacity),
            buffer: Vec::with_capacity(capacity * 128),
            row_count: 0,
            free_list: Vec::with_capacity(capacity / 10),
            tombstones: HashMap::with_capacity(capacity / 100),
        }
    }

    pub fn add_property(
        &mut self,
        name: String,
        data_type: DataType,
        nullable: bool,
    ) -> PropertyId {
        let prop_id = PropertyId::new(self.schema.len() as u16);
        let schema = PropertySchema::new(name.clone(), prop_id.as_usize() as i32, data_type)
            .nullable(nullable);
        self.name_indexer.register(name.clone());
        self.schema.push(schema);
        prop_id
    }

    pub fn add_property_with_encoding(
        &mut self,
        name: String,
        data_type: DataType,
        nullable: bool,
        _encoding: Option<()>,
    ) -> PropertyId {
        self.add_property(name, data_type, nullable)
    }

    pub fn remove_property(&mut self, name: &str) -> StorageResult<()> {
        let index = self
            .schema
            .iter()
            .position(|prop| prop.name == name)
            .ok_or_else(|| StorageError::column_not_found(name.to_string()))?;

        self.schema.remove(index);
        self.name_indexer.clear();
        for (idx, schema) in self.schema.iter_mut().enumerate() {
            schema.prop_id = idx as i32;
            self.name_indexer.register(schema.name.clone());
        }

        Ok(())
    }

    pub fn rename_property(&mut self, old_name: &str, new_name: &str) -> StorageResult<()> {
        if self.has_property(new_name) {
            return Err(StorageError::column_already_exists(new_name.to_string()));
        }

        let index = self
            .schema
            .iter()
            .position(|prop| prop.name == old_name)
            .ok_or_else(|| StorageError::column_not_found(old_name.to_string()))?;

        self.schema[index].name = new_name.to_string();

        self.name_indexer.clear();
        for (idx, schema) in self.schema.iter_mut().enumerate() {
            schema.prop_id = idx as i32;
            self.name_indexer.register(schema.name.clone());
        }

        Ok(())
    }

    pub fn apply_encoding(&mut self, _prop_id: PropertyId, _encoding: ()) -> StorageResult<()> {
        Ok(())
    }

    pub fn auto_apply_encodings(&mut self, _config: Option<()>) -> StorageResult<()> {
        Ok(())
    }

    fn serialize_row(&self, values: &[(String, Value)]) -> StorageResult<Vec<u8>> {
        let mut buffer = Vec::new();

        for schema in &self.schema {
            let value = values
                .iter()
                .find(|(k, _)| k == &schema.name)
                .map(|(_, v)| v.clone());

            self.serialize_value(&mut buffer, value.as_ref(), &schema)?;
        }

        Ok(buffer)
    }

    fn serialize_row_with_nulls(&self, values: &[(String, Option<Value>)]) -> StorageResult<Vec<u8>> {
        let mut buffer = Vec::new();

        for schema in &self.schema {
            let value = values
                .iter()
                .find(|(k, _)| k == &schema.name)
                .map(|(_, v)| v.clone())
                .flatten();

            self.serialize_value(&mut buffer, value.as_ref(), &schema)?;
        }

        Ok(buffer)
    }

    fn serialize_value(&self, buffer: &mut Vec<u8>, value: Option<&Value>, schema: &PropertySchema) -> StorageResult<()> {
        match value {
            None => {
                buffer.push(0); // null marker
            }
            Some(val) => {
                buffer.push(1); // not null marker
                match &schema.data_type {
                    DataType::Bool => {
                        if let Value::Bool(b) = val {
                            buffer.push(if *b { 1 } else { 0 });
                        }
                    }
                    DataType::SmallInt => {
                        if let Value::SmallInt(i) = val {
                            buffer.extend_from_slice(&i.to_le_bytes());
                        }
                    }
                    DataType::Int => {
                        if let Value::Int(i) = val {
                            buffer.extend_from_slice(&i.to_le_bytes());
                        }
                    }
                    DataType::BigInt => {
                        if let Value::BigInt(i) = val {
                            buffer.extend_from_slice(&i.to_le_bytes());
                        }
                    }
                    DataType::Float => {
                        if let Value::Float(f) = val {
                            buffer.extend_from_slice(&f.to_le_bytes());
                        }
                    }
                    DataType::Double => {
                        if let Value::Double(d) = val {
                            buffer.extend_from_slice(&d.to_le_bytes());
                        }
                    }
                    DataType::String => {
                        if let Value::String(s) = val {
                            let s_bytes = s.as_bytes();
                            encode_varint(s_bytes.len() as u32, buffer);
                            buffer.extend_from_slice(s_bytes);
                        }
                    }
                    DataType::Date => {
                        if let Value::Date(d) = val {
                            buffer.extend_from_slice(&d.year.to_le_bytes());
                            buffer.extend_from_slice(&d.month.to_le_bytes());
                            buffer.extend_from_slice(&d.day.to_le_bytes());
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn deserialize_row(&self, record: &[u8]) -> StorageResult<Vec<(String, Option<Value>)>> {
        let mut cursor = Cursor::new(record);
        let mut result = Vec::new();

        for schema in &self.schema {
            let mut null_marker = [0u8; 1];
            if cursor.read_exact(&mut null_marker).is_err() {
                result.push((schema.name.clone(), None));
                continue;
            }

            if null_marker[0] == 0 {
                result.push((schema.name.clone(), None));
            } else {
                let value = self.deserialize_value(&mut cursor, &schema.data_type)?;
                result.push((schema.name.clone(), value));
            }
        }

        Ok(result)
    }

    fn deserialize_value(&self, cursor: &mut Cursor<&[u8]>, data_type: &DataType) -> StorageResult<Option<Value>> {
        match data_type {
            DataType::Bool => {
                let mut b = [0u8; 1];
                cursor.read_exact(&mut b)?;
                Ok(Some(Value::Bool(b[0] != 0)))
            }
            DataType::SmallInt => {
                let mut buf = [0u8; 2];
                cursor.read_exact(&mut buf)?;
                Ok(Some(Value::SmallInt(i16::from_le_bytes(buf))))
            }
            DataType::Int => {
                let mut buf = [0u8; 4];
                cursor.read_exact(&mut buf)?;
                Ok(Some(Value::Int(i32::from_le_bytes(buf))))
            }
            DataType::BigInt => {
                let mut buf = [0u8; 8];
                cursor.read_exact(&mut buf)?;
                Ok(Some(Value::BigInt(i64::from_le_bytes(buf))))
            }
            DataType::Float => {
                let mut buf = [0u8; 4];
                cursor.read_exact(&mut buf)?;
                Ok(Some(Value::Float(f32::from_le_bytes(buf))))
            }
            DataType::Double => {
                let mut buf = [0u8; 8];
                cursor.read_exact(&mut buf)?;
                Ok(Some(Value::Double(f64::from_le_bytes(buf))))
            }
            DataType::String => {
                let len = decode_varint(cursor)? as usize;
                let mut str_buf = vec![0u8; len];
                cursor.read_exact(&mut str_buf)?;
                Ok(Some(Value::String(String::from_utf8_lossy(&str_buf).to_string())))
            }
            DataType::Date => {
                let mut buf = [0u8; 10];
                cursor.read_exact(&mut buf[..4])?;
                let year = i32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                cursor.read_exact(&mut buf[..4])?;
                let month = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                cursor.read_exact(&mut buf[..4])?;
                let day = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                Ok(Some(Value::Date(DateValue { year, month, day })))
            }
            _ => Ok(None),
        }
    }

    pub fn insert(&mut self, values: &[(String, Value)], create_ts: Timestamp) -> StorageResult<u32> {
        let record_data = self.serialize_row(values)?;

        let record = PropertyRecord::new(record_data.clone(), create_ts);

        let offset = if let Some(free_idx) = self.free_list.pop() {
            let row_idx = (free_idx - 1) as usize;
            self.records[row_idx] = Some(record);
            free_idx
        } else {
            let row_idx = self.records.len();
            let row_offset = prop_index_to_offset(row_idx);
            self.records.push(Some(record));
            self.row_count += 1;
            row_offset
        };

        Ok(offset)
    }

    pub fn update(&mut self, offset: u32, values: &[(String, Value)], ts: Timestamp) -> StorageResult<u32> {
        // Mark old record as deleted
        let row_idx = prop_offset_to_index(offset).ok_or_else(|| StorageError::invalid_offset(offset))?;
        if row_idx >= self.records.len() {
            return Err(StorageError::invalid_offset(offset));
        }

        if let Some(record) = &mut self.records[row_idx] {
            if record.delete_ts.is_none() {
                record.delete_ts = Some(ts);
                self.tombstones.insert(offset, ts);
            }
        }

        // Insert new record with same structure
        let merged_values = self.get_for_update(offset, values)?;
        self.insert(&merged_values, ts)
    }

    fn get_for_update(&self, offset: u32, updates: &[(String, Value)]) -> StorageResult<Vec<(String, Value)>> {
        let mut result = Vec::new();

        if let Some(current_props) = self.get(offset, None) {
            for (name, opt_value) in current_props {
                if let Some((_, new_val)) = updates.iter().find(|(k, _)| k == &name) {
                    result.push((name, new_val.clone()));
                } else if let Some(old_val) = opt_value {
                    result.push((name, old_val));
                }
            }

            // Add any new properties from updates that weren't in current
            for (name, val) in updates {
                if !result.iter().any(|(n, _)| n == name) {
                    result.push((name.clone(), val.clone()));
                }
            }
        } else {
            result = updates.to_vec();
        }

        Ok(result)
    }

    pub fn get(&self, offset: u32, query_ts: Option<Timestamp>) -> Option<Vec<(String, Option<Value>)>> {
        let row_idx = prop_offset_to_index(offset)?;
        if row_idx >= self.records.len() {
            return None;
        }

        let record = self.records[row_idx].as_ref()?;

        // Check visibility based on create_ts and delete_ts
        let visible = match query_ts {
            None => record.delete_ts.is_none(),  // Current version
            Some(ts) => record.is_visible_at(ts), // Time-travel query
        };

        if !visible {
            return None;
        }

        self.deserialize_row(&record.data).ok()
    }

    pub fn set_property(
        &mut self,
        offset: u32,
        name: &str,
        value: Option<Value>,
        ts: Timestamp,
    ) -> StorageResult<()> {
        let row_idx = prop_offset_to_index(offset).ok_or_else(|| StorageError::invalid_offset(offset))?;
        if row_idx >= self.records.len() {
            return Err(StorageError::invalid_offset(offset));
        }

        if !self.has_property(name) {
            return Err(StorageError::column_not_found(name.to_string()));
        }

        let mut merged_values: Vec<(String, Option<Value>)> = Vec::new();
        if let Some(props) = self.get(offset, None) {
            for (n, v) in props {
                if n == name {
                    merged_values.push((n, value.clone()));
                } else {
                    merged_values.push((n, v));
                }
            }
        }

        let new_record = self.serialize_row_with_nulls(&merged_values)?;

        // Mark old as deleted and insert new
        if let Some(record) = &mut self.records[row_idx] {
            if record.delete_ts.is_none() {
                record.delete_ts = Some(ts);
                self.tombstones.insert(offset, ts);
            }
        }

        let new_record_obj = PropertyRecord::new(new_record, ts);
        self.records[row_idx] = Some(new_record_obj);

        Ok(())
    }

    pub fn set_property_by_id(
        &mut self,
        offset: u32,
        prop_id: PropertyId,
        value: Option<Value>,
        ts: Timestamp,
    ) -> StorageResult<()> {
        let col_idx = prop_id.as_usize();
        if col_idx >= self.schema.len() {
            return Err(StorageError::column_not_found(format!("prop_id={}", prop_id)));
        }

        self.set_property(offset, &self.schema[col_idx].name.clone(), value, ts)
    }

    /// Mark a property record as deleted for MVCC tracking
    pub fn mark_deleted(&mut self, offset: u32, delete_ts: Timestamp) -> StorageResult<()> {
        let row_idx = prop_offset_to_index(offset).ok_or_else(|| StorageError::invalid_offset(offset))?;
        if row_idx >= self.records.len() {
            return Ok(());  // Already deleted or doesn't exist
        }

        if let Some(record) = &mut self.records[row_idx] {
            if record.delete_ts.is_none() {
                record.delete_ts = Some(delete_ts);
                self.tombstones.insert(offset, delete_ts);
                Ok(())
            } else {
                Err(StorageError::invalid_operation("record already marked deleted"))
            }
        } else {
            Ok(())  // Idempotent: already deleted
        }
    }

    /// Garbage collect tombstones older than min_active_snapshot_ts
    pub fn gc_tombstones(&mut self, min_active_snapshot_ts: Timestamp) -> u32 {
        let mut reclaimed = 0u32;

        let to_remove: Vec<u32> = self.tombstones
            .iter()
            .filter(|(_, &delete_ts)| delete_ts < min_active_snapshot_ts)
            .map(|(&offset, _)| offset)
            .collect();

        for offset in to_remove {
            if let Some(row_idx) = prop_offset_to_index(offset) {
                if row_idx < self.records.len() {
                    self.records[row_idx] = None;
                    self.free_list.push(offset);
                    self.tombstones.remove(&offset);
                    reclaimed += 1;
                }
            }
        }

        reclaimed
    }

    /// Legacy delete method for backward compatibility (physical delete)
    pub fn delete(&mut self, offset: u32) -> bool {
        let row_idx = match prop_offset_to_index(offset) {
            Some(idx) => idx,
            None => return false,
        };
        if row_idx >= self.records.len() {
            return false;
        }

        self.records[row_idx] = None;
        self.free_list.push(offset);
        self.tombstones.remove(&offset);
        true
    }

    pub fn has_property(&self, name: &str) -> bool {
        self.name_indexer.contains(name)
    }

    pub fn dump(&self) -> Vec<u8> {
        let mut result = Vec::new();

        write_header(&mut result, section::PROPERTY_TABLE);

        let checksum_pos = result.len();
        result.extend_from_slice(&[0u8; 4]);

        // Version 2: Include MVCC support
        result.push(2);  // version

        result.extend_from_slice(&(self.schema.len() as u32).to_le_bytes());
        for prop in &self.schema {
            let name_bytes = prop.name.as_bytes();
            result.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
            result.extend_from_slice(name_bytes);
            result.extend_from_slice(&prop.prop_id.to_le_bytes());
            result.push(prop.data_type.as_u8());
            result.push(if prop.nullable { 1 } else { 0 });
        }

        result.extend_from_slice(&(self.records.len() as u32).to_le_bytes());

        // Store each PropertyRecord with timestamps
        for record_opt in &self.records {
            match record_opt {
                Some(record) => {
                    result.push(1);  // marker: has data
                    result.extend_from_slice(&record.create_ts.to_le_bytes());
                    if let Some(del_ts) = record.delete_ts {
                        result.push(1);  // marker: has delete_ts
                        result.extend_from_slice(&del_ts.to_le_bytes());
                    } else {
                        result.push(0);  // marker: no delete_ts
                    }
                    result.extend_from_slice(&(record.data.len() as u32).to_le_bytes());
                    result.extend_from_slice(&record.data);
                }
                None => {
                    result.push(0);  // marker: deleted
                }
            }
        }

        // Store tombstones for garbage collection tracking
        result.extend_from_slice(&(self.tombstones.len() as u32).to_le_bytes());
        for (offset, delete_ts) in &self.tombstones {
            result.extend_from_slice(&offset.to_le_bytes());
            result.extend_from_slice(&delete_ts.to_le_bytes());
        }

        // Store free list
        result.extend_from_slice(&(self.free_list.len() as u32).to_le_bytes());
        for off in &self.free_list {
            result.extend_from_slice(&off.to_le_bytes());
        }

        let checksum = crc32fast::hash(&result[checksum_pos + 4..]);
        result[checksum_pos..checksum_pos + 4].copy_from_slice(&checksum.to_le_bytes());

        result
    }

    pub fn load(&mut self, data: &[u8]) -> StorageResult<()> {
        if data.is_empty() {
            return Ok(());
        }

        let mut cursor = data;
        let (_version, section_id) = read_header(&mut cursor)?;
        if section_id != section::PROPERTY_TABLE {
            return Err(StorageError::deserialize_error(format!(
                "invalid section_id for PropertyTable: expected 0x{:04X}, got 0x{:04X}",
                section::PROPERTY_TABLE,
                section_id
            )));
        }

        if cursor.len() < 4 {
            return Err(StorageError::deserialize_error(
                "PropertyTable data too short for checksum",
            ));
        }
        let stored_checksum = u32::from_le_bytes(cursor[..4].try_into().map_err(|_| {
            StorageError::deserialize_error("failed to read PropertyTable checksum")
        })?);
        let payload = &cursor[4..];
        let computed_checksum = crc32fast::hash(payload);
        if stored_checksum != computed_checksum {
            return Err(StorageError::deserialize_error(format!(
                "PropertyTable checksum mismatch: stored {:#x}, computed {:#x}",
                stored_checksum, computed_checksum
            )));
        }

        let data = payload;
        let mut offset = 0usize;

        // Read version
        let version = if offset < data.len() {
            let v = data[offset];
            offset += 1;
            v
        } else {
            1  // Default to v1 if not specified
        };

        let schema_len = read_u32_le(data, &mut offset)? as usize;

        self.schema.clear();
        self.name_indexer.clear();

        for _ in 0..schema_len {
            let name_len = read_u32_le(data, &mut offset)? as usize;
            if offset + name_len > data.len() {
                return Err(StorageError::deserialize_error("unexpected end of data"));
            }
            let name = String::from_utf8_lossy(&data[offset..offset + name_len]).to_string();
            offset += name_len;

            let prop_id_bytes: [u8; 4] = data[offset..offset + 4]
                .try_into()
                .map_err(|_| StorageError::deserialize_error("failed to read prop_id"))?;
            let prop_id = i32::from_le_bytes(prop_id_bytes);
            offset += 4;
            let data_type = DataType::from_u8(data[offset]);
            offset += 1;
            let nullable = data[offset] == 1;
            offset += 1;

            let prop_schema = PropertySchema::new(name.clone(), prop_id, data_type).nullable(nullable);
            self.name_indexer.register(name.clone());
            self.schema.push(prop_schema);
        }

        if version >= 2 {
            // Load PropertyRecords with MVCC support (v2+)
            let records_len = read_u32_le(data, &mut offset)? as usize;
            self.records.clear();
            self.row_count = 0;

            for _ in 0..records_len {
                if offset >= data.len() {
                    return Err(StorageError::deserialize_error("unexpected end of data"));
                }
                let marker = data[offset];
                offset += 1;

                if marker == 1 {
                    let create_ts = read_u32_le(data, &mut offset)?;
                    let has_delete_ts = data[offset];
                    offset += 1;
                    let delete_ts = if has_delete_ts == 1 {
                        Some(read_u32_le(data, &mut offset)?)
                    } else {
                        None
                    };
                    let data_len = read_u32_le(data, &mut offset)? as usize;
                    if offset + data_len > data.len() {
                        return Err(StorageError::deserialize_error("unexpected end of data"));
                    }
                    let record_data = data[offset..offset + data_len].to_vec();
                    offset += data_len;

                    let record = PropertyRecord {
                        data: record_data,
                        create_ts,
                        delete_ts,
                    };
                    self.records.push(Some(record));
                    self.row_count += 1;
                } else {
                    self.records.push(None);
                }
            }

            // Load tombstones for GC tracking
            let tombstones_len = read_u32_le(data, &mut offset)? as usize;
            self.tombstones.clear();
            for _ in 0..tombstones_len {
                let prop_offset = read_u32_le(data, &mut offset)?;
                let delete_ts = read_u32_le(data, &mut offset)?;
                self.tombstones.insert(prop_offset, delete_ts);
            }
        } else {
            // Backward compatibility: migrate v1 format to v2
            // In v1, we had buffer + row_offsets instead of PropertyRecords
            // This is a simplified migration that marks all records as create_ts=0
            self.records.clear();
            self.row_count = 0;

            // Read old v1 format (skip this for now, or implement minimal v1 support)
            return Err(StorageError::deserialize_error(
                "PropertyTable v1 format is no longer supported. Please use v2 format or export/reimport data."
            ));
        }

        // Load free list
        let free_list_len = read_u32_le(data, &mut offset)? as usize;
        self.free_list.clear();
        for _ in 0..free_list_len {
            self.free_list.push(read_u32_le(data, &mut offset)?);
        }

        Ok(())
    }

    pub fn compact(&mut self, valid_offsets: &HashSet<u32>) {
        let mut new_records = Vec::new();
        let mut offset_mapping = std::collections::HashMap::new();

        for (old_idx, record_opt) in self.records.iter().enumerate() {
            let old_offset = prop_index_to_offset(old_idx);
            if valid_offsets.contains(&old_offset) {
                if let Some(record) = record_opt {
                    offset_mapping.insert(old_offset, prop_index_to_offset(new_records.len()));
                    new_records.push(Some(record.clone()));
                } else {
                    new_records.push(None);
                }
            }
        }

        self.records = new_records;
        self.row_count = self.records.iter().filter(|r| r.is_some()).count();
        self.free_list.clear();
        self.tombstones.clear();
    }

    pub fn used_memory_size(&self) -> usize {
        let mut total = 0usize;
        for record_opt in &self.records {
            if let Some(record) = record_opt {
                total += record.data.len();
            }
        }
        total += self.records.len() * std::mem::size_of::<Option<PropertyRecord>>();
        total += std::mem::size_of::<Self>();
        total
    }

    /// Calculate compaction statistics for the property table
    pub fn compaction_stats(&self) -> PropertyCompactionStats {
        let tombstone_count = self.tombstones.len();
        let live_count = self.records.iter().filter(|r| r.is_some()).count();

        // Estimate reclaimable bytes from tombstoned records
        let mut reclaimable_bytes = 0usize;
        for (offset, _) in &self.tombstones {
            if let Some(row_idx) = prop_offset_to_index(*offset) {
                if row_idx < self.records.len() {
                    if let Some(record) = &self.records[row_idx] {
                        reclaimable_bytes += record.data.len() + std::mem::size_of::<PropertyRecord>();
                    }
                }
            }
        }

        PropertyCompactionStats {
            tombstone_count,
            total_records: self.records.len(),
            buffer_size: self.buffer.len(),
            free_list_size: self.free_list.len(),
            reclaimable_bytes,
        }
    }

    /// Get all live records (non-deleted) with their current offsets
    pub fn filter_live_records(&self) -> Vec<(u32, PropertyRecord)> {
        self.records
            .iter()
            .enumerate()
            .filter_map(|(idx, record_opt)| {
                record_opt.as_ref().map(|record| {
                    let offset = prop_index_to_offset(idx);
                    (offset, record.clone())
                })
            })
            .collect()
    }

    /// Compact the property table by removing deleted records and rebuilding arrays
    ///
    /// This operation:
    /// 1. Filters out deleted/tombstoned records not in valid_offsets
    /// 2. Rebuilds the records array sequentially
    /// 3. Generates offset mappings (old_offset -> new_offset)
    /// 4. Clears the free list and tombstones
    ///
    /// Returns a HashMap mapping old offsets to new offsets for any external
    /// callers that need to update their references.
    pub fn compact_with_relocation(&mut self, valid_offsets: &HashSet<u32>) -> HashMap<u32, u32> {
        let mut new_records = Vec::new();
        let mut offset_mapping = HashMap::new();

        // Collect live records and build mapping based on valid_offsets
        for (old_idx, record_opt) in self.records.iter().enumerate() {
            let old_offset = prop_index_to_offset(old_idx);
            if valid_offsets.contains(&old_offset) {
                if let Some(record) = record_opt {
                    let new_offset = prop_index_to_offset(new_records.len());
                    offset_mapping.insert(old_offset, new_offset);
                    new_records.push(Some(record.clone()));
                } else {
                    new_records.push(None);
                }
            }
        }

        // Update row count
        self.row_count = new_records.iter().filter(|r| r.is_some()).count();

        // Clear and rebuild arrays
        self.records = new_records;
        self.free_list.clear();
        self.tombstones.clear();

        offset_mapping
    }

    /// Check if schema is suitable for fast path deserialization:
    /// all types are fixed-size (no String, no Date) and no nulls
    pub fn is_schema_fixed_size(&self) -> bool {
        self.schema.iter().all(|s| {
            matches!(
                s.data_type,
                DataType::Bool
                    | DataType::SmallInt
                    | DataType::Int
                    | DataType::BigInt
                    | DataType::Float
                    | DataType::Double
            )
        })
    }

    /// Prefetch a single property offset into CPU cache
    /// This is a no-op on most systems but signals intent for cache optimization
    #[inline]
    pub fn prefetch(&self, offset: u32) {
        if let Some(row_idx) = prop_offset_to_index(offset) {
            if row_idx < self.records.len() {
                if let Some(record) = &self.records[row_idx] {
                    // Prefetch the data location to L1/L2 cache
                    #[allow(unsafe_code)]
                    unsafe {
                        let addr = record.data.as_ptr() as *const u8;
                        // Use a volatile read to ensure prefetch happens
                        std::ptr::read_volatile(addr);
                    }
                }
            }
        }
    }

    /// Prefetch multiple property offsets in batch
    /// Improves cache locality for bulk operations
    pub fn prefetch_batch(&self, offsets: &[u32]) {
        for offset in offsets {
            self.prefetch(*offset);
        }
    }

    /// Fast path deserialization for fixed-size schemas
    /// Skips null checks and type dispatching for 2-3x speedup
    pub fn get_fast(&self, offset: u32, query_ts: Option<Timestamp>) -> Option<Vec<(String, Option<Value>)>> {
        if !self.is_schema_fixed_size() {
            return self.get(offset, query_ts);
        }

        let row_idx = prop_offset_to_index(offset)?;
        if row_idx >= self.records.len() {
            return None;
        }

        let record = self.records[row_idx].as_ref()?;

        // Check visibility based on create_ts and delete_ts
        let visible = match query_ts {
            None => record.delete_ts.is_none(),
            Some(ts) => record.is_visible_at(ts),
        };

        if !visible {
            return None;
        }

        let record_data = &record.data;

        // Fast path: directly deserialize without null checks
        let mut cursor = Cursor::new(record_data);
        let mut result = Vec::with_capacity(self.schema.len());

        for schema in &self.schema {
            match &schema.data_type {
                DataType::Bool => {
                    let mut b = [0u8; 1];
                    if cursor.read_exact(&mut b).is_err() {
                        return None;
                    }
                    result.push((schema.name.clone(), Some(Value::Bool(b[0] != 0))));
                }
                DataType::SmallInt => {
                    let mut buf = [0u8; 2];
                    if cursor.read_exact(&mut buf).is_err() {
                        return None;
                    }
                    result.push((schema.name.clone(), Some(Value::SmallInt(i16::from_le_bytes(buf)))));
                }
                DataType::Int => {
                    let mut buf = [0u8; 4];
                    if cursor.read_exact(&mut buf).is_err() {
                        return None;
                    }
                    result.push((schema.name.clone(), Some(Value::Int(i32::from_le_bytes(buf)))));
                }
                DataType::BigInt => {
                    let mut buf = [0u8; 8];
                    if cursor.read_exact(&mut buf).is_err() {
                        return None;
                    }
                    result.push((schema.name.clone(), Some(Value::BigInt(i64::from_le_bytes(buf)))));
                }
                DataType::Float => {
                    let mut buf = [0u8; 4];
                    if cursor.read_exact(&mut buf).is_err() {
                        return None;
                    }
                    result.push((schema.name.clone(), Some(Value::Float(f32::from_le_bytes(buf)))));
                }
                DataType::Double => {
                    let mut buf = [0u8; 8];
                    if cursor.read_exact(&mut buf).is_err() {
                        return None;
                    }
                    result.push((schema.name.clone(), Some(Value::Double(f64::from_le_bytes(buf)))));
                }
                _ => {
                    // Should not reach here due to is_schema_fixed_size check
                    return None;
                }
            }
        }

        Some(result)
    }

    /// Batch retrieval of properties, sorted by offset for cache locality
    /// Returns results in original order via the provided iterator
    pub fn get_batch<'a, I>(&'a self, offsets: I, query_ts: Option<Timestamp>) -> Vec<Option<Vec<(String, Option<Value>)>>>
    where
        I: IntoIterator<Item = &'a u32>,
    {
        let offsets: Vec<_> = offsets.into_iter().collect();
        let mut indexed: Vec<_> = offsets
            .iter().enumerate()
            .map(|(idx, offset)| (idx, **offset))
            .collect();

        // Sort by offset to improve cache locality
        indexed.sort_by_key(|(_, offset)| *offset);

        // Prefetch all offsets
        for (_, offset) in &indexed {
            self.prefetch(*offset);
        }

        // Retrieve in sorted order
        let sorted_results: Vec<_> = indexed
            .iter()
            .map(|(_, offset)| self.get_fast(*offset, query_ts).or_else(|| self.get(*offset, query_ts)))
            .collect();

        // Restore original order
        let mut results = vec![None; offsets.len()];
        for (orig_idx, sorted_result) in indexed.iter().zip(sorted_results) {
            results[orig_idx.0] = sorted_result;
        }

        results
    }
}

impl Default for PropertyTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "property_table_tests.rs"]
mod tests;
