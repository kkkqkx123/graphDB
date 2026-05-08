//! SSTable (Sorted String Table) Implementation
//!
//! Provides persistent storage for sorted key-value pairs following RocksDB's SSTable design.
//! Structure:
//! [Data Block 1]
//! [Data Block 2]
//! ...
//! [Filter Block] (Bloom Filter)
//! [Index Block]
//! [Footer]

use crate::core::{StorageError, StorageResult};
use crate::storage::persistence::{CompressionType, Compressor};
use crate::storage::vertex::encoding::varint::Varint;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub const SSTABLE_BLOCK_SIZE: usize = 4 * 1024;
pub const SSTABLE_MAGIC_NUMBER: u32 = 0x53535442;
pub const SSTABLE_VERSION: u32 = 1;

trait WriteExt: Write {
    fn write_u32_le(&mut self, value: u32) -> std::io::Result<()> {
        self.write_all(&value.to_le_bytes())
    }

    fn write_u64_le(&mut self, value: u64) -> std::io::Result<()> {
        self.write_all(&value.to_le_bytes())
    }
}

impl<W: Write> WriteExt for W {}

trait ReadExt: Read {
    fn read_u32_le(&mut self) -> std::io::Result<u32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    fn read_u64_le(&mut self) -> std::io::Result<u64> {
        let mut buf = [0u8; 8];
        self.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }
}

impl<R: Read> ReadExt for R {}

#[derive(Debug, Clone)]
pub struct SsTableConfig {
    pub block_size: usize,
    pub compression: CompressionType,
    pub use_bloom_filter: bool,
    pub bloom_filter_bits_per_key: i32,
}

impl Default for SsTableConfig {
    fn default() -> Self {
        Self {
            block_size: SSTABLE_BLOCK_SIZE,
            compression: CompressionType::Zstd { level: 3 },
            use_bloom_filter: true,
            bloom_filter_bits_per_key: 10,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SsTableBuilder {
    config: SsTableConfig,
    data_blocks: Vec<DataBlock>,
    current_block: DataBlock,
    index_entries: Vec<IndexEntry>,
    keys: Vec<Vec<u8>>,
}

impl SsTableBuilder {
    pub fn new(config: SsTableConfig) -> Self {
        let block_size = config.block_size;
        Self {
            config,
            data_blocks: Vec::new(),
            current_block: DataBlock::new(block_size),
            index_entries: Vec::new(),
            keys: Vec::new(),
        }
    }

    pub fn add(&mut self, key: &[u8], value: &[u8]) -> StorageResult<()> {
        if self.current_block.is_full(key.len() + value.len()) {
            self.flush_current_block()?;
        }

        self.current_block.add(key, value);
        self.keys.push(key.to_vec());
        Ok(())
    }

    fn flush_current_block(&mut self) -> StorageResult<()> {
        if self.current_block.is_empty() {
            return Ok(());
        }

        let first_key = self.current_block.first_key().ok_or_else(|| {
            StorageError::InvalidOperation("Cannot flush empty block".to_string())
        })?;

        let block_data = self.current_block.finish();
        let compressor = Compressor::new(self.config.compression);
        let compressed_data = compressor.compress(&block_data)?;

        let offset = self.data_blocks.iter().map(|b| b.data.len()).sum::<usize>();
        let size = compressed_data.len();

        self.index_entries.push(IndexEntry {
            key: first_key,
            offset,
            size,
        });

        self.data_blocks.push(DataBlock {
            data: compressed_data,
            max_size: 0,
            entries: Vec::new(),
        });

        self.current_block = DataBlock::new(self.config.block_size);
        Ok(())
    }

    pub fn finish(mut self, path: &Path) -> StorageResult<SsTableMetadata> {
        self.flush_current_block()?;

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        let mut writer = BufWriter::new(file);
        let mut offset = 0usize;

        for block in &self.data_blocks {
            writer.write_all(&block.data)?;
            offset += block.data.len();
        }

        let filter_block = if self.config.use_bloom_filter {
            Some(self.build_filter_block()?)
        } else {
            None
        };

        let filter_offset = offset;
        let filter_size = if let Some(ref filter) = filter_block {
            writer.write_all(filter)?;
            offset += filter.len();
            filter.len()
        } else {
            0
        };

        let index_block = self.build_index_block()?;
        let index_offset = offset;
        let index_size = index_block.len();
        writer.write_all(&index_block)?;

        let footer = Footer {
            version: SSTABLE_VERSION,
            filter_offset,
            filter_size,
            index_offset,
            index_size,
        };

        footer.write_to(&mut writer)?;
        writer.flush()?;

        Ok(SsTableMetadata {
            file_path: path.to_path_buf(),
            key_count: self.keys.len(),
            data_block_count: self.data_blocks.len(),
            file_size: offset + Footer::size(),
        })
    }

    fn build_filter_block(&self) -> StorageResult<Vec<u8>> {
        let mut filter = BloomFilter::new(self.keys.len(), self.config.bloom_filter_bits_per_key);

        for key in &self.keys {
            filter.add(key);
        }

        Ok(filter.to_bytes())
    }

    fn build_index_block(&self) -> StorageResult<Vec<u8>> {
        let mut data = Vec::new();

        for entry in &self.index_entries {
            let key_len = entry.key.len() as u64;
            Varint::encode_into(key_len, &mut data);
            data.write_all(&entry.key)?;
            Varint::encode_into(entry.offset as u64, &mut data);
            Varint::encode_into(entry.size as u64, &mut data);
        }

        Ok(data)
    }
}

#[derive(Debug, Clone)]
struct DataBlock {
    data: Vec<u8>,
    #[allow(dead_code)]
    max_size: usize,
    #[allow(dead_code)]
    entries: Vec<(Vec<u8>, Vec<u8>)>,
}

impl DataBlock {
    fn new(max_size: usize) -> Self {
        Self {
            data: Vec::new(),
            max_size,
            entries: Vec::new(),
        }
    }

    fn add(&mut self, key: &[u8], value: &[u8]) {
        self.entries.push((key.to_vec(), value.to_vec()));
    }

    fn is_full(&self, additional_size: usize) -> bool {
        self.estimate_size() + additional_size > self.max_size
    }

    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn first_key(&self) -> Option<Vec<u8>> {
        self.entries.first().map(|(k, _)| k.clone())
    }

    fn estimate_size(&self) -> usize {
        self.entries
            .iter()
            .map(|(k, v)| {
                k.len() + v.len() + Varint::encoded_len(k.len() as u64) + Varint::encoded_len(v.len() as u64)
            })
            .sum()
    }

    fn finish(&mut self) -> Vec<u8> {
        let mut data = Vec::new();

        for (key, value) in &self.entries {
            let key_len = key.len() as u64;
            let value_len = value.len() as u64;

            Varint::encode_into(key_len, &mut data);
            data.write_all(key).expect("Write failed");
            Varint::encode_into(value_len, &mut data);
            data.write_all(value).expect("Write failed");
        }

        self.data = data.clone();
        data
    }
}

#[derive(Debug, Clone)]
struct IndexEntry {
    key: Vec<u8>,
    offset: usize,
    size: usize,
}

#[derive(Debug, Clone)]
struct Footer {
    version: u32,
    filter_offset: usize,
    filter_size: usize,
    index_offset: usize,
    index_size: usize,
}

impl Footer {
    fn size() -> usize {
        4 + 8 + 8 + 8 + 8 + 4
    }

    fn write_to<W: Write>(&self, writer: &mut W) -> StorageResult<()> {
        writer.write_u32_le(SSTABLE_MAGIC_NUMBER)?;
        writer.write_u32_le(self.version)?;
        writer.write_u64_le(self.filter_offset as u64)?;
        writer.write_u64_le(self.filter_size as u64)?;
        writer.write_u64_le(self.index_offset as u64)?;
        writer.write_u64_le(self.index_size as u64)?;
        writer.write_u32_le(SSTABLE_MAGIC_NUMBER)?;
        Ok(())
    }

    fn read_from<R: Read>(reader: &mut R) -> StorageResult<Self> {
        let magic = reader.read_u32_le()?;
        if magic != SSTABLE_MAGIC_NUMBER {
            return Err(StorageError::DeserializeError(
                "Invalid SSTable magic number".to_string(),
            ));
        }

        let version = reader.read_u32_le()?;
        let filter_offset = reader.read_u64_le()? as usize;
        let filter_size = reader.read_u64_le()? as usize;
        let index_offset = reader.read_u64_le()? as usize;
        let index_size = reader.read_u64_le()? as usize;

        let magic_end = reader.read_u32_le()?;
        if magic_end != SSTABLE_MAGIC_NUMBER {
            return Err(StorageError::DeserializeError(
                "Invalid SSTable magic number at end".to_string(),
            ));
        }

        Ok(Self {
            version,
            filter_offset,
            filter_size,
            index_offset,
            index_size,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SsTableMetadata {
    pub file_path: PathBuf,
    pub key_count: usize,
    pub data_block_count: usize,
    pub file_size: usize,
}

pub struct SsTableReader {
    file: File,
    file_path: PathBuf,
    footer: Footer,
    index: Vec<IndexEntry>,
    filter: Option<BloomFilter>,
    compressor: Compressor,
}

impl SsTableReader {
    pub fn open(path: &Path) -> StorageResult<Self> {
        let mut file = OpenOptions::new().read(true).open(path)?;
        let file_path = path.to_path_buf();

        let file_size = file.metadata()?.len() as usize;
        let footer_offset = file_size.saturating_sub(Footer::size());

        file.seek(SeekFrom::Start(footer_offset as u64))?;
        let footer = Footer::read_from(&mut file)?;

        file.seek(SeekFrom::Start(footer.index_offset as u64))?;
        let mut index_data = vec![0u8; footer.index_size];
        file.read_exact(&mut index_data)?;

        let index = Self::parse_index_block(&index_data)?;

        let filter = if footer.filter_size > 0 {
            file.seek(SeekFrom::Start(footer.filter_offset as u64))?;
            let mut filter_data = vec![0u8; footer.filter_size];
            file.read_exact(&mut filter_data)?;
            Some(BloomFilter::from_bytes(&filter_data))
        } else {
            None
        };

        Ok(Self {
            file,
            file_path,
            footer,
            index,
            filter,
            compressor: Compressor::new(CompressionType::Zstd { level: 3 }),
        })
    }

    pub fn get(&mut self, key: &[u8]) -> StorageResult<Option<Vec<u8>>> {
        if let Some(ref filter) = self.filter {
            if !filter.may_contain(key) {
                return Ok(None);
            }
        }

        let block_idx = self.find_block(key);
        if block_idx >= self.index.len() {
            return Ok(None);
        }

        let entry = &self.index[block_idx];
        let block_data = self.read_block(entry.offset, entry.size)?;

        self.search_in_block(&block_data, key)
    }

    pub fn scan(&mut self, start_key: Option<&[u8]>, end_key: Option<&[u8]>) -> StorageResult<Vec<(Vec<u8>, Vec<u8>)>> {
        let mut results = Vec::new();

        let start_block = start_key
            .map(|k| self.find_block(k))
            .unwrap_or(0);

        for i in start_block..self.index.len() {
            let entry_offset = self.index[i].offset;
            let entry_size = self.index[i].size;
            let entry_key = self.index[i].key.clone();
            let block_data = self.read_block(entry_offset, entry_size)?;

            let block_results = self.scan_block(&block_data, start_key, end_key)?;
            results.extend(block_results);

            if let Some(end) = end_key {
                if entry_key.as_slice().cmp(end) == std::cmp::Ordering::Greater {
                    break;
                }
            }
        }

        Ok(results)
    }

    fn find_block(&self, key: &[u8]) -> usize {
        self.index
            .binary_search_by(|entry| entry.key.as_slice().cmp(key))
            .unwrap_or_else(|idx| idx.saturating_sub(1))
    }

    fn read_block(&mut self, offset: usize, size: usize) -> StorageResult<Vec<u8>> {
        self.file.seek(SeekFrom::Start(offset as u64))?;
        let mut compressed_data = vec![0u8; size];
        self.file.read_exact(&mut compressed_data)?;

        self.compressor.decompress(&compressed_data)
    }

    fn parse_index_block(data: &[u8]) -> StorageResult<Vec<IndexEntry>> {
        let mut entries = Vec::new();
        let mut offset = 0;

        while offset < data.len() {
            let (key_len, len) = Varint::decode_at(data, offset);
            if len == 0 {
                break;
            }
            offset += len;

            let key_len = key_len as usize;
            if offset + key_len > data.len() {
                break;
            }

            let key = data[offset..offset + key_len].to_vec();
            offset += key_len;

            let (block_offset, len) = Varint::decode_at(data, offset);
            if len == 0 {
                break;
            }
            offset += len;

            let (block_size, len) = Varint::decode_at(data, offset);
            if len == 0 {
                break;
            }
            offset += len;

            entries.push(IndexEntry {
                key,
                offset: block_offset as usize,
                size: block_size as usize,
            });
        }

        Ok(entries)
    }

    fn search_in_block(&self, block_data: &[u8], key: &[u8]) -> StorageResult<Option<Vec<u8>>> {
        let mut offset = 0;

        while offset < block_data.len() {
            let (key_len, len) = Varint::decode_at(block_data, offset);
            if len == 0 {
                break;
            }
            offset += len;

            let key_len = key_len as usize;
            if offset + key_len > block_data.len() {
                break;
            }

            let entry_key = &block_data[offset..offset + key_len];
            offset += key_len;

            let (value_len, len) = Varint::decode_at(block_data, offset);
            if len == 0 {
                break;
            }
            offset += len;

            let value_len = value_len as usize;
            if offset + value_len > block_data.len() {
                break;
            }

            let value = &block_data[offset..offset + value_len];
            offset += value_len;

            if entry_key == key {
                return Ok(Some(value.to_vec()));
            }
        }

        Ok(None)
    }

    fn scan_block(
        &self,
        block_data: &[u8],
        start_key: Option<&[u8]>,
        end_key: Option<&[u8]>,
    ) -> StorageResult<Vec<(Vec<u8>, Vec<u8>)>> {
        let mut results = Vec::new();
        let mut offset = 0;

        while offset < block_data.len() {
            let (key_len, len) = Varint::decode_at(block_data, offset);
            if len == 0 {
                break;
            }
            offset += len;

            let key_len = key_len as usize;
            if offset + key_len > block_data.len() {
                break;
            }

            let key = block_data[offset..offset + key_len].to_vec();
            offset += key_len;

            let (value_len, len) = Varint::decode_at(block_data, offset);
            if len == 0 {
                break;
            }
            offset += len;

            let value_len = value_len as usize;
            if offset + value_len > block_data.len() {
                break;
            }

            let value = block_data[offset..offset + value_len].to_vec();
            offset += value_len;

            let include = true
                && start_key.is_none_or(|start| key.as_slice() >= start)
                && end_key.is_none_or(|end| key.as_slice() <= end);

            if include {
                results.push((key, value));
            }
        }

        Ok(results)
    }

    pub fn key_count(&self) -> usize {
        self.index.len()
    }

    pub fn metadata(&self) -> StorageResult<SsTableMetadata> {
        let file_size = self.file.metadata()?.len() as usize;
        Ok(SsTableMetadata {
            file_path: self.file_path.clone(),
            key_count: self.index.len(),
            data_block_count: self.index.len(),
            file_size,
        })
    }
}

#[derive(Debug, Clone)]
struct BloomFilter {
    bits: Vec<u8>,
    bit_count: usize,
    hash_count: usize,
}

impl BloomFilter {
    fn new(expected_items: usize, bits_per_key: i32) -> Self {
        let bit_count = (expected_items as i64 * bits_per_key as i64) as usize;
        let bit_count = bit_count.max(64);

        let hash_count = Self::calculate_hash_count(bits_per_key);
        let byte_count = bit_count.div_ceil(8);

        Self {
            bits: vec![0u8; byte_count],
            bit_count,
            hash_count,
        }
    }

    fn calculate_hash_count(bits_per_key: i32) -> usize {
        let k = (bits_per_key as f64 * 0.69) as i32;
        k.clamp(1, 30) as usize
    }

    fn add(&mut self, key: &[u8]) {
        let hash = Self::hash(key);
        let delta = (hash >> 17) | (hash << 15);

        for i in 0..self.hash_count {
            let bit_pos = (hash.wrapping_add((i as u64).wrapping_mul(delta)) as usize) % self.bit_count;
            let byte_pos = bit_pos / 8;
            let bit_offset = bit_pos % 8;

            self.bits[byte_pos] |= 1 << bit_offset;
        }
    }

    fn may_contain(&self, key: &[u8]) -> bool {
        let hash = Self::hash(key);
        let delta = (hash >> 17) | (hash << 15);

        for i in 0..self.hash_count {
            let bit_pos = (hash.wrapping_add((i as u64).wrapping_mul(delta)) as usize) % self.bit_count;
            let byte_pos = bit_pos / 8;
            let bit_offset = bit_pos % 8;

            if self.bits.get(byte_pos).is_none_or(|&b| (b & (1 << bit_offset)) == 0) {
                return false;
            }
        }

        true
    }

    fn hash(key: &[u8]) -> u64 {
        let mut hash: u64 = 0;
        for &byte in key {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
        }
        hash
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&(self.bit_count as u64).to_le_bytes());
        data.extend_from_slice(&(self.hash_count as u64).to_le_bytes());
        data.extend_from_slice(&self.bits);
        data
    }

    fn from_bytes(data: &[u8]) -> Self {
        let bit_count = u64::from_le_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]]) as usize;
        let hash_count = u64::from_le_bytes([data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15]]) as usize;
        let bits = data[16..].to_vec();

        Self {
            bits,
            bit_count,
            hash_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_sstable_builder_and_reader() {
        let config = SsTableConfig::default();
        let mut builder = SsTableBuilder::new(config);

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path().to_path_buf();

        builder.add(b"key1", b"value1").expect("Add failed");
        builder.add(b"key2", b"value2").expect("Add failed");
        builder.add(b"key3", b"value3").expect("Add failed");

        let metadata = builder.finish(&temp_path).expect("Finish failed");
        assert_eq!(metadata.key_count, 3);

        let mut reader = SsTableReader::open(&temp_path).expect("Open failed");

        let value = reader.get(b"key1").expect("Get failed");
        assert_eq!(value, Some(b"value1".to_vec()));

        let value = reader.get(b"key2").expect("Get failed");
        assert_eq!(value, Some(b"value2".to_vec()));

        let value = reader.get(b"nonexistent").expect("Get failed");
        assert_eq!(value, None);
    }

    #[test]
    fn test_sstable_scan() {
        let config = SsTableConfig::default();
        let mut builder = SsTableBuilder::new(config);

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path().to_path_buf();

        for i in 0..100 {
            let key = format!("key{:03}", i);
            let value = format!("value{}", i);
            builder.add(key.as_bytes(), value.as_bytes()).expect("Add failed");
        }

        let metadata = builder.finish(&temp_path).expect("Finish failed");
        assert_eq!(metadata.key_count, 100);

        let mut reader = SsTableReader::open(&temp_path).expect("Open failed");

        let results = reader
            .scan(Some(b"key020"), Some(b"key040"))
            .expect("Scan failed");

        assert_eq!(results.len(), 21);
        assert_eq!(results[0].0, b"key020".to_vec());
        assert_eq!(results[20].0, b"key040".to_vec());
    }

    #[test]
    fn test_bloom_filter() {
        let mut filter = BloomFilter::new(1000, 10);

        filter.add(b"test_key");
        assert!(filter.may_contain(b"test_key"));
        assert!(!filter.may_contain(b"nonexistent_key"));
    }

    #[test]
    fn test_sstable_compression() {
        let config = SsTableConfig {
            compression: CompressionType::Zstd { level: 3 },
            ..Default::default()
        };
        let mut builder = SsTableBuilder::new(config);

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path().to_path_buf();

        let large_value = vec![b'x'; 1000];
        builder.add(b"key1", &large_value).expect("Add failed");

        let _metadata = builder.finish(&temp_path).expect("Finish failed");

        let mut reader = SsTableReader::open(&temp_path).expect("Open failed");
        let value = reader.get(b"key1").expect("Get failed");

        assert_eq!(value, Some(large_value));
    }

    #[test]
    fn test_sstable_metadata() {
        let config = SsTableConfig::default();
        let mut builder = SsTableBuilder::new(config);

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path().to_path_buf();

        for i in 0..50 {
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            builder.add(key.as_bytes(), value.as_bytes()).expect("Add failed");
        }

        let metadata = builder.finish(&temp_path).expect("Finish failed");

        assert_eq!(metadata.key_count, 50);
        assert!(metadata.file_size > 0);
        assert!(metadata.data_block_count > 0);
    }
}
