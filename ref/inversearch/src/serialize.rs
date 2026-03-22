//! 序列化模块
//! 
//! 提供索引的导入导出功能，支持JSON和二进制格式

pub mod r#async;
pub mod chunked;

use crate::r#type::{SearchResults, IntermediateSearchResults};
use crate::error::Result;
use crate::Index;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

pub use r#async::{AsyncSerializer, AsyncDocumentSerializer};
pub use chunked::{ChunkedSerializer, ChunkData, ChunkDataType, ChunkDataProvider};

/// 序列化配置
#[derive(Debug, Clone)]
pub struct SerializeConfig {
    pub format: SerializeFormat,
    pub compression: bool,
    pub chunk_size: usize,
}

impl Default for SerializeConfig {
    fn default() -> Self {
        Self {
            format: SerializeFormat::Json,
            compression: false,
            chunk_size: 1000,
        }
    }
}

/// 序列化格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializeFormat {
    Json,
    Binary,
}

/// 索引数据导出结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexExportData {
    pub version: String,
    pub created_at: String,
    pub index_info: IndexInfo,
    pub config: IndexConfigExport,
    pub data: ExportData,
}

/// 索引信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexInfo {
    pub resolution: usize,
    pub resolution_ctx: usize,
    pub tokenize_mode: String,
    pub depth: usize,
    pub bidirectional: bool,
    pub fastupdate: bool,
    pub rtl: bool,
    pub encoder_type: String,
}

/// 导出数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub main_index: HashMap<String, Vec<u64>>,
    pub context_index: HashMap<String, HashMap<String, Vec<u64>>>,
    pub registry: RegistryData,
}

/// 注册表数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegistryData {
    Set(Vec<u64>),                    // fastupdate = false
    Map(HashMap<u64, Vec<IndexRefData>>), // fastupdate = true
}

/// 索引引用数据（序列化格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexRefData {
    MapRef(String),
    CtxRef(String, String),
}

/// 索引配置导出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfigExport {
    pub index_options: crate::r#type::IndexOptions,
    pub encoder_options: crate::r#type::EncoderOptions,
    pub tokenizer_config: TokenizerConfig,
}

/// 分词器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenizerConfig {
    pub mode: String,
    pub separator: Option<String>,
    pub normalize: bool,
}

impl Index {
    /// 导出索引数据
    pub fn export(&self, _config: &SerializeConfig) -> Result<IndexExportData> {
        let index_info = IndexInfo {
            resolution: self.resolution,
            resolution_ctx: self.resolution_ctx,
            tokenize_mode: format!("{:?}", self.tokenize).to_lowercase(),
            depth: self.depth,
            bidirectional: self.bidirectional,
            fastupdate: self.fastupdate,
            rtl: self.rtl,
            encoder_type: "default".to_string(),
        };

        let main_index = self.export_main_index();
        let context_index = self.export_context_index();
        let registry = self.export_registry();

        let config_export = IndexConfigExport {
            index_options: self.get_index_options(),
            encoder_options: self.encoder.get_options(),
            tokenizer_config: self.get_tokenizer_config(),
        };

        let data = ExportData {
            main_index,
            context_index,
            registry,
        };

        Ok(IndexExportData {
            version: "0.1.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            index_info,
            config: config_export,
            data,
        })
    }

    /// 导出主索引
    fn export_main_index(&self) -> HashMap<String, Vec<u64>> {
        let mut result = HashMap::new();
        
        for (_term_hash, doc_ids) in &self.map.index {
            for (term_str, ids) in doc_ids {
                result.insert(term_str.clone(), ids.clone());
            }
        }
        
        result
    }

    /// 导出上下文索引
    fn export_context_index(&self) -> HashMap<String, HashMap<String, Vec<u64>>> {
        let mut result = HashMap::new();
        
        for (_ctx_key, ctx_map) in &self.ctx.index {
            let mut ctx_data = HashMap::new();
            for (ctx_term, doc_ids) in ctx_map {
                ctx_data.insert(ctx_term.clone(), doc_ids.clone());
            }
            // 简化处理，使用字符串表示
            let ctx_key_str = format!("ctx_{}", result.len());
            result.insert(ctx_key_str, ctx_data);
        }
        
        result
    }

    /// 导出注册表
    fn export_registry(&self) -> RegistryData {
        match &self.reg {
            crate::index::Register::Set(set) => {
                let mut doc_ids = Vec::new();
                for (_, set_data) in &set.index {
                    for &doc_id in set_data {
                        doc_ids.push(doc_id);
                    }
                }
                RegistryData::Set(doc_ids)
            },
            crate::index::Register::Map(map) => {
                let mut result = HashMap::new();
                for (_, map_data) in &map.index {
                    for (&doc_id, refs) in map_data {
                        let ref_data: Vec<IndexRefData> = refs.iter()
                            .map(|r| IndexRefData::from_index_ref(r))
                            .collect();
                        result.insert(doc_id, ref_data);
                    }
                }
                RegistryData::Map(result)
            }
        }
    }

    /// 导入索引数据
    pub fn import(&mut self, data: IndexExportData, _config: &SerializeConfig) -> Result<()> {
        // 验证版本兼容性
        if data.version != "0.1.0" {
            return Err(crate::error::InversearchError::Serialization(
                format!("Unsupported version: {}", data.version)
            ));
        }

        // 应用导入的配置
        self.apply_config(&data.config)?;

        // 清空当前索引
        self.clear();

        // 导入主索引
        self.import_main_index(&data.data.main_index)?;
        
        // 导入上下文索引
        self.import_context_index(&data.data.context_index)?;
        
        // 导入注册表
        self.import_registry(&data.data.registry)?;

        Ok(())
    }

    /// 导入主索引
    fn import_main_index(&mut self, data: &HashMap<String, Vec<u64>>) -> Result<()> {
        for (term, doc_ids) in data {
            let term_hash = self.keystore_hash_str(term);
            self.map.index.insert(term_hash, HashMap::new());
            if let Some(map) = self.map.index.get_mut(&term_hash) {
                map.insert(term.clone(), doc_ids.clone());
            }
        }
        Ok(())
    }

    /// 导入上下文索引
    fn import_context_index(&mut self, data: &HashMap<String, HashMap<String, Vec<u64>>>) -> Result<()> {
        for (ctx_key, ctx_data) in data {
            let ctx_hash = self.keystore_hash_str(ctx_key);
            self.ctx.index.insert(ctx_hash, HashMap::new());
            if let Some(ctx_map) = self.ctx.index.get_mut(&ctx_hash) {
                for (term, doc_ids) in ctx_data {
                    ctx_map.insert(term.clone(), doc_ids.clone());
                }
            }
        }
        Ok(())
    }

    /// 导入注册表
    fn import_registry(&mut self, data: &RegistryData) -> Result<()> {
        match data {
            RegistryData::Set(doc_ids) => {
                // 先收集所有需要处理的数据
                let mut items_to_insert = Vec::new();
                for &doc_id in doc_ids {
                    let doc_hash = self.keystore_hash(&doc_id.to_string());
                    items_to_insert.push((doc_hash, doc_id));
                }
                
                // 然后插入数据
                if let crate::index::Register::Set(set) = &mut self.reg {
                    for (doc_hash, doc_id) in items_to_insert {
                        set.index.entry(doc_hash).or_insert_with(std::collections::HashSet::new);
                        if let Some(set_data) = set.index.get_mut(&doc_hash) {
                            set_data.insert(doc_id);
                        }
                    }
                }
            },
            RegistryData::Map(doc_map) => {
                // 先收集所有需要处理的数据
                let mut items_to_insert = Vec::new();
                for (&doc_id, refs) in doc_map {
                    let doc_hash = self.keystore_hash(&doc_id.to_string());
                    let index_refs: Vec<crate::index::IndexRef> = refs.iter()
                        .map(|r| r.to_index_ref())
                        .collect();
                    items_to_insert.push((doc_hash, doc_id, index_refs));
                }
                
                // 然后插入数据
                if let crate::index::Register::Map(map) = &mut self.reg {
                    for (doc_hash, doc_id, index_refs) in items_to_insert {
                        map.index.entry(doc_hash).or_insert_with(HashMap::new);
                        if let Some(map_data) = map.index.get_mut(&doc_hash) {
                            map_data.insert(doc_id, index_refs);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// 获取上下文键字符串（辅助函数）
    fn get_ctx_key_string(&self, _key: &usize) -> Option<String> {
        // 这里需要实现从哈希到字符串的反向映射
        // 为了简化，暂时返回None
        None
    }

    /// 序列化为JSON字符串
    pub fn to_json(&self, config: &SerializeConfig) -> Result<String> {
        let data = self.export(config)?;
        Ok(serde_json::to_string_pretty(&data)?)
    }

    /// 从JSON字符串反序列化
    pub fn from_json(json_str: &str, config: &SerializeConfig) -> Result<Index> {
        let data: IndexExportData = serde_json::from_str(json_str)?;
        
        // 创建新的索引实例
        let mut index = Index::new(crate::index::IndexOptions::default())?;
        index.import(data, config)?;
        
        Ok(index)
    }

    /// 序列化为二进制格式
    pub fn to_binary(&self, config: &SerializeConfig) -> Result<Vec<u8>> {
        let data = self.export(config)?;
        
        if config.compression {
            // TODO: 实现压缩
            Ok(bincode::serialize(&data)?)
        } else {
            Ok(bincode::serialize(&data)?)
        }
    }

    /// 从二进制格式反序列化
    pub fn from_binary(binary_data: &[u8], config: &SerializeConfig) -> Result<Index> {
        let data: IndexExportData = if config.compression {
            // TODO: 实现解压缩
            bincode::deserialize(binary_data)?
        } else {
            bincode::deserialize(binary_data)?
        };
        
        // 创建新的索引实例
        let mut index = Index::new(crate::index::IndexOptions::default())?;
        index.import(data, config)?;
        
        Ok(index)
    }

    /// 获取索引选项
    fn get_index_options(&self) -> crate::r#type::IndexOptions {
        crate::r#type::IndexOptions {
            preset: None,
            context: Some(crate::r#type::ContextOptions {
                depth: Some(self.depth),
                bidirectional: Some(self.bidirectional),
                resolution: Some(self.resolution_ctx),
            }),
            encoder: None,
            resolution: Some(self.resolution),
            tokenize: Some(format!("{:?}", self.tokenize).to_lowercase()),
            fastupdate: Some(self.fastupdate),
            keystore: None,
            rtl: Some(self.rtl),
            cache: None,
            commit: None,
            priority: None,
        }
    }

    /// 获取分词器配置
    fn get_tokenizer_config(&self) -> TokenizerConfig {
        TokenizerConfig {
            mode: format!("{:?}", self.tokenize).to_lowercase(),
            separator: None,
            normalize: true,
        }
    }

    /// 应用导入的配置
    fn apply_config(&mut self, config: &IndexConfigExport) -> Result<()> {
        self.resolution = config.index_options.resolution.unwrap_or(9);
        self.resolution_ctx = config.index_options.context.as_ref()
            .and_then(|c| c.resolution)
            .unwrap_or(self.resolution);
        self.depth = config.index_options.context.as_ref()
            .and_then(|c| c.depth)
            .unwrap_or(0);
        self.bidirectional = config.index_options.context.as_ref()
            .and_then(|c| c.bidirectional)
            .unwrap_or(false);
        self.fastupdate = config.index_options.fastupdate.unwrap_or(false);
        self.rtl = config.index_options.rtl.unwrap_or(false);

        self.tokenize = match config.tokenizer_config.mode.as_str() {
            "strict" => crate::index::TokenizeMode::Strict,
            "forward" => crate::index::TokenizeMode::Forward,
            "reverse" => crate::index::TokenizeMode::Reverse,
            "full" => crate::index::TokenizeMode::Full,
            _ => crate::index::TokenizeMode::Strict,
        };

        Ok(())
    }
}

impl IndexRefData {
    fn from_index_ref(index_ref: &crate::index::IndexRef) -> Self {
        match index_ref {
            crate::index::IndexRef::MapRef(s) => IndexRefData::MapRef(s.clone()),
            crate::index::IndexRef::CtxRef(s1, s2) => IndexRefData::CtxRef(s1.clone(), s2.clone()),
        }
    }
    
    fn to_index_ref(&self) -> crate::index::IndexRef {
        match self {
            IndexRefData::MapRef(s) => crate::index::IndexRef::MapRef(s.clone()),
            IndexRefData::CtxRef(s1, s2) => crate::index::IndexRef::CtxRef(s1.clone(), s2.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Index;

    #[test]
    fn test_export_import_json() {
        // 创建测试索引
        let mut original_index = Index::default();
        original_index.add(1, "hello world", false).unwrap();
        original_index.add(2, "rust programming", false).unwrap();
        original_index.add(3, "hello rust", false).unwrap();

        // 导出为JSON
        let config = SerializeConfig::default();
        let json_str = original_index.to_json(&config).unwrap();
        
        // 从JSON导入
        let imported_index = Index::from_json(&json_str, &config).unwrap();
        
        // 验证导入结果
        let results = imported_index.search_simple("hello").unwrap();
        assert!(results.contains(&1));
        assert!(results.contains(&3));
        assert!(!results.contains(&2));
    }

    #[test]
    fn test_export_import_binary() {
        // 创建测试索引
        let mut original_index = Index::default();
        original_index.add(1, "test document", false).unwrap();
        original_index.add(2, "another test", false).unwrap();

        // 导出为二进制
        let config = SerializeConfig::default();
        let binary_data = original_index.to_binary(&config).unwrap();
        
        // 从二进制导入
        let imported_index = Index::from_binary(&binary_data, &config).unwrap();
        
        // 验证导入结果
        let results = imported_index.search_simple("test").unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.contains(&1));
        assert!(results.contains(&2));
    }

    #[test]
    fn test_serialize_config() {
        let config = SerializeConfig {
            format: SerializeFormat::Binary,
            compression: true,
            chunk_size: 500,
        };
        
        assert!(matches!(config.format, SerializeFormat::Binary));
        assert!(config.compression);
        assert_eq!(config.chunk_size, 500);
    }
}