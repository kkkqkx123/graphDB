//! Property Table for Edges
//!
//! Stores edge properties using columnar storage for all values.
//! All property values (regardless of size) are stored using the same columnar format
//! that vertex properties use, ensuring consistency and enabling compression
//! and predicate pushdown for all columns.

use std::collections::HashSet;

use crate::core::{DataType, DateValue, NullType, StorageError, StorageResult, Value};
use crate::storage::encoding::{CompressionConfig, CompressionSelector, EncodingType};
use crate::storage::naming::NameIndexer;
use crate::storage::persistence::{read_header, read_u32_le, read_u64_le, section, write_header};
use crate::storage::types::PropertyId;
use crate::storage::vertex::column_store::Column;

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
    pub encoding: Option<EncodingType>,
}

impl PropertySchema {
    pub fn new(name: String, prop_id: i32, data_type: DataType) -> Self {
        Self {
            name,
            prop_id,
            data_type,
            nullable: false,
            encoding: None,
        }
    }

    pub fn nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }

    pub fn with_encoding(mut self, encoding: EncodingType) -> Self {
        self.encoding = Some(encoding);
        self
    }
}

#[derive(Debug, Clone)]
pub struct RowGroup {
    pub start_row: usize,
    pub end_row: usize,
}

impl RowGroup {
    pub fn new(start_row: usize, end_row: usize) -> Self {
        Self { start_row, end_row }
    }
}

#[derive(Debug, Clone)]
pub struct PropertyTable {
    schema: Vec<PropertySchema>,
    name_indexer: NameIndexer,
    columns: Vec<Column>,
    row_count: usize,
    free_list: Vec<u32>,
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
            row_groups: Vec::new(),
            row_group_size: DEFAULT_ROW_GROUP_SIZE,
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

    pub fn remove_property(&mut self, name: &str) -> StorageResult<()> {
        let index = self
            .schema
            .iter()
            .position(|prop| prop.name == name)
            .ok_or_else(|| StorageError::column_not_found(name.to_string()))?;

        self.schema.remove(index);
        self.columns.remove(index);

        self.name_indexer.clear();
        for (idx, schema) in self.schema.iter_mut().enumerate() {
            schema.prop_id = idx as i32;
            self.name_indexer.register(schema.name.clone());
        }

        for (idx, column) in self.columns.iter_mut().enumerate() {
            column.col_id = idx as i32;
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
        self.columns[index].name = new_name.to_string();

        self.name_indexer.clear();
        for (idx, schema) in self.schema.iter_mut().enumerate() {
            schema.prop_id = idx as i32;
            self.name_indexer.register(schema.name.clone());
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

    pub fn auto_apply_encodings(&mut self, config: Option<CompressionConfig>) -> StorageResult<()> {
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
                    self.columns[col_idx].set(row_idx, Some(value))?;
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
                .map(|col| {
                    (col.name.clone(), col.get(row_idx))
                })
                .collect(),
        )
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
            self.columns[col_idx].set(row_idx, value.as_ref())?;
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

        self.columns[col_idx].set(row_idx, value.as_ref())?;

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
            let _ = self.columns[col_idx].set(row_idx, None);
        }
        self.free_list.push(offset);
        true
    }

    pub fn has_property(&self, name: &str) -> bool {
        self.name_indexer.contains(name)
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
                section::PROPERTY_TABLE,
                section_id
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

            let prop_id_bytes: [u8; 4] = data[offset..offset + 4]
                .try_into()
                .map_err(|_| StorageError::deserialize_error("failed to read prop_id"))?;
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

        // Build row_groups from loaded headers
        self.row_groups.clear();
        for (start_row, end_row) in &loaded_row_groups {
            self.row_groups.push(RowGroup::new(*start_row, *end_row));
        }

        // Re-apply column encodings from persisted schema metadata
        let encoding_restore: Vec<(PropertyId, EncodingType)> = self
            .schema
            .iter()
            .enumerate()
            .filter_map(|(col_idx, schema)| {
                schema.encoding.and_then(|enc| {
                    if enc != EncodingType::None && col_idx < self.columns.len() {
                        Some((PropertyId::new(schema.prop_id as u16), enc))
                    } else {
                        None
                    }
                })
            })
            .collect();
        for (prop_id, enc) in encoding_restore {
            self.apply_encoding(prop_id, enc)?;
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
                    let value = col.get(old_row_idx);
                    let _ = new_columns[col_idx].set(new_row_count, value.as_ref());
                }
                new_row_count += 1;
            }
        }

        self.columns = new_columns;
        self.row_count = new_row_count;
        self.free_list.clear();

        self.row_groups.clear();
        let mut group_start = 0;
        while group_start < self.row_count {
            let group_end = (group_start + self.row_group_size).min(self.row_count);
            self.row_groups.push(RowGroup::new(group_start, group_end));
            group_start = group_end;
        }

        let _ = self.auto_apply_encodings(None);
    }

    pub fn used_memory_size(&self) -> usize {
        let mut total = 0;
        for col in &self.columns {
            total += col.used_memory_size();
        }
        total + std::mem::size_of::<Self>()
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
