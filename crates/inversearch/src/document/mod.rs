//! Document 模块
//!
//! 提供多字段文档索引的统一管理
//!
//! # 模块结构
//!
//! - `mod.rs`: Document 主结构和公共接口
//! - `field.rs`: 字段定义和配置
//! - `tree.rs`: 树形结构解析
//! - `tag.rs`: 标签系统
//! - `batch.rs`: 批量操作

mod field;
mod tree;
mod tag;
mod batch;

use crate::search;

pub use field::{Field, FieldConfig, FieldType, Fields};
pub use tree::{parse_tree, parse_tree_cached, TreePath, PathCache, EvaluationStrategy, PathParseError};
pub use tag::{TagSystem, TagConfig};
pub use batch::{Batch, BatchOperation, BatchExecutor, BatchExecutorFn, BatchResult, BatchStatus, BatchMetadata};

// 从 serialize 模块导出 Document 序列化相关类型
pub use crate::serialize::types::{
    DocumentExportData, DocumentInfo, FieldExportData, FieldConfigExport,
    TagExportData, TagConfigExport, StoreExportData, DocumentRegistryData,
};

use crate::{
    SearchOptions, SearchResult,
    DocId,
    error::{Result, InversearchError as Error},
    keystore::KeystoreSet,
};
use serde_json::Value;
use std::collections::HashMap;

/// 文档搜索引擎主结构
pub struct Document {
    fields: Vec<Field>,
    name_to_index: HashMap<String, usize>,
    tag_system: Option<TagSystem>,
    store: Option<HashMap<DocId, Value>>,
    reg: Register,
    #[allow(dead_code)]
    fastupdate: bool,
}

/// 注册表类型
#[derive(Debug, Clone)]
pub enum Register {
    Set(KeystoreSet<DocId>),
    Map(HashMap<DocId, ()>),
}

impl Document {
    /// 创建新的 Document 实例
    pub fn new(config: DocumentConfig) -> Result<Self> {
        let mut fields: Vec<Field> = Vec::new();
        let mut name_to_index = HashMap::new();
        
        for field_config in config.fields.into_iter() {
            let name = field_config.name.clone();
            if name_to_index.contains_key(&name) {
                return Err(Error::DuplicateFieldName(name.clone(), name_to_index[&name]));
            }
            let field = Field::new(field_config)?;
            name_to_index.insert(name, fields.len());
            fields.push(field);
        }
        
        let tag_system = if config.tags.is_empty() {
            None
        } else {
            let mut ts = TagSystem::new();
            for (field, filter) in config.tags {
                ts.add_config_str(field, filter);
            }
            Some(ts)
        };
        
        let store = if config.store {
            Some(HashMap::new())
        } else {
            None
        };
        
        let reg = if config.fastupdate {
            Register::Map(HashMap::new())
        } else {
            Register::Set(KeystoreSet::new(8))
        };
        
        Ok(Document {
            fields,
            name_to_index,
            tag_system,
            store,
            reg,
            fastupdate: config.fastupdate,
        })
    }

    /// 添加文档
    pub fn add(&mut self, id: DocId, content: &Value) -> Result<()> {
        for field in &mut self.fields {
            field.add(id, content)?;
        }
        
        if let Some(ref mut tag_system) = self.tag_system {
            let config_fields = tag_system.config_fields();
            let tags: Vec<(String, Value)> = config_fields.iter()
                .filter_map(|field_name| {
                    extract_simple(content, field_name).map(|v| (field_name.to_string(), v))
                })
                .collect();
            let tags_refs: Vec<(&str, &Value)> = tags.iter()
                .map(|(s, v)| (s.as_str(), v))
                .collect();
            tag_system.add_tags(id, &tags_refs);
        }
        
        match &mut self.reg {
            Register::Set(set) => { set.add(id); }
            Register::Map(map) => { map.insert(id, ()); }
        }
        
        if let Some(ref mut store) = self.store {
            store.insert(id, content.clone());
        }
        
        Ok(())
    }

    /// 更新文档
    pub fn update(&mut self, id: DocId, content: &Value) -> Result<()> {
        self.remove(id)?;
        self.add(id, content)?;
        Ok(())
    }

    /// 删除文档
    pub fn remove(&mut self, id: DocId) -> Result<()> {
        for field in &mut self.fields {
            field.remove(id)?;
        }
        
        if let Some(ref mut tag_system) = self.tag_system {
            tag_system.remove_tags(id);
        }
        
        if let Some(ref mut store) = self.store {
            store.remove(&id);
        }
        
        match &mut self.reg {
            Register::Set(set) => { set.delete(&id); }
            Register::Map(map) => { map.remove(&id); }
        }
        
        Ok(())
    }

    /// 搜索
    pub fn search(&self, options: &SearchOptions) -> Result<SearchResult> {
        let query = options.query.as_deref().unwrap_or("");
        if query.is_empty() {
            return Ok(SearchResult {
                results: Vec::new(),
                total: 0,
                query: String::new(),
            });
        }
        
        let limit = options.limit.unwrap_or(100);
        let offset = options.offset.unwrap_or(0);
        
        let mut all_results: Vec<Vec<DocId>> = Vec::new();
        
        for field in &self.fields {
            let field_options = SearchOptions {
                query: options.query.clone(),
                limit: Some(limit),
                offset: Some(0),
                context: options.context,
                resolve: options.resolve,
                ..Default::default()
            };
            
            let result = search::search(field.index(), &field_options)?;
            if !result.results.is_empty() {
                all_results.push(result.results);
            }
        }
        
        let mut final_results: Vec<DocId> = Vec::new();
        for vec in all_results {
            final_results.extend(vec);
        }
        
        final_results.sort();
        final_results.dedup();
        
        let total = final_results.len();
        
        if offset > 0 && offset < final_results.len() {
            final_results.drain(0..offset);
        }
        
        if limit > 0 && limit < final_results.len() {
            final_results.truncate(limit);
        }
        
        Ok(SearchResult {
            results: final_results,
            total,
            query: query.to_string(),
        })
    }

    /// 获取文档
    pub fn get(&self, id: DocId) -> Option<&Value> {
        self.store.as_ref()?.get(&id)
    }

    /// 检查文档是否存在
    pub fn contains(&self, id: DocId) -> bool {
        match &self.reg {
            Register::Set(set) => set.has(&id),
            Register::Map(map) => map.contains_key(&id),
        }
    }

    /// 获取store的引用（用于序列化）
    pub fn get_store(&self) -> Option<&HashMap<DocId, Value>> {
        self.store.as_ref()
    }

    /// 获取store的可变引用（用于序列化）
    pub fn get_store_mut(&mut self) -> Option<&mut HashMap<DocId, Value>> {
        self.store.as_mut()
    }

    /// 获取register的引用（用于序列化）
    pub fn get_reg(&self) -> &Register {
        &self.reg
    }

    /// 获取register的可变引用（用于序列化）
    pub fn get_reg_mut(&mut self) -> &mut Register {
        &mut self.reg
    }

    /// 检查是否启用了store
    pub fn has_store(&self) -> bool {
        self.store.is_some()
    }

    /// 检查是否启用了tag system
    pub fn has_tag_system(&self) -> bool {
        self.tag_system.is_some()
    }

    /// 检查是否启用了fastupdate
    pub fn is_fastupdate(&self) -> bool {
        matches!(self.reg, Register::Map(_))
    }

    /// 清空所有索引
    pub fn clear(&mut self) {
        for field in &mut self.fields {
            field.clear();
        }
        if let Some(ref mut tag_system) = self.tag_system {
            tag_system.clear();
        }
        if let Some(ref mut store) = self.store {
            store.clear();
        }
        match &mut self.reg {
            Register::Set(set) => { set.clear(); }
            Register::Map(map) => { map.clear(); }
        }
    }

    /// 获取字段数量
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// 获取所有字段名
    pub fn field_names(&self) -> Vec<&str> {
        self.name_to_index.keys().map(|s| s.as_str()).collect()
    }

    /// 获取字段引用
    pub fn field(&self, name: &str) -> Option<&Field> {
        self.name_to_index.get(name).map(|&idx| &self.fields[idx])
    }

    /// 获取可变字段引用（内部使用）
    pub fn field_mut(&mut self, name: &str) -> Option<&mut Field> {
        self.name_to_index.get(name).copied().map(|idx| &mut self.fields[idx])
    }

    /// 执行批量操作
    pub fn execute_batch(&mut self, batch: &crate::document::Batch) -> crate::document::BatchResult {
        let executor = crate::document::BatchExecutor::new(0);
        executor.execute_batch_mixed(batch.operations(), self)
    }

    /// 批量添加文档
    pub fn batch_add(&mut self, operations: &[(DocId, &Value)]) -> crate::document::BatchResult {
        let executor = crate::document::BatchExecutor::new(0);
        executor.execute_batch_add(operations, self)
    }

    /// 批量更新文档
    pub fn batch_update(&mut self, operations: &[(DocId, &Value)]) -> crate::document::BatchResult {
        let executor = crate::document::BatchExecutor::new(0);
        executor.execute_batch_update(operations, self)
    }

    /// 批量删除文档
    pub fn batch_remove(&mut self, operations: &[DocId]) -> crate::document::BatchResult {
        let executor = crate::document::BatchExecutor::new(0);
        executor.execute_batch_remove(operations, self)
    }
}

/// 文档配置
#[derive(Default)]
pub struct DocumentConfig {
    pub fields: Vec<FieldConfig>,
    pub tags: Vec<(String, Option<Box<dyn Fn(&Value) -> bool + Send + Sync>>)>,
    pub store: bool,
    pub fastupdate: bool,
    pub cache: Option<usize>,
}

impl DocumentConfig {
    /// 创建新的配置
    pub fn new() -> Self {
        DocumentConfig {
            fields: Vec::new(),
            tags: Vec::new(),
            store: false,
            fastupdate: false,
            cache: None,
        }
    }

    /// 添加字段
    pub fn add_field(mut self, field: FieldConfig) -> Self {
        self.fields.push(field);
        self
    }

    /// 添加标签配置（字符串形式）
    pub fn add_tag(mut self, field: &str) -> Self {
        self.tags.push((field.to_string(), None));
        self
    }

    /// 添加带过滤器的标签配置
    pub fn add_tag_with_filter(mut self, field: &str, filter: Box<dyn Fn(&Value) -> bool + Send + Sync>) -> Self {
        self.tags.push((field.to_string(), Some(filter)));
        self
    }

    /// 启用文档存储
    pub fn with_store(mut self) -> Self {
        self.store = true;
        self
    }

    /// 启用快速更新
    pub fn with_fastupdate(mut self) -> Self {
        self.fastupdate = true;
        self
    }

    /// 设置缓存大小
    pub fn with_cache(mut self, size: usize) -> Self {
        self.cache = Some(size);
        self
    }
}

/// 从文档中提取简单路径的值
fn extract_simple(document: &Value, path: &str) -> Option<Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = document;
    
    for part in parts {
        current = current.get(part)?;
    }
    
    Some(current.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_document_new() {
        let config = DocumentConfig::new()
            .add_field(FieldConfig::new("title"))
            .add_field(FieldConfig::new("content"));
        
        let doc = Document::new(config).unwrap();
        assert_eq!(doc.len(), 2);
    }

    #[test]
    fn test_document_add() {
        let config = DocumentConfig::new()
            .add_field(FieldConfig::new("title"));
        
        let mut doc = Document::new(config).unwrap();
        doc.add(1, &json!({"title": "Hello World"})).unwrap();
        
        assert!(doc.contains(1));
    }

    #[test]
    fn test_document_search() {
        let config = DocumentConfig::new()
            .add_field(FieldConfig::new("title"));
        
        let mut doc = Document::new(config).unwrap();
        doc.add(1, &json!({"title": "Hello World"})).unwrap();
        doc.add(2, &json!({"title": "Rust Programming"})).unwrap();
        
        let result = doc.search(&SearchOptions {
            query: Some("Hello".to_string()),
            limit: Some(10),
            ..Default::default()
        }).unwrap();
        
        assert_eq!(result.results.len(), 1);
        assert_eq!(result.results[0], 1);
    }

    #[test]
    fn test_document_remove() {
        let config = DocumentConfig::new()
            .add_field(FieldConfig::new("title"));
        
        let mut doc = Document::new(config).unwrap();
        doc.add(1, &json!({"title": "Hello"})).unwrap();
        
        assert!(doc.contains(1));
        
        doc.remove(1).unwrap();
        
        assert!(!doc.contains(1));
    }

    #[test]
    fn test_document_update() {
        let config = DocumentConfig::new()
            .add_field(FieldConfig::new("title"))
            .with_store();
        
        let mut doc = Document::new(config).unwrap();
        doc.add(1, &json!({"title": "Original"})).unwrap();
        
        doc.update(1, &json!({"title": "Updated"})).unwrap();
        
        let stored = doc.get(1);
        assert_eq!(stored.unwrap()["title"], "Updated");
    }

    #[test]
    fn test_document_clear() {
        let config = DocumentConfig::new()
            .add_field(FieldConfig::new("title"));
        
        let mut doc = Document::new(config).unwrap();
        doc.add(1, &json!({"title": "Doc 1"})).unwrap();
        doc.add(2, &json!({"title": "Doc 2"})).unwrap();
        
        doc.clear();
        
        assert!(!doc.contains(1));
        assert!(!doc.contains(2));
    }

    #[test]
    fn test_document_batch() {
        let config = DocumentConfig::new()
            .add_field(FieldConfig::new("title"));
        
        let mut doc = Document::new(config).unwrap();
        
        let mut batch = Batch::new(100);
        let doc1 = json!({"title": "Doc 1"});
        let doc2 = json!({"title": "Doc 2"});
        let doc3 = json!({"title": "Doc 3"});
        batch.add(1, &doc1);
        batch.add(2, &doc2);
        batch.add(3, &doc3);
        
        let result = doc.execute_batch(&batch);
        
        assert_eq!(result.total_operations, 3);
        assert_eq!(result.successful_operations, 3);
        assert_eq!(result.failed_operations, 0);
        assert!(doc.contains(1));
        assert!(doc.contains(2));
        assert!(doc.contains(3));
    }

    #[test]
    fn test_document_batch_add() {
        let config = DocumentConfig::new()
            .add_field(FieldConfig::new("title"));
        
        let mut doc = Document::new(config).unwrap();
        
        let doc1 = json!({"title": "Doc 1"});
        let doc2 = json!({"title": "Doc 2"});
        let doc3 = json!({"title": "Doc 3"});
        let operations = vec![
            (1, &doc1),
            (2, &doc2),
            (3, &doc3),
        ];
        
        let result = doc.batch_add(&operations);
        
        assert_eq!(result.total_operations, 3);
        assert_eq!(result.successful_operations, 3);
        assert_eq!(result.failed_operations, 0);
        assert!(doc.contains(1));
        assert!(doc.contains(2));
        assert!(doc.contains(3));
    }

    #[test]
    fn test_document_batch_update() {
        let config = DocumentConfig::new()
            .add_field(FieldConfig::new("title"))
            .with_store();
        
        let mut doc = Document::new(config).unwrap();
        doc.add(1, &json!({"title": "Original"})).unwrap();
        doc.add(2, &json!({"title": "Original"})).unwrap();
        
        let doc1 = json!({"title": "Updated 1"});
        let doc2 = json!({"title": "Updated 2"});
        let operations = vec![
            (1, &doc1),
            (2, &doc2),
        ];
        
        let result = doc.batch_update(&operations);
        
        assert_eq!(result.total_operations, 2);
        assert_eq!(result.successful_operations, 2);
        assert_eq!(result.failed_operations, 0);
        
        let stored1 = doc.get(1);
        let stored2 = doc.get(2);
        assert_eq!(stored1.unwrap()["title"], "Updated 1");
        assert_eq!(stored2.unwrap()["title"], "Updated 2");
    }

    #[test]
    fn test_document_batch_remove() {
        let config = DocumentConfig::new()
            .add_field(FieldConfig::new("title"));
        
        let mut doc = Document::new(config).unwrap();
        doc.add(1, &json!({"title": "Doc 1"})).unwrap();
        doc.add(2, &json!({"title": "Doc 2"})).unwrap();
        doc.add(3, &json!({"title": "Doc 3"})).unwrap();
        
        let operations = vec![1, 2];
        let result = doc.batch_remove(&operations);
        
        assert_eq!(result.total_operations, 2);
        assert_eq!(result.successful_operations, 2);
        assert_eq!(result.failed_operations, 0);
        assert!(!doc.contains(1));
        assert!(!doc.contains(2));
        assert!(doc.contains(3));
    }

    #[test]
    fn test_document_batch_mixed_operations() {
        let config = DocumentConfig::new()
            .add_field(FieldConfig::new("title"));
        
        let mut doc = Document::new(config).unwrap();
        doc.add(1, &json!({"title": "Doc 1"})).unwrap();
        doc.add(2, &json!({"title": "Doc 2"})).unwrap();
        
        let mut batch = Batch::new(100);
        let doc3 = json!({"title": "Doc 3"});
        let doc1_updated = json!({"title": "Updated Doc 1"});
        batch.add(3, &doc3);
        batch.update(1, &doc1_updated);
        batch.remove(2);
        
        let result = doc.execute_batch(&batch);
        
        assert_eq!(result.total_operations, 3);
        assert_eq!(result.successful_operations, 3);
        assert_eq!(result.failed_operations, 0);
        assert!(doc.contains(1));
        assert!(!doc.contains(2));
        assert!(doc.contains(3));
    }
}
