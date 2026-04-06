//! 序列化类型定义模块
//!
//! 提供所有序列化相关的核心类型定义，作为数据的唯一来源

use crate::r#type::{EncoderOptions, IndexOptions};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 序列化配置
#[derive(Debug, Clone)]
pub struct SerializeConfig {
    pub format: SerializeFormat,
    pub compression: bool,
    pub compression_algorithm: CompressionAlgorithm,
    pub compression_level: i32,
    pub chunk_size: usize,
    pub enable_incremental: bool,
}

impl Default for SerializeConfig {
    fn default() -> Self {
        Self {
            format: SerializeFormat::MessagePack,
            compression: true,
            compression_algorithm: CompressionAlgorithm::Zstd,
            compression_level: 3,
            chunk_size: 1000,
            enable_incremental: true,
        }
    }
}

impl SerializeConfig {
    /// 创建带压缩的配置
    pub fn with_compression(algorithm: CompressionAlgorithm, level: i32) -> Self {
        Self {
            format: SerializeFormat::MessagePack,
            compression: true,
            compression_algorithm: algorithm,
            compression_level: level,
            chunk_size: 1000,
            enable_incremental: true,
        }
    }

    /// 创建快速配置（低延迟，适合频繁操作）
    pub fn fast() -> Self {
        Self {
            format: SerializeFormat::MessagePack,
            compression: true,
            compression_algorithm: CompressionAlgorithm::Lz4,
            compression_level: 1,
            chunk_size: 5000,
            enable_incremental: true,
        }
    }

    /// 创建紧凑配置（高压缩比，适合存储）
    pub fn compact() -> Self {
        Self {
            format: SerializeFormat::MessagePack,
            compression: true,
            compression_algorithm: CompressionAlgorithm::Zstd,
            compression_level: 19,
            chunk_size: 1000,
            enable_incremental: false,
        }
    }
}

/// 序列化格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializeFormat {
    Json,
    Binary,
    MessagePack,
    Cbor,
}

/// 压缩算法
#[derive(Debug, Clone, Copy)]
pub enum CompressionAlgorithm {
    None,
    Zstd,
    Lz4,
}

impl CompressionAlgorithm {
    /// 获取算法名称
    pub fn name(&self) -> &'static str {
        match self {
            CompressionAlgorithm::None => "none",
            CompressionAlgorithm::Zstd => "zstd",
            CompressionAlgorithm::Lz4 => "lz4",
        }
    }
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
    Set(Vec<u64>),                        // fastupdate = false
    Map(HashMap<u64, Vec<IndexRefData>>), // fastupdate = true
}

/// 索引引用数据（序列化格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexRefData {
    MapRef(String),
    CtxRef(String, String),
}

impl IndexRefData {
    pub fn from_index_ref(index_ref: &crate::index::IndexRef) -> Self {
        match index_ref {
            crate::index::IndexRef::MapRef(s) => IndexRefData::MapRef(s.clone()),
            crate::index::IndexRef::CtxRef(s1, s2) => IndexRefData::CtxRef(s1.clone(), s2.clone()),
        }
    }

    pub fn to_index_ref(&self) -> crate::index::IndexRef {
        match self {
            IndexRefData::MapRef(s) => crate::index::IndexRef::MapRef(s.clone()),
            IndexRefData::CtxRef(s1, s2) => crate::index::IndexRef::CtxRef(s1.clone(), s2.clone()),
        }
    }
}

/// 增量序列化数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalData {
    pub version: String,
    pub timestamp: String,
    pub changes: Vec<IndexChange>,
    pub base_snapshot: Option<String>,
}

/// 索引变更（用于增量序列化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexChange {
    Add { doc_id: u64, content: String },
    Remove { doc_id: u64 },
    Update { doc_id: u64, content: String },
}

/// 索引配置导出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfigExport {
    pub index_options: IndexOptions,
    pub encoder_options: EncoderOptions,
    pub tokenizer_config: TokenizerConfig,
}

/// 分词器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenizerConfig {
    pub mode: String,
    pub separator: Option<String>,
    pub normalize: bool,
}

/// Document 数据导出结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentExportData {
    pub version: String,
    pub created_at: String,
    pub document_info: DocumentInfo,
    pub fields: Vec<FieldExportData>,
    pub tags: Option<TagExportData>,
    pub store: Option<StoreExportData>,
    pub registry: DocumentRegistryData,
}

/// Document 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentInfo {
    pub field_count: usize,
    pub fastupdate: bool,
    pub store_enabled: bool,
    pub tag_enabled: bool,
}

/// 字段导出数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldExportData {
    pub name: String,
    pub field_config: FieldConfigExport,
    pub index_data: IndexExportData,
}

/// 字段配置导出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldConfigExport {
    pub field_type: String,
    pub index: bool,
    pub optimize: bool,
    pub resolution: usize,
}

/// 标签导出数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagExportData {
    pub tags: HashMap<String, Vec<u64>>,
    pub config: TagConfigExport,
}

/// 标签配置导出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagConfigExport {
    pub enabled: bool,
    pub case_sensitive: bool,
}

/// 存储导出数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreExportData {
    pub documents: HashMap<u64, String>,
    pub enabled: bool,
}

/// Document 注册表数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRegistryData {
    pub doc_count: usize,
    pub next_doc_id: u64,
}

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

impl IndexExportData {
    /// 从索引创建导出数据
    pub fn from_index(index: &crate::Index) -> crate::error::Result<Self> {
        use chrono::Utc;
        
        Ok(Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: Utc::now().to_rfc3339(),
            index_info: IndexInfo {
                resolution: index.resolution,
                resolution_ctx: index.resolution_ctx,
                tokenize_mode: format!("{:?}", index.tokenize),
                depth: index.depth,
                bidirectional: index.bidirectional,
                fastupdate: index.fastupdate,
                rtl: index.rtl,
                encoder_type: "default".to_string(),
            },
            config: IndexConfigExport {
                index_options: IndexOptions::default(),
                encoder_options: EncoderOptions::default(),
                tokenizer_config: TokenizerConfig {
                    mode: "default".to_string(),
                    separator: None,
                    normalize: true,
                },
            },
            data: ExportData {
                main_index: std::collections::HashMap::new(),
                context_index: std::collections::HashMap::new(),
                registry: RegistryData::Set(Vec::new()),
            },
        })
    }

    /// 将导出数据应用到索引
    pub fn apply_to_index(&self, _index: &mut crate::Index) -> crate::error::Result<()> {
        // TODO: 实现数据恢复逻辑
        Ok(())
    }
}
