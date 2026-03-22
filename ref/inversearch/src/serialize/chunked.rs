//! 分块序列化模块
//!
//! 提供大数据的分块导入导出功能

use crate::serialize::{SerializeConfig, IndexExportData};
use crate::Index;
use crate::error::Result;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use bincode;

const CHUNK_SIZE_REG: usize = 250000;
const CHUNK_SIZE_MAP: usize = 5000;
const CHUNK_SIZE_CTX: usize = 1000;

/// 分块数据类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkDataType {
    Registry,
    MainIndex,
    ContextIndex,
}

/// 分块数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkData {
    pub chunk_index: usize,
    pub total_chunks: usize,
    pub data_type: ChunkDataType,
    pub data: Vec<u8>,
}

/// 分块序列化器
pub struct ChunkedSerializer {
    config: SerializeConfig,
}

impl ChunkedSerializer {
    /// 创建新的分块序列化器
    pub fn new(config: SerializeConfig) -> Self {
        Self { config }
    }

    /// 计算动态块大小
    fn calculate_chunk_size(&self, total_size: usize, base_size: usize) -> usize {
        if total_size <= base_size {
            return base_size;
        }
        base_size
    }

    /// 分块导出
    pub fn export_chunked<F>(&self, index: &Index, callback: F) -> Result<()>
    where
        F: FnMut(ChunkData) -> Result<()>,
    {
        let export_data = index.export(&self.config)?;

        let mut callback = callback;

        // 分块导出注册表
        self.export_registry_chunked(&export_data.data.registry, &mut callback)?;

        // 分块导出主索引
        self.export_main_index_chunked(&export_data.data.main_index, &mut callback)?;

        // 分块导出上下文索引
        self.export_context_index_chunked(&export_data.data.context_index, &mut callback)?;

        Ok(())
    }

    /// 分块导出注册表
    fn export_registry_chunked<F>(&self, registry: &crate::serialize::RegistryData, callback: &mut F) -> Result<()>
    where
        F: FnMut(ChunkData) -> Result<()>,
    {
        let mut items = Vec::new();

        match registry {
            crate::serialize::RegistryData::Set(doc_ids) => {
                for &doc_id in doc_ids {
                    items.push(doc_id.to_string());
                }
            },
            crate::serialize::RegistryData::Map(map) => {
                for (&doc_id, _) in map {
                    items.push(doc_id.to_string());
                }
            }
        }

        let chunk_size = self.calculate_chunk_size(items.len(), CHUNK_SIZE_REG);
        let total_chunks = (items.len() + chunk_size - 1) / chunk_size;

        for (chunk_index, chunk) in items.chunks(chunk_size).enumerate() {
            let chunk_data = ChunkData {
                chunk_index,
                total_chunks,
                data_type: ChunkDataType::Registry,
                data: bincode::serialize(chunk)?,
            };
            callback(chunk_data)?;
        }

        Ok(())
    }

    /// 分块导出主索引
    fn export_main_index_chunked<F>(
        &self,
        main_index: &HashMap<String, Vec<u64>>,
        callback: &mut F,
    ) -> Result<()>
    where
        F: FnMut(ChunkData) -> Result<()>,
    {
        let mut items: Vec<(String, Vec<u64>)> = main_index.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let chunk_size = self.calculate_chunk_size(items.len(), CHUNK_SIZE_MAP);
        let total_chunks = (items.len() + chunk_size - 1) / chunk_size;

        for (chunk_index, chunk) in items.chunks(chunk_size).enumerate() {
            let chunk_data = ChunkData {
                chunk_index,
                total_chunks,
                data_type: ChunkDataType::MainIndex,
                data: bincode::serialize(chunk)?,
            };
            callback(chunk_data)?;
        }

        Ok(())
    }

    /// 分块导出上下文索引
    fn export_context_index_chunked<F>(
        &self,
        context_index: &HashMap<String, HashMap<String, Vec<u64>>>,
        callback: &mut F,
    ) -> Result<()>
    where
        F: FnMut(ChunkData) -> Result<()>,
    {
        let mut items: Vec<(String, HashMap<String, Vec<u64>>)> = context_index.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let chunk_size = self.calculate_chunk_size(items.len(), CHUNK_SIZE_CTX);
        let total_chunks = (items.len() + chunk_size - 1) / chunk_size;

        for (chunk_index, chunk) in items.chunks(chunk_size).enumerate() {
            let chunk_data = ChunkData {
                chunk_index,
                total_chunks,
                data_type: ChunkDataType::ContextIndex,
                data: bincode::serialize(chunk)?,
            };
            callback(chunk_data)?;
        }

        Ok(())
    }

    /// 分块导入
    pub fn import_chunked<F>(&self, index: &mut Index, mut provider: F) -> Result<()>
    where
        F: FnMut() -> Result<Option<ChunkData>>,
    {
        let mut registry_data: Option<crate::serialize::RegistryData> = None;
        let mut main_index_data: HashMap<String, Vec<u64>> = HashMap::new();
        let mut context_index_data: HashMap<String, HashMap<String, Vec<u64>>> = HashMap::new();

        while let Some(chunk) = provider()? {
            match chunk.data_type {
                ChunkDataType::Registry => {
                    let items: Vec<String> = bincode::deserialize(&chunk.data)?;
                    if registry_data.is_none() {
                        registry_data = Some(crate::serialize::RegistryData::Set(Vec::new()));
                    }
                    if let Some(crate::serialize::RegistryData::Set(ref mut set)) = registry_data {
                        for item in items {
                            if let Ok(id) = item.parse::<u64>() {
                                set.push(id);
                            }
                        }
                    }
                },
                ChunkDataType::MainIndex => {
                    let items: Vec<(String, Vec<u64>)> = bincode::deserialize(&chunk.data)?;
                    for (key, value) in items {
                        main_index_data.insert(key, value);
                    }
                },
                ChunkDataType::ContextIndex => {
                    let items: Vec<(String, HashMap<String, Vec<u64>>)> = bincode::deserialize(&chunk.data)?;
                    for (key, value) in items {
                        context_index_data.insert(key, value);
                    }
                },
            }
        }

        // 导入收集的数据
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

/// 分块数据提供器
pub struct ChunkDataProvider {
    chunks: Vec<ChunkData>,
    current_index: usize,
}

impl ChunkDataProvider {
    /// 创建新的分块数据提供器
    pub fn new(chunks: Vec<ChunkData>) -> Self {
        Self {
            chunks,
            current_index: 0,
        }
    }

    /// 获取下一个分块
    pub fn next(&mut self) -> Result<Option<ChunkData>> {
        if self.current_index < self.chunks.len() {
            let chunk = self.chunks[self.current_index].clone();
            self.current_index += 1;
            Ok(Some(chunk))
        } else {
            Ok(None)
        }
    }

    /// 检查是否还有更多分块
    pub fn has_more(&self) -> bool {
        self.current_index < self.chunks.len()
    }

    /// 获取总块数
    pub fn total_chunks(&self) -> usize {
        self.chunks.len()
    }

    /// 重置提供器
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

        // 分块导出
        serializer.export_chunked(&index, |chunk| {
            chunks.push(chunk);
            Ok(())
        }).unwrap();

        // 验证分块
        assert!(!chunks.is_empty());

        // 分块导入
        let mut imported_index = Index::default();
        let mut provider = ChunkDataProvider::new(chunks);
        serializer.import_chunked(&mut imported_index, || provider.next()).unwrap();

        // 验证导入结果
        let results = imported_index.search_simple("hello").unwrap();
        assert!(results.contains(&1));
        assert!(results.contains(&3));
        assert!(!results.contains(&2));
    }

    #[test]
    fn test_chunk_size_calculation() {
        let serializer = ChunkedSerializer::default();

        let size1 = serializer.calculate_chunk_size(1000, CHUNK_SIZE_MAP);
        assert_eq!(size1, CHUNK_SIZE_MAP);

        let size2 = serializer.calculate_chunk_size(500000, CHUNK_SIZE_MAP);
        assert!(size2 >= 1);
    }

    #[test]
    fn test_chunk_data_provider() {
        let chunks = vec![
            ChunkData {
                chunk_index: 0,
                total_chunks: 2,
                data_type: ChunkDataType::Registry,
                data: vec![1, 2, 3],
            },
            ChunkData {
                chunk_index: 1,
                total_chunks: 2,
                data_type: ChunkDataType::Registry,
                data: vec![4, 5, 6],
            },
        ];

        let mut provider = ChunkDataProvider::new(chunks);
        assert_eq!(provider.total_chunks(), 2);
        assert!(provider.has_more());

        let chunk1 = provider.next().unwrap().unwrap();
        assert_eq!(chunk1.chunk_index, 0);

        let chunk2 = provider.next().unwrap().unwrap();
        assert_eq!(chunk2.chunk_index, 1);

        assert!(!provider.has_more());
        assert!(provider.next().unwrap().is_none());

        provider.reset();
        assert!(provider.has_more());
    }
}
