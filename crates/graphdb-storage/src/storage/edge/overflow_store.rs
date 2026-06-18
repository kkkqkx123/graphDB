//! Overflow Store for Property Values
//!
//! Manages storage of property values that exceed the overflow threshold (256 bytes).
//! Uses a continuous memory pool with best-fit allocation and free list coalescing.

use std::collections::HashMap;
use crate::core::{StorageError, StorageResult, Value};
use crate::storage::persistence::{read_header, read_u32_le, read_u64_le, section};

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

#[derive(Debug, Clone)]
pub struct OverflowPointer;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OverflowKey {
    pub col_idx: usize,
    pub row_idx: usize,
}

/// Stores values that exceed the overflow threshold (>256 bytes).
///
/// Uses a continuous memory pool with best-fit allocation strategy.
/// Tracks location of each value for efficient retrieval and removal.
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
        self.location_index
            .insert(OverflowKey { col_idx, row_idx }, id);
        self.entry_count += 1;

        OverflowPointer
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
                self.free_list
                    .push((offset + needed_size as u64, size - needed_size));
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

    pub fn remap_column_indices(&mut self, removed_col: usize) {
        let mut remapped = HashMap::new();
        for (key, overflow_id) in self.location_index.drain() {
            if key.col_idx == removed_col {
                continue;
            }
            let new_key = if key.col_idx > removed_col {
                OverflowKey {
                    col_idx: key.col_idx - 1,
                    row_idx: key.row_idx,
                }
            } else {
                key
            };
            remapped.insert(new_key, overflow_id);
        }
        self.location_index = remapped;
    }

    pub fn memory_size(&self) -> usize {
        let mut total = std::mem::size_of::<Self>();
        total += self.data_pool.capacity();
        total += self.index.capacity()
            * (std::mem::size_of::<u64>() + std::mem::size_of::<(u64, u32)>());
        total += self.location_index.capacity()
            * (std::mem::size_of::<OverflowKey>() + std::mem::size_of::<u64>());
        total += self.free_list.capacity() * std::mem::size_of::<(u64, u32)>();
        total
    }

    pub fn dump(&self) -> Vec<u8> {
        let mut result = Vec::new();

        use crate::storage::persistence::write_header;

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
                section::OVERFLOW_STORE,
                section_id
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
            self.location_index
                .insert(OverflowKey { col_idx, row_idx }, overflow_id);
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
