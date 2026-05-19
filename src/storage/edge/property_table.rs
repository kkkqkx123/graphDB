//! Property Table for Edges
//!
//! Stores edge properties using a columnar internal layout with row-oriented API.
//! This design reuses the Column infrastructure from vertex::column_store while
//! presenting a row-level access pattern that edges require.

use std::collections::HashMap;
use std::collections::HashSet;

use crate::core::{DataType, DateValue, NullType, StorageError, StorageResult, Value};
use crate::storage::storage_types::PropertyId;
use crate::storage::utils::encoding::{read_header, section, write_header};
use crate::storage::utils::{read_u32_le, read_u64_le, NameIndexer};
use crate::storage::vertex::column_store::Column;
use crate::storage::vertex::encoding::{CompressionConfig, CompressionSelector, EncodingType};

/// Check that at least `needed` bytes remain in data starting at offset
fn check_remaining(data: &[u8], offset: usize, needed: usize) -> StorageResult<()> {
    let end = offset + needed;
    if end > data.len() {
        Err(StorageError::deserialize_error(format!(
            "unexpected end of data: needed {} bytes, have {} at offset {}",
            needed,
            data.len(),
            offset
        )))
    } else {
        Ok(())
    }
}

/// Sentinel value meaning "no properties"
pub const PROP_OFFSET_NONE: u32 = 0;

/// Threshold for overflow storage (values larger than this go to overflow store)
pub const OVERFLOW_THRESHOLD: usize = 256;

/// Default row group size (similar to DuckDB's 2048 rows)
pub const DEFAULT_ROW_GROUP_SIZE: usize = 2048;

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
    pub encoding: Option<EncodingType>,
}

impl PropertySchema {
    pub fn new(name: String, prop_id: i32, data_type: DataType) -> Self {
        Self {
            name,
            prop_id,
            data_type,
            nullable: false,
            default_value: None,
            encoding: None,
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

    pub fn with_encoding(mut self, encoding: EncodingType) -> Self {
        self.encoding = Some(encoding);
        self
    }
}

#[derive(Debug, Clone)]
pub struct OverflowPointer {
    pub overflow_id: u64,
    pub original_size: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct OverflowKey {
    col_idx: usize,
    row_idx: usize,
}

#[derive(Debug)]
pub struct OverflowStore {
    /// Continuous memory pool storing all overflow data
    data_pool: Vec<u8>,
    /// Index: overflow_id -> (offset_in_pool, size)
    index: HashMap<u64, (u64, u32)>,
    /// Location index: (col_idx, row_idx) -> overflow_id
    location_index: HashMap<OverflowKey, u64>,
    next_id: u64,
    /// Free list for reusing space from deleted values: (offset, size)
    free_list: Vec<(u64, u32)>,
    /// Number of active entries
    entry_count: usize,
}

impl OverflowStore {
    pub fn new() -> Self {
        Self {
            data_pool: Vec::new(),
            index: HashMap::new(),
            location_index: HashMap::new(),
            next_id: 0,
            free_list: Vec::new(),
            entry_count: 0,
        }
    }

    pub fn with_pool_capacity(pool_size: usize) -> Self {
        Self {
            data_pool: Vec::with_capacity(pool_size),
            index: HashMap::new(),
            location_index: HashMap::new(),
            next_id: 0,
            free_list: Vec::new(),
            entry_count: 0,
        }
    }

    pub fn store(&mut self, col_idx: usize, row_idx: usize, value: &Value) -> OverflowPointer {
        let bytes = value.to_bytes();
        let size = bytes.len() as u32;
        let id = self.next_id;
        self.next_id += 1;

        let (offset, _allocated_size) = self.allocate_space(size);

        let end = offset as usize + size as usize;
        if end > self.data_pool.len() {
            self.data_pool.resize(end, 0);
        }
        self.data_pool[offset as usize..end].copy_from_slice(&bytes);

        self.index.insert(id, (offset, size));
        self.location_index.insert(OverflowKey { col_idx, row_idx }, id);
        self.entry_count += 1;

        OverflowPointer {
            overflow_id: id,
            original_size: bytes.len() as u32,
        }
    }

    /// Best-fit allocation: find smallest free slot that fits the needed size.
    /// If no free slot fits, append to the end of the pool.
    fn allocate_space(&mut self, needed_size: u32) -> (u64, u32) {
        let mut best_idx = None;
        let mut best_size = u32::MAX;

        for (i, &(_offset, size)) in self.free_list.iter().enumerate() {
            if size >= needed_size && size < best_size {
                best_idx = Some(i);
                best_size = size;
            }
        }

        if let Some(idx) = best_idx {
            let (offset, size) = self.free_list.swap_remove(idx);
            if size > needed_size {
                self.free_list.push((offset + needed_size as u64, size - needed_size));
            }
            (offset, needed_size)
        } else {
            (self.data_pool.len() as u64, needed_size)
        }
    }

    pub fn retrieve(&self, col_idx: usize, row_idx: usize) -> Option<Value> {
        let key = OverflowKey { col_idx, row_idx };
        let overflow_id = self.location_index.get(&key)?;
        let &(offset, size) = self.index.get(overflow_id)?;

        let start = offset as usize;
        let end = start + size as usize;
        if end > self.data_pool.len() {
            return None;
        }

        Value::from_bytes(&self.data_pool[start..end]).map(|(v, _)| v)
    }

    pub fn remove(&mut self, col_idx: usize, row_idx: usize) {
        let key = OverflowKey { col_idx, row_idx };
        if let Some(overflow_id) = self.location_index.remove(&key) {
            if let Some((offset, size)) = self.index.remove(&overflow_id) {
                self.add_to_free_list(offset, size);
                self.entry_count -= 1;
            }
        }
    }

    /// Add a freed block to the free list, coalescing with adjacent blocks.
    fn add_to_free_list(&mut self, offset: u64, size: u32) {
        let mut merged_offset = offset;
        let mut merged_size = size;

        self.free_list.retain(|&(free_offset, free_size)| {
            let free_end = free_offset + free_size as u64;
            let merged_end = merged_offset + merged_size as u64;

            if free_end == merged_offset {
                merged_offset = free_offset;
                merged_size += free_size;
                false
            } else if merged_end == free_offset {
                merged_size += free_size;
                false
            } else {
                true
            }
        });

        self.free_list.push((merged_offset, merged_size));
    }

    pub fn clear(&mut self) {
        self.data_pool.clear();
        self.index.clear();
        self.location_index.clear();
        self.next_id = 0;
        self.free_list.clear();
        self.entry_count = 0;
    }

    pub fn entry_count(&self) -> usize {
        self.entry_count
    }

    pub fn memory_size(&self) -> usize {
        let mut total = std::mem::size_of::<Self>();
        total += self.data_pool.capacity();
        total += self.index.capacity() * (std::mem::size_of::<u64>() + std::mem::size_of::<(u64, u32)>());
        total += self.location_index.capacity() * (std::mem::size_of::<OverflowKey>() + std::mem::size_of::<u64>());
        total += self.free_list.capacity() * std::mem::size_of::<(u64, u32)>();
        total
    }

    pub fn dump(&self) -> Vec<u8> {
        let mut result = Vec::new();

        // Header: magic + version + section_id
        write_header(&mut result, section::OVERFLOW_STORE);

        // Placeholder for CRC32 checksum (written at the end)
        let checksum_pos = result.len();
        result.extend_from_slice(&[0u8; 4]);

        // --- payload starts here ---

        // data_pool
        result.extend_from_slice(&(self.data_pool.len() as u64).to_le_bytes());
        result.extend_from_slice(&self.data_pool);

        // index
        result.extend_from_slice(&(self.index.len() as u64).to_le_bytes());
        let mut sorted_ids: Vec<&u64> = self.index.keys().collect();
        sorted_ids.sort();
        for id in sorted_ids {
            let (offset, size) = self.index[id];
            result.extend_from_slice(&id.to_le_bytes());
            result.extend_from_slice(&offset.to_le_bytes());
            result.extend_from_slice(&size.to_le_bytes());
        }

        // location_index
        result.extend_from_slice(&(self.location_index.len() as u64).to_le_bytes());
        let mut sorted_keys: Vec<&OverflowKey> = self.location_index.keys().collect();
        sorted_keys.sort_by(|a, b| a.col_idx.cmp(&b.col_idx).then(a.row_idx.cmp(&b.row_idx)));
        for key in sorted_keys {
            let overflow_id = self.location_index[key];
            result.extend_from_slice(&(key.col_idx as u32).to_le_bytes());
            result.extend_from_slice(&(key.row_idx as u32).to_le_bytes());
            result.extend_from_slice(&overflow_id.to_le_bytes());
        }

        // free_list
        result.extend_from_slice(&(self.free_list.len() as u64).to_le_bytes());
        for &(offset, size) in &self.free_list {
            result.extend_from_slice(&offset.to_le_bytes());
            result.extend_from_slice(&size.to_le_bytes());
        }

        result.extend_from_slice(&self.next_id.to_le_bytes());

        // --- payload ends here ---

        // Compute and write CRC32 checksum over the payload
        let checksum = crc32fast::hash(&result[checksum_pos + 4..]);
        result[checksum_pos..checksum_pos + 4].copy_from_slice(&checksum.to_le_bytes());

        result
    }

    pub fn load(&mut self, data: &[u8]) -> StorageResult<()> {
        if data.is_empty() {
            return Ok(());
        }

        // Validate header: magic + version + section_id
        let mut cursor = data;
        let (_version, section_id) = read_header(&mut cursor)?;
        if section_id != section::OVERFLOW_STORE {
            return Err(StorageError::deserialize_error(format!(
                "invalid section_id for OverflowStore: expected 0x{:04X}, got 0x{:04X}",
                section::OVERFLOW_STORE, section_id
            )));
        }

        // Read and verify CRC32 checksum
        if cursor.len() < 4 {
            return Err(StorageError::deserialize_error(
                "OverflowStore data too short for checksum",
            ));
        }
        let stored_checksum = u32::from_le_bytes(cursor[..4].try_into().map_err(|_| {
            StorageError::deserialize_error("failed to read OverflowStore checksum")
        })?);
        let payload = &cursor[4..];
        let computed_checksum = crc32fast::hash(payload);
        if stored_checksum != computed_checksum {
            return Err(StorageError::deserialize_error(format!(
                "OverflowStore checksum mismatch: stored {:#x}, computed {:#x}",
                stored_checksum, computed_checksum
            )));
        }

        // Shadow `data` with the payload slice so existing code works unchanged
        let data = payload;
        let mut offset = 0usize;

        // data_pool
        let pool_len = read_u64_le(data, &mut offset)? as usize;
        check_remaining(data, offset, pool_len)?;
        self.data_pool = data[offset..offset + pool_len].to_vec();
        offset += pool_len;

        // index
        let index_len = read_u64_le(data, &mut offset)? as usize;
        self.index.clear();
        for _ in 0..index_len {
            let id = read_u64_le(data, &mut offset)?;
            let pool_offset = read_u64_le(data, &mut offset)?;
            let size = read_u32_le(data, &mut offset)?;
            self.index.insert(id, (pool_offset, size));
        }

        // location_index
        let loc_len = read_u64_le(data, &mut offset)? as usize;
        self.location_index.clear();
        for _ in 0..loc_len {
            let col_idx = read_u32_le(data, &mut offset)? as usize;
            let row_idx = read_u32_le(data, &mut offset)? as usize;
            let overflow_id = read_u64_le(data, &mut offset)?;
            self.location_index.insert(OverflowKey { col_idx, row_idx }, overflow_id);
        }

        // free_list
        let free_len = read_u64_le(data, &mut offset)? as usize;
        self.free_list.clear();
        for _ in 0..free_len {
            let free_offset = read_u64_le(data, &mut offset)?;
            let free_size = read_u32_le(data, &mut offset)?;
            self.free_list.push((free_offset, free_size));
        }

        self.next_id = read_u64_le(data, &mut offset)?;
        self.entry_count = self.location_index.len();

        Ok(())
    }
}

impl Default for OverflowStore {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct RowGroup {
    pub start_row: usize,
    pub end_row: usize,
    pub column_indices: Vec<usize>,
}

impl RowGroup {
    pub fn new(start_row: usize, end_row: usize) -> Self {
        Self {
            start_row,
            end_row,
            column_indices: Vec::new(),
        }
    }

    pub fn row_count(&self) -> usize {
        self.end_row - self.start_row
    }

    pub fn contains_row(&self, row_idx: usize) -> bool {
        row_idx >= self.start_row && row_idx < self.end_row
    }
}

#[derive(Debug)]
pub struct PropertyTable {
    schema: Vec<PropertySchema>,
    name_indexer: NameIndexer,
    columns: Vec<Column>,
    row_count: usize,
    free_list: Vec<u32>,
    overflow_store: OverflowStore,
    row_groups: Vec<RowGroup>,
    row_group_size: usize,
}

impl PropertyTable {
    pub fn new() -> Self {
        Self {
            schema: Vec::new(),
            name_indexer: NameIndexer::new(),
            columns: Vec::new(),
            row_count: 0,
            free_list: Vec::new(),
            overflow_store: OverflowStore::new(),
            row_groups: Vec::new(),
            row_group_size: DEFAULT_ROW_GROUP_SIZE,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            schema: Vec::new(),
            name_indexer: NameIndexer::with_capacity(capacity),
            columns: Vec::new(),
            row_count: 0,
            free_list: Vec::with_capacity(capacity),
            overflow_store: OverflowStore::new(),
            row_groups: Vec::new(),
            row_group_size: DEFAULT_ROW_GROUP_SIZE,
        }
    }

    pub fn with_row_group_size(capacity: usize, row_group_size: usize) -> Self {
        Self {
            schema: Vec::new(),
            name_indexer: NameIndexer::with_capacity(capacity),
            columns: Vec::new(),
            row_count: 0,
            free_list: Vec::with_capacity(capacity),
            overflow_store: OverflowStore::new(),
            row_groups: Vec::new(),
            row_group_size,
        }
    }

    pub fn add_property(
        &mut self,
        name: String,
        data_type: DataType,
        nullable: bool,
    ) -> PropertyId {
        self.add_property_with_encoding(name, data_type, nullable, None)
    }

    pub fn add_property_with_encoding(
        &mut self,
        name: String,
        data_type: DataType,
        nullable: bool,
        encoding: Option<EncodingType>,
    ) -> PropertyId {
        let prop_id = PropertyId::new(self.schema.len() as u16);
        let schema =
            PropertySchema::new(name.clone(), prop_id.as_usize() as i32, data_type.clone())
                .nullable(nullable)
                .with_encoding(encoding.unwrap_or(EncodingType::None));
        self.name_indexer.register(name.clone());
        self.schema.push(schema);

        let mut column = Column::new(name, prop_id.as_usize() as i32, data_type, nullable);
        column.resize(self.row_count);
        self.columns.push(column);

        prop_id
    }

    pub fn rename_property(&mut self, old_name: &str, new_name: &str) -> StorageResult<()> {
        let col_idx = self
            .name_indexer
            .get_id(old_name)
            .ok_or_else(|| StorageError::column_not_found(old_name.to_string()))?
            .as_usize();

        if let Some(schema) = self.schema.get_mut(col_idx) {
            schema.name = new_name.to_string();
        }
        if let Some(column) = self.columns.get_mut(col_idx) {
            column.name = new_name.to_string();
        }

        self.name_indexer.remove(old_name);
        self.name_indexer.register(new_name.to_string());

        Ok(())
    }

    pub fn remove_property(&mut self, name: &str) -> StorageResult<()> {
        let col_idx = self
            .name_indexer
            .get_id(name)
            .ok_or_else(|| StorageError::column_not_found(name.to_string()))?
            .as_usize();

        self.schema.remove(col_idx);
        self.columns.remove(col_idx);
        self.name_indexer.remove(name);

        for (idx, schema) in self.schema.iter_mut().enumerate() {
            schema.prop_id = idx as i32;
        }

        Ok(())
    }

    pub fn apply_encoding(
        &mut self,
        prop_id: PropertyId,
        encoding: EncodingType,
    ) -> StorageResult<()> {
        let col_idx = prop_id.as_usize();
        if col_idx >= self.columns.len() {
            return Err(StorageError::column_not_found(format!(
                "prop_id={}",
                prop_id
            )));
        }

        let column = &mut self.columns[col_idx];
        match encoding {
            EncodingType::Dictionary => column.apply_dictionary_encoding()?,
            EncodingType::Rle => column.apply_rle_encoding()?,
            EncodingType::BitPacking => column.apply_bitpacking_encoding()?,
            EncodingType::Fsst => column.apply_fsst_encoding(1024)?,
            EncodingType::Alp => column.apply_alp_encoding()?,
            EncodingType::None => {}
        }

        if let Some(schema) = self.schema.get_mut(col_idx) {
            schema.encoding = Some(encoding);
        }

        Ok(())
    }

    pub fn auto_apply_encodings(
        &mut self,
        config: Option<CompressionConfig>,
    ) -> StorageResult<()> {
        let selector = match config {
            Some(c) => CompressionSelector::with_config(c),
            None => CompressionSelector::new(),
        };

        for (col_idx, col) in self.columns.iter_mut().enumerate() {
            if col.is_empty() {
                continue;
            }

            let stats = col.compute_stats();
            let encoding = selector.select(&stats);

            match encoding {
                EncodingType::Fsst => {
                    if col.data_type == DataType::String {
                        col.apply_fsst_encoding(1024)?;
                        if let Some(schema) = self.schema.get_mut(col_idx) {
                            schema.encoding = Some(EncodingType::Fsst);
                        }
                    }
                }
                EncodingType::Dictionary => {
                    col.apply_dictionary_encoding()?;
                    if let Some(schema) = self.schema.get_mut(col_idx) {
                        schema.encoding = Some(EncodingType::Dictionary);
                    }
                }
                EncodingType::Rle => {
                    col.apply_rle_encoding()?;
                    if let Some(schema) = self.schema.get_mut(col_idx) {
                        schema.encoding = Some(EncodingType::Rle);
                    }
                }
                EncodingType::BitPacking => {
                    col.apply_bitpacking_encoding()?;
                    if let Some(schema) = self.schema.get_mut(col_idx) {
                        schema.encoding = Some(EncodingType::BitPacking);
                    }
                }
                EncodingType::Alp => {
                    col.apply_alp_encoding()?;
                    if let Some(schema) = self.schema.get_mut(col_idx) {
                        schema.encoding = Some(EncodingType::Alp);
                    }
                }
                EncodingType::None => {}
            }
        }

        Ok(())
    }

    fn should_use_overflow(&self, value: &Value) -> bool {
        value.to_bytes().len() > OVERFLOW_THRESHOLD
    }

    pub fn insert(&mut self, values: &[(String, Value)]) -> StorageResult<u32> {
        let offset = if let Some(free) = self.free_list.pop() {
            // Clear all columns for this reused offset to prevent stale data
            self.clear_row(free);
            free
        } else {
            let new_offset = prop_index_to_offset(self.row_count);
            self.row_count += 1;
            for col in &mut self.columns {
                col.resize(self.row_count);
            }
            self.ensure_row_group();
            new_offset
        };

        self.update(offset, values)?;
        Ok(offset)
    }

    /// Clear all column values at the given offset for reuse.
    /// Nullable columns are set to None; non-nullable columns are reset to the zero value of their type.
    fn clear_row(&mut self, offset: u32) {
        let row_idx = match prop_offset_to_index(offset) {
            Some(idx) => idx,
            None => return,
        };
        if row_idx >= self.row_count {
            return;
        }

        for col_idx in 0..self.columns.len() {
            self.overflow_store.remove(col_idx, row_idx);
            let col = &self.columns[col_idx];
            if col.nullable {
                let _ = self.columns[col_idx].set(row_idx, None);
            } else {
                let zero = Self::zero_value_for_type(&col.data_type);
                let _ = self.columns[col_idx].set(row_idx, Some(&zero));
            }
        }
    }

    /// Return a zero value for the given DataType used to reset non-nullable columns.
    fn zero_value_for_type(data_type: &DataType) -> Value {
        match data_type {
            DataType::Bool => Value::Bool(false),
            DataType::SmallInt => Value::SmallInt(0),
            DataType::Int => Value::Int(0),
            DataType::BigInt => Value::BigInt(0),
            DataType::Float => Value::Float(0.0),
            DataType::Double => Value::Double(0.0),
            DataType::String => Value::String(String::new()),
            DataType::Date => Value::Date(DateValue {
                year: 0,
                month: 0,
                day: 0,
            }),
            _ => Value::Null(NullType::Null),
        }
    }

    fn ensure_row_group(&mut self) {
        if self.columns.is_empty() {
            return;
        }

        let current_row = self.row_count - 1;
        let group_index = current_row / self.row_group_size;
        let group_start = group_index * self.row_group_size;
        let group_end = ((group_index + 1) * self.row_group_size).min(self.row_count);

        match self.row_groups.last_mut() {
            Some(last_group) if last_group.start_row == group_start => {
                last_group.end_row = group_end;
            }
            _ => {
                self.row_groups.push(RowGroup::new(group_start, group_end));
            }
        }
    }

    pub fn update(&mut self, offset: u32, values: &[(String, Value)]) -> StorageResult<()> {
        let row_idx =
            prop_offset_to_index(offset).ok_or_else(|| StorageError::invalid_offset(offset))?;
        if row_idx >= self.row_count {
            return Err(StorageError::invalid_offset(offset));
        }

        for (name, value) in values {
            if let Some(col_idx) = self.name_indexer.get_id(name) {
                let col_idx = col_idx.as_usize();
                if col_idx < self.columns.len() {
                    if self.should_use_overflow(value) {
                        self.overflow_store.remove(col_idx, row_idx);
                        self.overflow_store.store(col_idx, row_idx, value);
                    } else {
                        self.overflow_store.remove(col_idx, row_idx);
                        self.columns[col_idx].set(row_idx, Some(value))?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn get(&self, offset: u32) -> Option<Vec<(String, Option<Value>)>> {
        let row_idx = prop_offset_to_index(offset)?;
        if row_idx >= self.row_count {
            return None;
        }

        Some(
            self.columns
                .iter()
                .enumerate()
                .map(|(col_idx, col)| {
                    let value = col.get(row_idx);
                    let resolved_value = if value.is_none() {
                        self.overflow_store.retrieve(col_idx, row_idx)
                    } else {
                        value
                    };
                    (col.name.clone(), resolved_value)
                })
                .collect(),
        )
    }

    pub fn get_property(&self, offset: u32, name: &str) -> Option<Value> {
        let col_idx = self.name_indexer.get_id(name)?;
        let row_idx = prop_offset_to_index(offset)?;
        if row_idx >= self.row_count {
            return None;
        }

        let col_idx = col_idx.as_usize();
        if col_idx < self.columns.len() {
            self.columns[col_idx]
                .get(row_idx)
                .or_else(|| self.overflow_store.retrieve(col_idx, row_idx))
        } else {
            None
        }
    }

    pub fn get_property_by_id(&self, offset: u32, prop_id: PropertyId) -> Option<Value> {
        let col_idx = prop_id.as_usize();
        let row_idx = prop_offset_to_index(offset)?;
        if row_idx >= self.row_count {
            return None;
        }

        if col_idx < self.columns.len() {
            self.columns[col_idx]
                .get(row_idx)
                .or_else(|| self.overflow_store.retrieve(col_idx, row_idx))
        } else {
            None
        }
    }

    pub fn set_property(
        &mut self,
        offset: u32,
        name: &str,
        value: Option<Value>,
    ) -> StorageResult<()> {
        let col_idx = self
            .name_indexer
            .get_id(name)
            .ok_or_else(|| StorageError::column_not_found(name.to_string()))?;
        let col_idx = col_idx.as_usize();

        let row_idx =
            prop_offset_to_index(offset).ok_or_else(|| StorageError::invalid_offset(offset))?;
        if row_idx >= self.row_count {
            return Err(StorageError::invalid_offset(offset));
        }

        if col_idx < self.columns.len() {
            if let Some(ref v) = value {
                if self.should_use_overflow(v) {
                    self.overflow_store.remove(col_idx, row_idx);
                    self.overflow_store.store(col_idx, row_idx, v);
                    self.columns[col_idx].set(row_idx, None)?;
                } else {
                    self.overflow_store.remove(col_idx, row_idx);
                    self.columns[col_idx].set(row_idx, Some(v))?;
                }
            } else {
                self.overflow_store.remove(col_idx, row_idx);
                self.columns[col_idx].set(row_idx, None)?;
            }
        }

        Ok(())
    }

    pub fn set_property_by_id(
        &mut self,
        offset: u32,
        prop_id: PropertyId,
        value: Option<Value>,
    ) -> StorageResult<()> {
        let col_idx = prop_id.as_usize();
        let row_idx =
            prop_offset_to_index(offset).ok_or_else(|| StorageError::invalid_offset(offset))?;
        if row_idx >= self.row_count {
            return Err(StorageError::invalid_offset(offset));
        }

        if col_idx >= self.columns.len() {
            return Err(StorageError::column_not_found(format!(
                "prop_id={}",
                prop_id
            )));
        }

        if let Some(ref v) = value {
            if self.should_use_overflow(v) {
                self.overflow_store.remove(col_idx, row_idx);
                self.overflow_store.store(col_idx, row_idx, v);
                self.columns[col_idx].set(row_idx, None)?;
            } else {
                self.overflow_store.remove(col_idx, row_idx);
                self.columns[col_idx].set(row_idx, Some(v))?;
            }
        } else {
            self.overflow_store.remove(col_idx, row_idx);
            self.columns[col_idx].set(row_idx, None)?;
        }

        Ok(())
    }

    pub fn delete(&mut self, offset: u32) -> bool {
        let row_idx = match prop_offset_to_index(offset) {
            Some(idx) => idx,
            None => return false,
        };
        if row_idx >= self.row_count {
            return false;
        }

        for col_idx in 0..self.columns.len() {
            self.overflow_store.remove(col_idx, row_idx);
            let _ = self.columns[col_idx].set(row_idx, None);
        }
        self.free_list.push(offset);
        true
    }

    pub fn row_count(&self) -> usize {
        self.row_count - self.free_list.len()
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
        for col in &mut self.columns {
            col.clear();
        }
        self.row_count = 0;
        self.free_list.clear();
        self.overflow_store.clear();
        self.row_groups.clear();
    }

    pub fn has_property(&self, name: &str) -> bool {
        self.name_indexer.contains(name)
    }

    pub fn get_schema(&self, name: &str) -> Option<&PropertySchema> {
        self.name_indexer
            .get_id(name)
            .map(|id| id.as_usize())
            .and_then(|idx| self.schema.get(idx))
    }

    pub fn get_schema_by_id(&self, prop_id: PropertyId) -> Option<&PropertySchema> {
        self.schema.get(prop_id.as_usize())
    }

    pub fn get_property_type(&self, prop_id: PropertyId) -> Option<DataType> {
        self.schema
            .get(prop_id.as_usize())
            .map(|s| s.data_type.clone())
    }

    pub fn name_indexer(&self) -> &NameIndexer {
        &self.name_indexer
    }

    pub fn row_group_count(&self) -> usize {
        self.row_groups.len()
    }

    pub fn get_row_group(&self, group_idx: usize) -> Option<&RowGroup> {
        self.row_groups.get(group_idx)
    }

    pub fn get_row_group_for_row(&self, row_idx: usize) -> Option<&RowGroup> {
        self.row_groups.iter().find(|rg| rg.contains_row(row_idx))
    }

    pub fn dump(&self) -> Vec<u8> {
        let mut result = Vec::new();

        // Header: magic + version + section_id
        write_header(&mut result, section::PROPERTY_TABLE);

        // Placeholder for CRC32 checksum (written at the end)
        let checksum_pos = result.len();
        result.extend_from_slice(&[0u8; 4]);

        // --- payload starts here ---

        result.extend_from_slice(&(self.schema.len() as u32).to_le_bytes());
        for prop in &self.schema {
            let name_bytes = prop.name.as_bytes();
            result.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
            result.extend_from_slice(name_bytes);
            result.extend_from_slice(&prop.prop_id.to_le_bytes());
            result.push(prop.data_type.as_u8());
            result.push(if prop.nullable { 1 } else { 0 });
            result.push(prop.encoding.unwrap_or(EncodingType::None) as u8);
        }

        result.extend_from_slice(&(self.row_count as u32).to_le_bytes());

        // RowGroup-level sharding: write column data per RowGroup
        let column_count = self.columns.len() as u32;
        result.extend_from_slice(&column_count.to_le_bytes());
        result.extend_from_slice(&(self.row_groups.len() as u32).to_le_bytes());

        // Write RowGroup headers first
        for rg in &self.row_groups {
            result.extend_from_slice(&(rg.start_row as u32).to_le_bytes());
            result.extend_from_slice(&(rg.end_row as u32).to_le_bytes());
        }

        // Write column data for each RowGroup
        for rg in &self.row_groups {
            for col in &self.columns {
                let (data, offsets, bitmap) = col.get_flush_data_range(rg.start_row, rg.end_row);

                result.extend_from_slice(&(data.len() as u32).to_le_bytes());
                result.extend_from_slice(&data);

                let offsets_count = offsets.len() as u32;
                result.extend_from_slice(&offsets_count.to_le_bytes());
                for &off in &offsets {
                    result.extend_from_slice(&off.to_le_bytes());
                }

                if let Some(bitmap) = bitmap {
                    result.push(1);
                    let bitmap_bit_len = bitmap.len() as u32;
                    let bitmap_bytes = bitmap.as_raw_slice();
                    result.extend_from_slice(&bitmap_bit_len.to_le_bytes());
                    result.extend_from_slice(&(bitmap_bytes.len() as u32).to_le_bytes());
                    result.extend_from_slice(bitmap_bytes);
                } else {
                    result.push(0);
                }
            }
        }

        result.extend_from_slice(&(self.free_list.len() as u32).to_le_bytes());
        for off in &self.free_list {
            result.extend_from_slice(&off.to_le_bytes());
        }

        let overflow_data = self.overflow_store.dump();
        result.extend_from_slice(&(overflow_data.len() as u64).to_le_bytes());
        result.extend_from_slice(&overflow_data);

        result.extend_from_slice(&(self.row_groups.len() as u32).to_le_bytes());
        for rg in &self.row_groups {
            result.extend_from_slice(&(rg.start_row as u32).to_le_bytes());
            result.extend_from_slice(&(rg.end_row as u32).to_le_bytes());
        }

        // --- payload ends here ---

        // Compute and write CRC32 checksum over the payload
        let checksum = crc32fast::hash(&result[checksum_pos + 4..]);
        result[checksum_pos..checksum_pos + 4].copy_from_slice(&checksum.to_le_bytes());

        result
    }

    pub fn load(&mut self, data: &[u8]) -> StorageResult<()> {
        if data.is_empty() {
            return Ok(());
        }

        // Validate header: magic + version + section_id
        let mut cursor = data;
        let (_version, section_id) = read_header(&mut cursor)?;
        if section_id != section::PROPERTY_TABLE {
            return Err(StorageError::deserialize_error(format!(
                "invalid section_id for PropertyTable: expected 0x{:04X}, got 0x{:04X}",
                section::PROPERTY_TABLE, section_id
            )));
        }

        // Read and verify CRC32 checksum
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

        // Shadow `data` with the payload slice so existing code works unchanged
        let data = payload;
        let mut offset = 0usize;

        let schema_len = read_u32_le(data, &mut offset)? as usize;

        self.schema.clear();
        self.name_indexer.clear();
        self.columns.clear();

        for _ in 0..schema_len {
            let name_len = read_u32_le(data, &mut offset)? as usize;

            check_remaining(data, offset, name_len)?;
            let name = String::from_utf8_lossy(&data[offset..offset + name_len]).to_string();
            offset += name_len;

            let prop_id_bytes: [u8; 4] = data[offset..offset + 4].try_into().map_err(|_| {
                StorageError::deserialize_error("failed to read prop_id")
            })?;
            let prop_id = i32::from_le_bytes(prop_id_bytes);
            offset += 4;
            let data_type = DataType::from_u8(data[offset]);
            offset += 1;
            let nullable = data[offset] == 1;
            offset += 1;
            let encoding_byte = data[offset];
            offset += 1;

            let encoding = match encoding_byte {
                1 => Some(EncodingType::Dictionary),
                2 => Some(EncodingType::Rle),
                3 => Some(EncodingType::BitPacking),
                4 => Some(EncodingType::Fsst),
                5 => Some(EncodingType::Alp),
                _ => None,
            };

            let prop_schema = PropertySchema::new(name.clone(), prop_id, data_type.clone())
                .nullable(nullable)
                .with_encoding(encoding.unwrap_or(EncodingType::None));
            self.name_indexer.register(name.clone());
            self.schema.push(prop_schema);

            let column = Column::new(name, prop_id, data_type, nullable);
            self.columns.push(column);
        }

        let rows_len = read_u32_le(data, &mut offset)? as usize;

        self.row_count = 0;

        for col in &mut self.columns {
            col.clear();
        }

        // RowGroup-level format: read RowGroup count, headers, then data per RowGroup
        let column_count = read_u32_le(data, &mut offset)? as usize;
        let row_group_count = read_u32_le(data, &mut offset)? as usize;

        // Read RowGroup headers
        let mut loaded_row_groups: Vec<(usize, usize)> = Vec::with_capacity(row_group_count);
        for _ in 0..row_group_count {
            let start_row = read_u32_le(data, &mut offset)? as usize;
            let end_row = read_u32_le(data, &mut offset)? as usize;
            loaded_row_groups.push((start_row, end_row));
        }

        // Per-column accumulators for merging RowGroup data
        struct ColumnAccum {
            data: Vec<u8>,
            offsets: Vec<u64>,
            bitmap_bytes: Vec<u8>,
            bitmap_bit_len: usize,
        }

        let mut col_accums: Vec<ColumnAccum> = (0..column_count.min(self.columns.len()))
            .map(|_| ColumnAccum {
                data: Vec::new(),
                offsets: Vec::new(),
                bitmap_bytes: Vec::new(),
                bitmap_bit_len: 0,
            })
            .collect();

        // Read data for each RowGroup
        for _ in 0..row_group_count {
            for col_idx in 0..column_count.min(self.columns.len()) {
                let data_len = read_u32_le(data, &mut offset)? as usize;
                check_remaining(data, offset, data_len)?;
                let col_data = &data[offset..offset + data_len];
                offset += data_len;

                let offsets_count = read_u32_le(data, &mut offset)? as usize;
                let mut rg_offsets = Vec::with_capacity(offsets_count);
                for _ in 0..offsets_count {
                    rg_offsets.push(read_u64_le(data, &mut offset)?);
                }

                let has_bitmap = data[offset] == 1;
                offset += 1;

                let (rg_bitmap_bytes, rg_bitmap_bit_len) = if has_bitmap {
                    let bitmap_bit_len = read_u32_le(data, &mut offset)? as usize;
                    let bitmap_bytes_len = read_u32_le(data, &mut offset)? as usize;
                    check_remaining(data, offset, bitmap_bytes_len)?;
                    let bytes = data[offset..offset + bitmap_bytes_len].to_vec();
                    offset += bitmap_bytes_len;
                    (bytes, bitmap_bit_len)
                } else {
                    (Vec::new(), 0)
                };

                if col_idx < col_accums.len() {
                    let accum = &mut col_accums[col_idx];
                    // Adjust offsets relative to previous accumulated data length
                    let base = accum.data.len() as u64;
                    for off in rg_offsets {
                        if off != u64::MAX {
                            accum.offsets.push(off + base);
                        } else {
                            accum.offsets.push(u64::MAX);
                        }
                    }
                    accum.data.extend_from_slice(col_data);
                    accum.bitmap_bytes.extend_from_slice(&rg_bitmap_bytes);
                    accum.bitmap_bit_len += rg_bitmap_bit_len;
                }
            }
        }

        // Load merged column data into columns
        for (col_idx, accum) in col_accums.iter().enumerate() {
            let bitmap_raw = if accum.bitmap_bit_len > 0 {
                Some(accum.bitmap_bytes.clone())
            } else {
                None
            };
            self.columns[col_idx].load_data_from_raw(
                accum.data.clone(),
                accum.offsets.clone(),
                bitmap_raw,
                accum.bitmap_bit_len,
            );
        }

        self.row_count = rows_len;

        let free_list_len = read_u32_le(data, &mut offset)? as usize;

        self.free_list.clear();
        for _ in 0..free_list_len {
            self.free_list.push(read_u32_le(data, &mut offset)?);
        }

        let overflow_len = read_u64_le(data, &mut offset)? as usize;
        if overflow_len > 0 {
            check_remaining(data, offset, overflow_len)?;
            self.overflow_store
                .load(&data[offset..offset + overflow_len])?;
        }

        // Build row_groups from loaded headers
        self.row_groups.clear();
        for (start_row, end_row) in &loaded_row_groups {
            self.row_groups.push(RowGroup::new(*start_row, *end_row));
        }

        Ok(())
    }

    pub fn compact(&mut self, valid_offsets: &HashSet<u32>) {
        let mut new_columns: Vec<Column> = Vec::with_capacity(self.columns.len());
        let mut new_row_count = 0;

        for col in &self.columns {
            let new_col = Column::new(
                col.name.clone(),
                col.col_id,
                col.data_type.clone(),
                col.nullable,
            );
            new_columns.push(new_col);
        }

        for old_row_idx in 0..self.row_count {
            let old_offset = prop_index_to_offset(old_row_idx);
            if valid_offsets.contains(&old_offset) {
                for (col_idx, col) in self.columns.iter().enumerate() {
                    let value = col
                        .get(old_row_idx)
                        .or_else(|| self.overflow_store.retrieve(col_idx, old_row_idx));
                    let _ = new_columns[col_idx].set(new_row_count, value.as_ref());
                }
                new_row_count += 1;
            }
        }

        self.columns = new_columns;
        self.row_count = new_row_count;
        self.free_list.clear();
        self.overflow_store.clear();

        self.row_groups.clear();
        let mut group_start = 0;
        while group_start < self.row_count {
            let group_end = (group_start + self.row_group_size).min(self.row_count);
            self.row_groups.push(RowGroup::new(group_start, group_end));
            group_start = group_end;
        }
    }

    pub fn compact_row_group(&mut self, group_idx: usize) -> StorageResult<()> {
        if group_idx >= self.row_groups.len() {
            return Err(StorageError::invalid_input(format!(
                "Row group index {} out of range",
                group_idx
            )));
        }

        let group = &self.row_groups[group_idx];
        let mut valid_offsets = HashSet::new();

        for row_idx in group.start_row..group.end_row {
            valid_offsets.insert(prop_index_to_offset(row_idx));
        }

        let mut new_columns: Vec<Column> = Vec::with_capacity(self.columns.len());
        let mut new_row_count = 0;

        for col in &self.columns {
            let new_col = Column::new(
                col.name.clone(),
                col.col_id,
                col.data_type.clone(),
                col.nullable,
            );
            new_columns.push(new_col);
        }

        for old_row_idx in group.start_row..group.end_row {
            let old_offset = prop_index_to_offset(old_row_idx);
            if valid_offsets.contains(&old_offset) {
                for (col_idx, col) in self.columns.iter().enumerate() {
                    let value = col
                        .get(old_row_idx)
                        .or_else(|| self.overflow_store.retrieve(col_idx, old_row_idx));
                    let _ = new_columns[col_idx].set(new_row_count, value.as_ref());
                }
                new_row_count += 1;
            }
        }

        for (col_idx, new_col) in new_columns.into_iter().enumerate() {
            self.columns[col_idx] = new_col;
        }

        self.row_groups[group_idx].end_row = self.row_groups[group_idx].start_row + new_row_count;

        Ok(())
    }

    pub fn memory_size(&self) -> usize {
        let mut total = 0;

        total += self.schema.len() * std::mem::size_of::<PropertySchema>();
        total += self.columns.len() * std::mem::size_of::<Column>();
        total += self.free_list.len() * std::mem::size_of::<u32>();
        total += std::mem::size_of::<Self>();
        total += self.overflow_store.memory_size();
        total += self.row_groups.len() * std::mem::size_of::<RowGroup>();
        total += self.name_indexer.memory_size();

        for col in &self.columns {
            total += col.used_memory_size();
        }

        total
    }

    pub fn used_memory_size(&self) -> usize {
        let mut total = 0;
        for col in &self.columns {
            total += col.used_memory_size();
        }
        total += self.overflow_store.memory_size();
        total + std::mem::size_of::<Self>()
    }

    pub fn overflow_store(&self) -> &OverflowStore {
        &self.overflow_store
    }

    pub fn overflow_count(&self) -> usize {
        self.overflow_store.entry_count()
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

    #[test]
    fn test_dump_load_roundtrip() {
        let mut table = PropertyTable::new();
        table.add_property("weight".to_string(), DataType::Double, false);
        table.add_property("since".to_string(), DataType::Int, true);

        let offset1 = table
            .insert(&[
                ("weight".to_string(), Value::Double(1.5)),
                ("since".to_string(), Value::Int(2020)),
            ])
            .unwrap();

        let offset2 = table
            .insert(&[
                ("weight".to_string(), Value::Double(2.5)),
                ("since".to_string(), Value::Int(2021)),
            ])
            .unwrap();

        let data = table.dump();

        let mut loaded_table = PropertyTable::new();
        let _ = loaded_table.load(&data);

        assert_eq!(loaded_table.property_count(), 2);
        assert_eq!(loaded_table.row_count(), 2);

        assert_eq!(
            loaded_table.get_property(offset1, "weight"),
            Some(Value::Double(1.5))
        );
        assert_eq!(
            loaded_table.get_property(offset2, "weight"),
            Some(Value::Double(2.5))
        );
    }

    #[test]
    fn test_overflow_storage() {
        let mut table = PropertyTable::new();
        table.add_property("data".to_string(), DataType::String, false);

        let large_string = "x".repeat(OVERFLOW_THRESHOLD + 100);
        let offset = table
            .insert(&[("data".to_string(), Value::String(large_string.clone()))])
            .unwrap();

        assert_eq!(table.overflow_count(), 1);

        let retrieved = table.get_property(offset, "data");
        assert_eq!(retrieved, Some(Value::String(large_string)));
    }

    #[test]
    fn test_overflow_storage_with_dump_load() {
        let mut table = PropertyTable::new();
        table.add_property("data".to_string(), DataType::String, false);

        let large_string = "x".repeat(OVERFLOW_THRESHOLD + 100);
        let offset = table
            .insert(&[("data".to_string(), Value::String(large_string.clone()))])
            .unwrap();

        let data = table.dump();

        let mut loaded_table = PropertyTable::new();
        let _ = loaded_table.load(&data);

        assert_eq!(loaded_table.overflow_count(), 1);

        let retrieved = loaded_table.get_property(offset, "data");
        assert_eq!(retrieved, Some(Value::String(large_string)));
    }

    #[test]
    fn test_row_groups() {
        let mut table = PropertyTable::with_row_group_size(100, 10);
        table.add_property("id".to_string(), DataType::Int, false);

        for i in 0..25 {
            table.insert(&[("id".to_string(), Value::Int(i))]).unwrap();
        }

        assert!(table.row_group_count() > 0);

        let group = table.get_row_group(0).unwrap();
        assert_eq!(group.start_row, 0);
        assert!(group.end_row <= 10);
    }

    #[test]
    fn test_encoding_application() {
        let mut table = PropertyTable::new();
        table.add_property("status".to_string(), DataType::Int, false);

        for i in 0..100 {
            table
                .insert(&[(
                    "status".to_string(),
                    Value::Int(if i % 2 == 0 { 1 } else { 0 }),
                )])
                .unwrap();
        }

        assert!(table
            .apply_encoding(PropertyId(0), EncodingType::Rle)
            .is_ok());
    }

    #[test]
    fn test_compact_row_group() {
        let mut table = PropertyTable::with_row_group_size(100, 10);
        table.add_property("id".to_string(), DataType::Int, false);

        let offsets: Vec<u32> = (0..15)
            .map(|i| table.insert(&[("id".to_string(), Value::Int(i))]).unwrap())
            .collect();

        table.delete(offsets[5]);
        table.delete(offsets[10]);

        assert!(table.compact_row_group(0).is_ok());
    }
}
