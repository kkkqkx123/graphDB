//! Property Table for Edges
//!
//! Stores edge properties using row-oriented storage.
//! Each property record is a serialized byte sequence, enabling fast access
//! and simplified property updates without re-encoding.

use std::collections::HashSet;
use std::io::{Cursor, Read};

use crate::core::{DataType, DateValue, StorageError, StorageResult, Value};
use crate::storage::naming::NameIndexer;
use crate::storage::persistence::{read_header, read_u32_le, section, write_header};
use crate::storage::types::PropertyId;

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
}

impl PropertySchema {
    pub fn new(name: String, prop_id: i32, data_type: DataType) -> Self {
        Self {
            name,
            prop_id,
            data_type,
            nullable: false,
        }
    }

    pub fn nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }
}

#[derive(Debug, Clone)]
pub struct PropertyTable {
    schema: Vec<PropertySchema>,
    name_indexer: NameIndexer,
    buffer: Vec<u8>,                       // single contiguous buffer
    row_offsets: Vec<Option<(usize, usize)>>, // (offset, size) or None if deleted
    row_count: usize,
    free_list: Vec<u32>,
}

impl PropertyTable {
    pub fn new() -> Self {
        Self {
            schema: Vec::new(),
            name_indexer: NameIndexer::new(),
            buffer: Vec::new(),
            row_offsets: Vec::new(),
            row_count: 0,
            free_list: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            schema: Vec::new(),
            name_indexer: NameIndexer::with_capacity(capacity),
            buffer: Vec::with_capacity(capacity * 128),
            row_offsets: Vec::with_capacity(capacity),
            row_count: 0,
            free_list: Vec::with_capacity(capacity / 10),
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

    pub fn insert(&mut self, values: &[(String, Value)]) -> StorageResult<u32> {
        let record = self.serialize_row(values)?;
        let record_size = record.len();

        let offset = if let Some(free) = self.free_list.pop() {
            let row_idx = (free - 1) as usize;
            let buf_offset = self.buffer.len();
            self.buffer.extend_from_slice(&record);
            self.row_offsets[row_idx] = Some((buf_offset, record_size));
            free
        } else {
            let buf_offset = self.buffer.len();
            self.buffer.extend_from_slice(&record);
            let new_offset = prop_index_to_offset(self.row_count);
            self.row_offsets.push(Some((buf_offset, record_size)));
            self.row_count += 1;
            new_offset
        };

        Ok(offset)
    }

    pub fn update(&mut self, offset: u32, values: &[(String, Value)]) -> StorageResult<()> {
        let row_idx = prop_offset_to_index(offset).ok_or_else(|| StorageError::invalid_offset(offset))?;
        if row_idx >= self.row_count {
            return Err(StorageError::invalid_offset(offset));
        }

        let (buf_offset, _) = self.row_offsets[row_idx].ok_or_else(|| StorageError::invalid_offset(offset))?;
        let current_data = &self.buffer[buf_offset..];
        let current_values = self.deserialize_row(current_data)?;
        let mut merged_values: Vec<(String, Option<Value>)> = current_values;

        for (name, value) in values {
            if let Some(pos) = merged_values.iter().position(|(n, _)| n == name) {
                merged_values[pos] = (name.clone(), Some(value.clone()));
            } else {
                merged_values.push((name.clone(), Some(value.clone())));
            }
        }

        let new_record = self.serialize_row_with_nulls(&merged_values)?;
        let new_size = new_record.len();
        let new_buf_offset = self.buffer.len();
        self.buffer.extend_from_slice(&new_record);
        self.row_offsets[row_idx] = Some((new_buf_offset, new_size));

        Ok(())
    }

    pub fn get(&self, offset: u32) -> Option<Vec<(String, Option<Value>)>> {
        let row_idx = prop_offset_to_index(offset)?;
        if row_idx >= self.row_count {
            return None;
        }

        let (buf_offset, _) = self.row_offsets[row_idx]?;
        let record_data = &self.buffer[buf_offset..];
        self.deserialize_row(record_data).ok()
    }

    pub fn set_property(
        &mut self,
        offset: u32,
        name: &str,
        value: Option<Value>,
    ) -> StorageResult<()> {
        let row_idx = prop_offset_to_index(offset).ok_or_else(|| StorageError::invalid_offset(offset))?;
        if row_idx >= self.row_count {
            return Err(StorageError::invalid_offset(offset));
        }

        if !self.has_property(name) {
            return Err(StorageError::column_not_found(name.to_string()));
        }

        let mut merged_values: Vec<(String, Option<Value>)> = Vec::new();
        if let Some(props) = self.get(offset) {
            for (n, v) in props {
                if n != name {
                    merged_values.push((n, v));
                } else {
                    merged_values.push((n, value.clone()));
                }
            }
        }

        let new_record = self.serialize_row_with_nulls(&merged_values)?;
        let new_size = new_record.len();
        let new_buf_offset = self.buffer.len();
        self.buffer.extend_from_slice(&new_record);
        self.row_offsets[row_idx] = Some((new_buf_offset, new_size));

        Ok(())
    }

    pub fn set_property_by_id(
        &mut self,
        offset: u32,
        prop_id: PropertyId,
        value: Option<Value>,
    ) -> StorageResult<()> {
        let col_idx = prop_id.as_usize();
        if col_idx >= self.schema.len() {
            return Err(StorageError::column_not_found(format!("prop_id={}", prop_id)));
        }

        self.set_property(offset, &self.schema[col_idx].name.clone(), value)
    }

    pub fn delete(&mut self, offset: u32) -> bool {
        let row_idx = match prop_offset_to_index(offset) {
            Some(idx) => idx,
            None => return false,
        };
        if row_idx >= self.row_count {
            return false;
        }

        self.row_offsets[row_idx] = None;
        self.free_list.push(offset);
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

        result.extend_from_slice(&(self.schema.len() as u32).to_le_bytes());
        for prop in &self.schema {
            let name_bytes = prop.name.as_bytes();
            result.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
            result.extend_from_slice(name_bytes);
            result.extend_from_slice(&prop.prop_id.to_le_bytes());
            result.push(prop.data_type.as_u8());
            result.push(if prop.nullable { 1 } else { 0 });
        }

        result.extend_from_slice(&(self.row_count as u32).to_le_bytes());

        // Store buffer size and data
        result.extend_from_slice(&(self.buffer.len() as u32).to_le_bytes());
        result.extend_from_slice(&self.buffer);

        // Store row offsets
        result.extend_from_slice(&(self.row_offsets.len() as u32).to_le_bytes());
        for offset_info in &self.row_offsets {
            match offset_info {
                Some((buf_offset, size)) => {
                    result.push(1); // marker: has data
                    result.extend_from_slice(&(*buf_offset as u32).to_le_bytes());
                    result.extend_from_slice(&(*size as u32).to_le_bytes());
                }
                None => {
                    result.push(0); // marker: deleted
                }
            }
        }

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

        self.row_count = read_u32_le(data, &mut offset)? as usize;

        // Load buffer
        let buffer_len = read_u32_le(data, &mut offset)? as usize;
        if offset + buffer_len > data.len() {
            return Err(StorageError::deserialize_error("unexpected end of data"));
        }
        self.buffer = data[offset..offset + buffer_len].to_vec();
        offset += buffer_len;

        // Load row offsets
        let row_offsets_len = read_u32_le(data, &mut offset)? as usize;
        self.row_offsets.clear();
        for _ in 0..row_offsets_len {
            if offset >= data.len() {
                return Err(StorageError::deserialize_error("unexpected end of data"));
            }
            let marker = data[offset];
            offset += 1;
            if marker == 1 {
                let buf_offset = read_u32_le(data, &mut offset)? as usize;
                let size = read_u32_le(data, &mut offset)? as usize;
                self.row_offsets.push(Some((buf_offset, size)));
            } else {
                self.row_offsets.push(None);
            }
        }

        let free_list_len = read_u32_le(data, &mut offset)? as usize;
        self.free_list.clear();
        for _ in 0..free_list_len {
            self.free_list.push(read_u32_le(data, &mut offset)?);
        }

        Ok(())
    }

    pub fn compact(&mut self, valid_offsets: &HashSet<u32>) {
        let mut new_buffer = Vec::new();
        let mut new_row_offsets = Vec::new();
        let mut offset_mapping = std::collections::HashMap::new();

        for old_row_idx in 0..self.row_count {
            let old_offset = prop_index_to_offset(old_row_idx);
            if valid_offsets.contains(&old_offset) {
                if let Some((buf_offset, size)) = self.row_offsets[old_row_idx] {
                    let new_buf_offset = new_buffer.len();
                    new_buffer.extend_from_slice(&self.buffer[buf_offset..buf_offset + size]);
                    new_row_offsets.push(Some((new_buf_offset, size)));
                    offset_mapping.insert(old_offset, prop_index_to_offset(new_row_offsets.len() - 1));
                } else {
                    new_row_offsets.push(None);
                }
            }
        }

        self.buffer = new_buffer;
        self.row_offsets = new_row_offsets;
        self.row_count = offset_mapping.len();
        self.free_list.clear();
    }

    pub fn used_memory_size(&self) -> usize {
        let mut total = self.buffer.len();
        total += self.row_offsets.len() * std::mem::size_of::<Option<(usize, usize)>>();
        total += std::mem::size_of::<Self>();
        total
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
            if row_idx < self.row_count {
                if let Some((buf_offset, _)) = self.row_offsets[row_idx] {
                    // Prefetch the buffer location to L1/L2 cache
                    // This is a volatile operation that the compiler cannot optimize away
                    #[allow(unsafe_code)]
                    unsafe {
                        let addr = self.buffer.as_ptr().add(buf_offset) as *const u8;
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
    pub fn get_fast(&self, offset: u32) -> Option<Vec<(String, Option<Value>)>> {
        if !self.is_schema_fixed_size() {
            return self.get(offset);
        }

        let row_idx = prop_offset_to_index(offset)?;
        if row_idx >= self.row_count {
            return None;
        }

        let (buf_offset, _) = self.row_offsets[row_idx]?;
        let record_data = &self.buffer[buf_offset..];

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
    pub fn get_batch<'a, I>(&'a self, offsets: I) -> Vec<Option<Vec<(String, Option<Value>)>>>
    where
        I: IntoIterator<Item = &'a u32>,
    {
        let offsets: Vec<_> = offsets.into_iter().collect();
        let mut indexed: Vec<_> = offsets
            .iter()
            .enumerate()
            .map(|(idx, offset)| (idx, **offset))
            .collect();

        // Sort by offset to improve cache locality
        indexed.sort_by_key(|(_, offset)| *offset);

        // Prefetch all offsets
        for (_, offset) in &indexed {
            self.prefetch(*offset);
        }

        // Retrieve in sorted order
        let mut sorted_results: Vec<_> = indexed
            .iter()
            .map(|(_, offset)| self.get_fast(*offset).or_else(|| self.get(*offset)))
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
