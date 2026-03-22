//! Document 序列化模块
//!
//! 提供 Document 类型的导入导出功能

use crate::document::{Document, Field, TagSystem};
use crate::serialize::{SerializeConfig, SerializeFormat, IndexExportData, IndexInfo};
use crate::r#type::{DocId, IndexOptions};
use crate::error::Result;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use serde_json::Value;
use bincode;

/// Document 数据导出结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentExportData {
    pub version: String,
    pub created_at: String,
    pub document_info: DocumentInfo,
    pub fields: Vec<FieldExportData>,
    pub tags: Option<TagExportData>,
    pub store: Option<StoreExportData>,
    pub registry: RegistryExportData,
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
    pub tags: HashMap<String, Vec<DocId>>,
    pub config: Vec<String>,
}

/// 存储导出数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreExportData {
    pub documents: HashMap<DocId, Value>,
}

/// 注册表导出数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegistryExportData {
    Set(Vec<DocId>),
    Map(HashMap<DocId, ()>),
}

impl Document {
    /// 导出 Document 数据
    pub fn export(&self, config: &SerializeConfig) -> Result<DocumentExportData> {
        let document_info = DocumentInfo {
            field_count: self.fields.len(),
            fastupdate: self.fastupdate,
            store_enabled: self.store.is_some(),
            tag_enabled: self.tag_system.is_some(),
        };

        let mut fields = Vec::new();
        for field in &self.fields {
            let field_export = self.export_field(field, config)?;
            fields.push(field_export);
        }

        let tags = if let Some(ref tag_system) = self.tag_system {
            Some(self.export_tags(tag_system)?)
        } else {
            None
        };

        let store = if let Some(ref store) = self.store {
            Some(StoreExportData {
                documents: store.clone(),
            })
        } else {
            None
        };

        let registry = self.export_registry();

        Ok(DocumentExportData {
            version: "0.1.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            document_info,
            fields,
            tags,
            store,
            registry,
        })
    }

    /// 导出字段
    fn export_field(&self, field: &Field, config: &SerializeConfig) -> Result<FieldExportData> {
        let field_config = FieldConfigExport {
            field_type: "string".to_string(),
            index: true,
            optimize: false,
            resolution: field.index().resolution,
        };

        let index_data = field.index().export(config)?;

        Ok(FieldExportData {
            name: field.name().to_string(),
            field_config,
            index_data,
        })
    }

    /// 导出标签
    fn export_tags(&self, tag_system: &TagSystem) -> Result<TagExportData> {
        let tags = HashMap::new();
        let config: Vec<String> = tag_system.config_fields().iter().map(|s| s.to_string()).collect();

        Ok(TagExportData {
            tags,
            config,
        })
    }

    /// 导出注册表
    fn export_registry(&self) -> RegistryExportData {
        match &self.reg {
            crate::document::Register::Set(set) => {
                let mut doc_ids = Vec::new();
                for (_, set_data) in &set.index {
                    for &doc_id in set_data {
                        doc_ids.push(doc_id);
                    }
                }
                RegistryExportData::Set(doc_ids)
            },
            crate::document::Register::Map(map) => {
                let mut result = HashMap::new();
                for (&doc_id, _) in map {
                    result.insert(doc_id, ());
                }
                RegistryExportData::Map(result)
            }
        }
    }

    /// 导入 Document 数据
    pub fn import(&mut self, data: DocumentExportData, config: &SerializeConfig) -> Result<()> {
        // 验证版本兼容性
        if data.version != "0.1.0" {
            return Err(crate::error::InversearchError::Serialization(
                format!("Unsupported version: {}", data.version)
            ));
        }

        // 清空当前 Document
        self.clear();

        // 导入字段
        self.import_fields(&data.fields, config)?;

        // 导入标签
        if let Some(ref tags) = data.tags {
            self.import_tags(&tags)?;
        }

        // 导入存储
        if let Some(ref store) = data.store {
            self.import_store(&store)?;
        }

        // 导入注册表
        self.import_registry(&data.registry)?;

        Ok(())
    }

    /// 导入字段
    fn import_fields(&mut self, fields: &[FieldExportData], config: &SerializeConfig) -> Result<()> {
        for field_export in fields {
            if let Some(field) = self.field_mut(&field_export.name) {
                field.index_mut().import(field_export.index_data.clone(), config)?;
            }
        }
        Ok(())
    }

    /// 导入标签
    fn import_tags(&mut self, tags: &TagExportData) -> Result<()> {
        if let Some(ref mut tag_system) = self.tag_system {
            // 导入标签数据
            // TODO: 实现标签数据的导入
        }
        Ok(())
    }

    /// 导入存储
    fn import_store(&mut self, store: &StoreExportData) -> Result<()> {
        if let Some(ref mut store_data) = self.store {
            for (doc_id, value) in &store.documents {
                store_data.insert(*doc_id, value.clone());
            }
        }
        Ok(())
    }

    /// 导入注册表
    fn import_registry(&mut self, data: &RegistryExportData) -> Result<()> {
        match data {
            RegistryExportData::Set(doc_ids) => {
                if let crate::document::Register::Set(set) = &mut self.reg {
                    for &doc_id in doc_ids {
                        let doc_hash = crate::index::Index::keystore_hash_static(&doc_id.to_string());
                        set.index.entry(doc_hash).or_insert_with(std::collections::HashSet::new);
                        if let Some(set_data) = set.index.get_mut(&doc_hash) {
                            set_data.insert(doc_id);
                        }
                    }
                }
            },
            RegistryExportData::Map(map) => {
                if let crate::document::Register::Map(reg_map) = &mut self.reg {
                    for (&doc_id, _) in map {
                        reg_map.insert(doc_id, ());
                    }
                }
            }
        }
        Ok(())
    }

    /// 序列化为 JSON 字符串
    pub fn to_json(&self, config: &SerializeConfig) -> Result<String> {
        let data = self.export(config)?;
        Ok(serde_json::to_string_pretty(&data)?)
    }

    /// 从 JSON 字符串反序列化
    pub fn from_json(json_str: &str, config: &SerializeConfig) -> Result<Document> {
        let data: DocumentExportData = serde_json::from_str(json_str)?;
        
        // 根据导出数据创建 DocumentConfig
        let mut doc_config = crate::document::DocumentConfig::new();
        
        // 添加字段配置
        for field_data in &data.fields {
            let field_config = crate::document::FieldConfig::new(&field_data.name);
            doc_config = doc_config.add_field(field_config);
        }
        
        // 启用存储（如果导出数据包含存储）
        if data.store.is_some() {
            doc_config = doc_config.with_store();
        }
        
        // 创建新的 Document 实例
        let mut document = Document::new(doc_config)?;
        document.import(data, config)?;
        
        Ok(document)
    }

    /// 序列化为二进制数据（高性能）
    pub fn to_binary(&self, config: &SerializeConfig) -> Result<Vec<u8>> {
        let data = self.export(config)?;
        let serialized = bincode::serialize(&data)?;
        Ok(serialized)
    }

    /// 从二进制数据反序列化
    pub fn from_binary(data: &[u8], config: &SerializeConfig) -> Result<Document> {
        let data: DocumentExportData = bincode::deserialize(data)?;
        
        // 根据导出数据创建 DocumentConfig
        let mut doc_config = crate::document::DocumentConfig::new();
        
        // 添加字段配置
        for field_data in &data.fields {
            let field_config = crate::document::FieldConfig::new(&field_data.name);
            doc_config = doc_config.add_field(field_config);
        }
        
        // 启用存储（如果导出数据包含存储）
        if data.store.is_some() {
            doc_config = doc_config.with_store();
        }
        
        // 创建新的 Document 实例
        let mut document = Document::new(doc_config)?;
        document.import(data, config)?;
        
        Ok(document)
    }

    /// 获取字段的可变引用
    fn field_mut(&mut self, name: &str) -> Option<&mut Field> {
        if let Some(&idx) = self.name_to_index.get(name) {
            self.fields.get_mut(idx)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{DocumentConfig, FieldConfig};
    use serde_json::json;

    #[test]
    fn test_document_export_import() {
        let config = DocumentConfig::new()
            .add_field(FieldConfig::new("title"))
            .add_field(FieldConfig::new("content"))
            .with_store();

        let mut document = Document::new(config).unwrap();
        document.add(1, &json!({"title": "Hello World", "content": "Test content"})).unwrap();
        document.add(2, &json!({"title": "Rust Programming", "content": "Another test"})).unwrap();

        // 导出为 JSON
        let serialize_config = SerializeConfig::default();
        let json_str = document.to_json(&serialize_config).unwrap();
        
        // 从 JSON 导入
        let imported_document = Document::from_json(&json_str, &serialize_config).unwrap();
        
        // 验证导入结果
        assert!(imported_document.contains(1));
        assert!(imported_document.contains(2));
        
        // 验证存储的文档
        let doc1 = imported_document.get(1);
        assert!(doc1.is_some());
        assert_eq!(doc1.unwrap()["title"], "Hello World");
    }

    #[test]
    fn test_document_export_info() {
        let config = DocumentConfig::new()
            .add_field(FieldConfig::new("title"))
            .with_store();

        let document = Document::new(config).unwrap();
        let serialize_config = SerializeConfig::default();
        let export_data = document.export(&serialize_config).unwrap();

        assert_eq!(export_data.document_info.field_count, 1);
        assert!(export_data.document_info.store_enabled);
        assert!(!export_data.document_info.tag_enabled);
    }

    #[test]
    fn test_document_binary_export_import() {
        let config = DocumentConfig::new()
            .add_field(FieldConfig::new("title"))
            .add_field(FieldConfig::new("content"))
            .with_store();

        let mut document = Document::new(config).unwrap();
        document.add(1, &json!({"title": "Hello World", "content": "Test content"})).unwrap();
        document.add(2, &json!({"title": "Rust Programming", "content": "Another test"})).unwrap();

        // 导出为二进制
        let serialize_config = SerializeConfig::default();
        let binary_data = document.to_binary(&serialize_config).unwrap();
        
        // 验证二进制数据比 JSON 更紧凑
        let json_str = document.to_json(&serialize_config).unwrap();
        assert!(binary_data.len() < json_str.len());
        
        // 从二进制导入
        let imported_document = Document::from_binary(&binary_data, &serialize_config).unwrap();
        
        // 验证导入结果
        assert!(imported_document.contains(1));
        assert!(imported_document.contains(2));
        
        // 验证存储的文档
        let doc1 = imported_document.get(1);
        assert!(doc1.is_some());
        assert_eq!(doc1.unwrap()["title"], "Hello World");
    }

    #[test]
    fn test_document_binary_empty() {
        let config = DocumentConfig::new()
            .add_field(FieldConfig::new("title"));

        let document = Document::new(config).unwrap();
        let serialize_config = SerializeConfig::default();
        
        // 导出空文档为二进制
        let binary_data = document.to_binary(&serialize_config).unwrap();
        assert!(!binary_data.is_empty());
        
        // 从二进制导入
        let imported_document = Document::from_binary(&binary_data, &serialize_config).unwrap();
        assert!(!imported_document.contains(1));
    }
}
