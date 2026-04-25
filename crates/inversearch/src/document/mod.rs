//! Document Module
//!
//! Provide unified management of multi-field document indexing
//!
//! # Module Structure
//!
//! - `mod.rs`: Document 主结构和公共接口
//! - `field.rs`: 字段定义和配置
//! - `tree.rs`: 树形结构解析
//! - `tag.rs`: 标签系统
//! - `batch.rs`: 批量操作

mod batch;
mod field;
mod tag;
pub mod tree;

use crate::search;

pub use batch::{
    Batch, BatchExecutor, BatchExecutorFn, BatchMetadata, BatchOperation, BatchResult, BatchStatus,
};
pub use field::{Field, FieldConfig, FieldType, Fields};
pub use tag::{TagConfig, TagSystem};
pub use tree::{
    parse_tree, parse_tree_cached, EvaluationStrategy, PathCache, PathParseError, TreePath,
};

// Exporting Document serialization-related types from the serialize module
pub use crate::serialize::types::{
    DocumentExportData, DocumentInfo, DocumentRegistryData, FieldConfigExport, FieldExportData,
    StoreExportData, TagConfigExport, TagExportData,
};

use crate::{
    error::{InversearchError as Error, Result},
    keystore::KeystoreSet,
    DocId, SearchOptions, SearchResult,
};
use serde_json::Value;
use std::collections::HashMap;

type TagFilterFn = Box<dyn Fn(&Value) -> bool + Send + Sync>;

/// Document search engine main structure
pub struct Document {
    fields: Vec<Field>,
    name_to_index: HashMap<String, usize>,
    tag_system: Option<TagSystem>,
    store: Option<HashMap<DocId, Value>>,
    reg: Register,
    #[allow(dead_code)]
    fastupdate: bool,
}

/// Registry Type
#[derive(Debug, Clone)]
pub enum Register {
    Set(KeystoreSet<DocId>),
    Map(HashMap<DocId, ()>),
}

impl Document {
    /// Creating a New Document Instance
    pub fn new(config: DocumentConfig) -> Result<Self> {
        let mut fields: Vec<Field> = Vec::new();
        let mut name_to_index = HashMap::new();

        for field_config in config.fields.into_iter() {
            let name = field_config.name.clone();
            if name_to_index.contains_key(&name) {
                return Err(Error::DuplicateFieldName(
                    name.clone(),
                    name_to_index[&name],
                ));
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

    /// Adding Documents
    pub fn add(&mut self, id: DocId, content: &Value) -> Result<()> {
        for field in &mut self.fields {
            field.add(id, content)?;
        }

        if let Some(ref mut tag_system) = self.tag_system {
            let config_fields = tag_system.config_fields();
            let tags: Vec<(String, Value)> = config_fields
                .iter()
                .filter_map(|field_name| {
                    extract_simple(content, field_name).map(|v| (field_name.to_string(), v))
                })
                .collect();
            let tags_refs: Vec<(&str, &Value)> =
                tags.iter().map(|(s, v)| (s.as_str(), v)).collect();
            tag_system.add_tags(id, &tags_refs);
        }

        match &mut self.reg {
            Register::Set(set) => {
                set.add(id);
            }
            Register::Map(map) => {
                map.insert(id, ());
            }
        }

        if let Some(ref mut store) = self.store {
            store.insert(id, content.clone());
        }

        Ok(())
    }

    /// Update Documentation
    pub fn update(&mut self, id: DocId, content: &Value) -> Result<()> {
        self.remove(id)?;
        self.add(id, content)?;
        Ok(())
    }

    /// Delete Document
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
            Register::Set(set) => {
                set.delete(&id);
            }
            Register::Map(map) => {
                map.remove(&id);
            }
        }

        Ok(())
    }

    /// look for sth.
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

    /// Get Documentation
    pub fn get(&self, id: DocId) -> Option<&Value> {
        self.store.as_ref()?.get(&id)
    }

    /// Check if the document exists
    pub fn contains(&self, id: DocId) -> bool {
        match &self.reg {
            Register::Set(set) => set.has(&id),
            Register::Map(map) => map.contains_key(&id),
        }
    }

    /// Getting a reference to store (for serialization)
    pub fn get_store(&self) -> Option<&HashMap<DocId, Value>> {
        self.store.as_ref()
    }

    /// Getting a mutable reference to store (for serialization)
    pub fn get_store_mut(&mut self) -> Option<&mut HashMap<DocId, Value>> {
        self.store.as_mut()
    }

    /// Get a reference to register (for serialization)
    pub fn get_reg(&self) -> &Register {
        &self.reg
    }

    /// Get a mutable reference to register (for serialization)
    pub fn get_reg_mut(&mut self) -> &mut Register {
        &mut self.reg
    }

    /// Check if store is enabled
    pub fn has_store(&self) -> bool {
        self.store.is_some()
    }

    /// Check if tag system is enabled
    pub fn has_tag_system(&self) -> bool {
        self.tag_system.is_some()
    }

    /// Check if fastupdate is enabled
    pub fn is_fastupdate(&self) -> bool {
        matches!(self.reg, Register::Map(_))
    }

    /// Empty all indexes
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
            Register::Set(set) => {
                set.clear();
            }
            Register::Map(map) => {
                map.clear();
            }
        }
    }

    /// Get the number of fields
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Check if it is empty
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Get all field names
    pub fn field_names(&self) -> Vec<&str> {
        self.name_to_index.keys().map(|s| s.as_str()).collect()
    }

    /// Getting field references
    pub fn field(&self, name: &str) -> Option<&Field> {
        self.name_to_index.get(name).map(|&idx| &self.fields[idx])
    }

    /// Getting variable field references (internal use)
    pub fn field_mut(&mut self, name: &str) -> Option<&mut Field> {
        self.name_to_index
            .get(name)
            .copied()
            .map(|idx| &mut self.fields[idx])
    }

    /// Perform batch operations
    pub fn execute_batch(
        &mut self,
        batch: &crate::document::Batch,
    ) -> crate::document::BatchResult {
        let executor = crate::document::BatchExecutor::new(0);
        executor.execute_batch_mixed(batch.operations(), self)
    }

    /// Add documents in bulk
    pub fn batch_add(&mut self, operations: &[(DocId, &Value)]) -> crate::document::BatchResult {
        let executor = crate::document::BatchExecutor::new(0);
        executor.execute_batch_add(operations, self)
    }

    /// Batch update documents
    pub fn batch_update(&mut self, operations: &[(DocId, &Value)]) -> crate::document::BatchResult {
        let executor = crate::document::BatchExecutor::new(0);
        executor.execute_batch_update(operations, self)
    }

    /// Batch Delete Documents
    pub fn batch_remove(&mut self, operations: &[DocId]) -> crate::document::BatchResult {
        let executor = crate::document::BatchExecutor::new(0);
        executor.execute_batch_remove(operations, self)
    }
}

/// Document Configuration
#[derive(Default)]
pub struct DocumentConfig {
    pub fields: Vec<FieldConfig>,
    pub tags: Vec<(String, Option<TagFilterFn>)>,
    pub store: bool,
    pub fastupdate: bool,
    pub cache: Option<usize>,
}

impl DocumentConfig {
    /// Creating a new configuration
    pub fn new() -> Self {
        DocumentConfig {
            fields: Vec::new(),
            tags: Vec::new(),
            store: false,
            fastupdate: false,
            cache: None,
        }
    }

    /// Adding Fields
    pub fn add_field(mut self, field: FieldConfig) -> Self {
        self.fields.push(field);
        self
    }

    /// Add label configuration (in string form)
    pub fn add_tag(mut self, field: &str) -> Self {
        self.tags.push((field.to_string(), None));
        self
    }

    /// Adding Label Configuration with Filters
    pub fn add_tag_with_filter(
        mut self,
        field: &str,
        filter: Box<dyn Fn(&Value) -> bool + Send + Sync>,
    ) -> Self {
        self.tags.push((field.to_string(), Some(filter)));
        self
    }

    /// Enabling Document Storage
    pub fn with_store(mut self) -> Self {
        self.store = true;
        self
    }

    /// Enable Fast Updates
    pub fn with_fastupdate(mut self) -> Self {
        self.fastupdate = true;
        self
    }

    /// Setting the cache size
    pub fn with_cache(mut self, size: usize) -> Self {
        self.cache = Some(size);
        self
    }
}

/// Extracting the value of a simple path from a document
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

        let doc = Document::new(config).expect("Document::new should succeed");
        assert_eq!(doc.len(), 2);
    }

    #[test]
    fn test_document_add() {
        let config = DocumentConfig::new().add_field(FieldConfig::new("title"));

        let mut doc = Document::new(config).expect("Document::new should succeed");
        doc.add(1, &json!({"title": "Hello World"}))
            .expect("add should succeed");

        assert!(doc.contains(1));
    }

    #[test]
    fn test_document_search() {
        let config = DocumentConfig::new().add_field(FieldConfig::new("title"));

        let mut doc = Document::new(config).expect("Document::new should succeed");
        doc.add(1, &json!({"title": "Hello World"}))
            .expect("add should succeed");
        doc.add(2, &json!({"title": "Rust Programming"}))
            .expect("add should succeed");

        let result = doc
            .search(&SearchOptions {
                query: Some("Hello".to_string()),
                limit: Some(10),
                ..Default::default()
            })
            .expect("search should succeed");

        assert_eq!(result.results.len(), 1);
        assert_eq!(result.results[0], 1);
    }

    #[test]
    fn test_document_remove() {
        let config = DocumentConfig::new().add_field(FieldConfig::new("title"));

        let mut doc = Document::new(config).expect("Document::new should succeed");
        doc.add(1, &json!({"title": "Hello"}))
            .expect("add should succeed");

        assert!(doc.contains(1));

        doc.remove(1).expect("remove should succeed");

        assert!(!doc.contains(1));
    }

    #[test]
    fn test_document_update() {
        let config = DocumentConfig::new()
            .add_field(FieldConfig::new("title"))
            .with_store();

        let mut doc = Document::new(config).expect("Document::new should succeed");
        doc.add(1, &json!({"title": "Original"}))
            .expect("add should succeed");

        doc.update(1, &json!({"title": "Updated"}))
            .expect("update should succeed");

        let stored = doc.get(1);
        assert_eq!(stored.expect("get should return Some")["title"], "Updated");
    }

    #[test]
    fn test_document_clear() {
        let config = DocumentConfig::new().add_field(FieldConfig::new("title"));

        let mut doc = Document::new(config).expect("Document::new should succeed");
        doc.add(1, &json!({"title": "Doc 1"}))
            .expect("add should succeed");
        doc.add(2, &json!({"title": "Doc 2"}))
            .expect("add should succeed");

        doc.clear();

        assert!(!doc.contains(1));
        assert!(!doc.contains(2));
    }

    #[test]
    fn test_document_batch() {
        let config = DocumentConfig::new().add_field(FieldConfig::new("title"));

        let mut doc = Document::new(config).expect("Document::new should succeed");

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
        let config = DocumentConfig::new().add_field(FieldConfig::new("title"));

        let mut doc = Document::new(config).unwrap();

        let doc1 = json!({"title": "Doc 1"});
        let doc2 = json!({"title": "Doc 2"});
        let doc3 = json!({"title": "Doc 3"});
        let operations = vec![(1, &doc1), (2, &doc2), (3, &doc3)];

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

        let mut doc = Document::new(config).expect("Document::new should succeed");
        doc.add(1, &json!({"title": "Original"}))
            .expect("add should succeed");
        doc.add(2, &json!({"title": "Original"}))
            .expect("add should succeed");

        let doc1 = json!({"title": "Updated 1"});
        let doc2 = json!({"title": "Updated 2"});
        let operations = vec![(1, &doc1), (2, &doc2)];

        let result = doc.batch_update(&operations);

        assert_eq!(result.total_operations, 2);
        assert_eq!(result.successful_operations, 2);
        assert_eq!(result.failed_operations, 0);

        let stored1 = doc.get(1);
        let stored2 = doc.get(2);
        assert_eq!(
            stored1.expect("get should return Some")["title"],
            "Updated 1"
        );
        assert_eq!(
            stored2.expect("get should return Some")["title"],
            "Updated 2"
        );
    }

    #[test]
    fn test_document_batch_remove() {
        let config = DocumentConfig::new().add_field(FieldConfig::new("title"));

        let mut doc = Document::new(config).expect("Document::new should succeed");
        doc.add(1, &json!({"title": "Doc 1"}))
            .expect("add should succeed");
        doc.add(2, &json!({"title": "Doc 2"}))
            .expect("add should succeed");
        doc.add(3, &json!({"title": "Doc 3"}))
            .expect("add should succeed");

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
        let config = DocumentConfig::new().add_field(FieldConfig::new("title"));

        let mut doc = Document::new(config).expect("Document::new should succeed");
        doc.add(1, &json!({"title": "Doc 1"}))
            .expect("add should succeed");
        doc.add(2, &json!({"title": "Doc 2"}))
            .expect("add should succeed");

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
