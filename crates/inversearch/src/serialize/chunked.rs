//! Chunked Serialization Module
//!
//! Provide chunked import and export functions for big data

use crate::error::Result;
use crate::serialize::types::*;
use crate::Index;
use oxicode::config::standard;
use oxicode::serde::{decode_from_slice, encode_to_vec};
use std::collections::HashMap;

const CHUNK_SIZE_REG: usize = 250000;
const CHUNK_SIZE_MAP: usize = 5000;
const CHUNK_SIZE_CTX: usize = 1000;

/// chunking serializer
pub struct ChunkedSerializer {
    config: SerializeConfig,
}

impl ChunkedSerializer {
    /// Creating a new chunked serializer
    pub fn new(config: SerializeConfig) -> Self {
        Self { config }
    }

    /// Calculating Dynamic Block Size
    fn calculate_chunk_size(&self, total_size: usize, base_size: usize) -> usize {
        if total_size <= base_size {
            return base_size;
        }
        base_size
    }

    /// Chunk Export
    pub fn export_chunked<F>(&self, index: &Index, callback: F) -> Result<()>
    where
        F: FnMut(ChunkData) -> Result<()>,
    {
        let export_data = index.export(&self.config)?;

        let mut callback = callback;

        // Exporting the registry in chunks
        self.export_registry_chunked(&export_data.data.registry, &mut callback)?;

        // Chunked export of primary indexes
        self.export_main_index_chunked(&export_data.data.main_index, &mut callback)?;

        // Exporting Context Indexes in Chunks
        self.export_context_index_chunked(&export_data.data.context_index, &mut callback)?;

        Ok(())
    }

    /// Exporting the registry in chunks
    fn export_registry_chunked<F>(&self, registry: &RegistryData, callback: &mut F) -> Result<()>
    where
        F: FnMut(ChunkData) -> Result<()>,
    {
        let mut items = Vec::new();

        match registry {
            RegistryData::Set(doc_ids) => {
                for &doc_id in doc_ids {
                    items.push(doc_id.to_string());
                }
            }
            RegistryData::Map(map) => {
                for doc_id in map.keys() {
                    items.push(doc_id.to_string());
                }
            }
        }

        let chunk_size = self.calculate_chunk_size(items.len(), CHUNK_SIZE_REG);
        let total_chunks = items.len().div_ceil(chunk_size);

        for (chunk_index, chunk) in items.chunks(chunk_size).enumerate() {
            let chunk_data = ChunkData {
                chunk_index,
                total_chunks,
                data_type: ChunkDataType::Registry,
                data: encode_to_vec(&chunk.to_vec(), standard())?,
            };
            callback(chunk_data)?;
        }

        Ok(())
    }

    /// Chunked export of primary indexes
    fn export_main_index_chunked<F>(
        &self,
        main_index: &HashMap<String, Vec<u64>>,
        callback: &mut F,
    ) -> Result<()>
    where
        F: FnMut(ChunkData) -> Result<()>,
    {
        let items: Vec<(String, Vec<u64>)> = main_index
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let chunk_size = self.calculate_chunk_size(items.len(), CHUNK_SIZE_MAP);
        let total_chunks = items.len().div_ceil(chunk_size);

        for (chunk_index, chunk) in items.chunks(chunk_size).enumerate() {
            let chunk_data = ChunkData {
                chunk_index,
                total_chunks,
                data_type: ChunkDataType::MainIndex,
                data: encode_to_vec(&chunk.to_vec(), standard())?,
            };
            callback(chunk_data)?;
        }

        Ok(())
    }

    /// Exporting Context Indexes in Chunks
    fn export_context_index_chunked<F>(
        &self,
        context_index: &HashMap<String, HashMap<String, Vec<u64>>>,
        callback: &mut F,
    ) -> Result<()>
    where
        F: FnMut(ChunkData) -> Result<()>,
    {
        let items: Vec<(String, HashMap<String, Vec<u64>>)> = context_index
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let chunk_size = self.calculate_chunk_size(items.len(), CHUNK_SIZE_CTX);
        let total_chunks = items.len().div_ceil(chunk_size);

        for (chunk_index, chunk) in items.chunks(chunk_size).enumerate() {
            let chunk_data = ChunkData {
                chunk_index,
                total_chunks,
                data_type: ChunkDataType::ContextIndex,
                data: encode_to_vec(&chunk.to_vec(), standard())?,
            };
            callback(chunk_data)?;
        }

        Ok(())
    }

    /// chunking
    pub fn import_chunked<F>(&self, index: &mut Index, mut provider: F) -> Result<()>
    where
        F: FnMut() -> Result<Option<ChunkData>>,
    {
        let mut registry_data: Option<RegistryData> = None;
        let mut main_index_data: HashMap<String, Vec<u64>> = HashMap::new();
        let mut context_index_data: HashMap<String, HashMap<String, Vec<u64>>> = HashMap::new();

        while let Some(chunk) = provider()? {
            match chunk.data_type {
                ChunkDataType::Registry => {
                    let (items, _): (Vec<String>, usize) =
                        decode_from_slice(&chunk.data, standard())?;
                    if registry_data.is_none() {
                        registry_data = Some(RegistryData::Set(Vec::new()));
                    }
                    if let Some(RegistryData::Set(ref mut set)) = registry_data {
                        for item in items {
                            if let Ok(id) = item.parse::<u64>() {
                                set.push(id);
                            }
                        }
                    }
                }
                ChunkDataType::MainIndex => {
                    let (items, _): (Vec<(String, Vec<u64>)>, usize) =
                        decode_from_slice(&chunk.data, standard())?;
                    for (key, value) in items {
                        main_index_data.insert(key, value);
                    }
                }
                ChunkDataType::ContextIndex => {
                    let (items, _): (Vec<(String, HashMap<String, Vec<u64>>)>, usize) =
                        decode_from_slice(&chunk.data, standard())?;
                    for (key, value) in items {
                        context_index_data.insert(key, value);
                    }
                }
            }
        }

        // Importing collected data
        if let Some(registry) = registry_data {
            index.import_registry(&registry)?;
        }
        index.import_main_index(&main_index_data)?;
        index.import_context_index(&context_index_data)?;

        Ok(())
    }
}

impl Default for ChunkedSerializer {
    fn default() -> Self {
        Self::new(SerializeConfig::default())
    }
}

/// chunking data provider
pub struct ChunkDataProvider {
    chunks: Vec<ChunkData>,
    current_index: usize,
}

impl ChunkDataProvider {
    /// Creating a new chunked data provider
    pub fn new(chunks: Vec<ChunkData>) -> Self {
        Self {
            chunks,
            current_index: 0,
        }
    }

    /// Get the next chunk
    pub fn fetch_next(&mut self) -> Result<Option<ChunkData>> {
        if self.current_index < self.chunks.len() {
            let chunk = self.chunks[self.current_index].clone();
            self.current_index += 1;
            Ok(Some(chunk))
        } else {
            Ok(None)
        }
    }

    /// Check if there are more chunks
    pub fn has_more(&self) -> bool {
        self.current_index < self.chunks.len()
    }

    /// Get the total number of blocks
    pub fn total_chunks(&self) -> usize {
        self.chunks.len()
    }

    /// reset provider
    pub fn reset(&mut self) {
        self.current_index = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunked_export_import() {
        let mut index = Index::default();
        index.add(1, "hello world", false).unwrap();
        index.add(2, "rust programming", false).unwrap();
        index.add(3, "hello rust", false).unwrap();

        let serializer = ChunkedSerializer::default();
        let mut chunks = Vec::new();

        // Chunk Export
        serializer
            .export_chunked(&index, |chunk| {
                chunks.push(chunk);
                Ok(())
            })
            .unwrap();

        // validate chunking
        assert!(!chunks.is_empty());

        // chunking
        let mut imported_index = Index::default();
        let mut provider = ChunkDataProvider::new(chunks);
        serializer
            .import_chunked(&mut imported_index, || provider.fetch_next())
            .unwrap();

        // Verify import results
        let results = imported_index.search_simple("hello").unwrap();
        assert!(results.contains(&1));
        assert!(results.contains(&3));
        assert!(!results.contains(&2));
    }

    #[test]
    fn test_chunk_data_provider() {
        let chunks = vec![
            ChunkData {
                chunk_index: 0,
                total_chunks: 2,
                data_type: ChunkDataType::MainIndex,
                data: vec![1, 2, 3],
            },
            ChunkData {
                chunk_index: 1,
                total_chunks: 2,
                data_type: ChunkDataType::MainIndex,
                data: vec![4, 5, 6],
            },
        ];

        let mut provider = ChunkDataProvider::new(chunks);

        assert!(provider.has_more());
        assert_eq!(provider.total_chunks(), 2);

        let chunk1 = provider.fetch_next().unwrap().unwrap();
        assert_eq!(chunk1.chunk_index, 0);

        let chunk2 = provider.fetch_next().unwrap().unwrap();
        assert_eq!(chunk2.chunk_index, 1);

        assert!(!provider.has_more());
        assert!(provider.fetch_next().unwrap().is_none());

        provider.reset();
        assert!(provider.has_more());
    }
}
