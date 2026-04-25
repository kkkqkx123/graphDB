use crate::{
    config::{EmbeddedConfig, TokenizeMode},
    error::Result,
    keystore::DocId,
    r#type::SearchOptions,
    search::search,
    Index,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Embedded search result - simplified for library users
#[derive(Debug, Clone)]
pub struct EmbeddedSearchResult {
    pub id: DocId,
    pub content: String,
    pub score: f32,
    pub highlights: Option<Vec<String>>,
}

/// Embedded index statistics
#[derive(Debug, Clone)]
pub struct EmbeddedIndexStats {
    pub document_count: usize,
    pub stored_document_count: usize,
    pub index_path: Option<String>,
}

/// Batch operation type
#[derive(Debug, Clone)]
pub enum EmbeddedBatchOperation {
    Add { id: DocId, content: String },
    Remove { id: DocId },
}

/// Batch operation result
#[derive(Debug, Clone)]
pub struct EmbeddedBatchResult {
    pub success_count: usize,
    pub failed_count: usize,
    pub errors: Vec<String>,
}

/// Embedded index - high-level API for library users
pub struct EmbeddedIndex {
    index: Index,
    config: EmbeddedConfig,
    document_store: HashMap<DocId, String>,
}

impl EmbeddedIndex {
    /// Create a new index with default configuration
    pub fn create() -> Result<Self> {
        Self::with_config(EmbeddedConfig::default())
    }

    /// Create a new index at the specified path
    pub fn create_at(path: impl Into<PathBuf>) -> Result<Self> {
        let config = EmbeddedConfig::builder().path(path).build();
        Self::with_config(config)
    }

    /// Create a new index with custom configuration
    pub fn with_config(config: EmbeddedConfig) -> Result<Self> {
        let index_options = config.to_index_options();
        let index = Index::new(index_options)?;
        let document_store = HashMap::new();
        Ok(Self {
            index,
            config,
            document_store,
        })
    }

    /// Create a builder for configuring the index
    pub fn builder() -> EmbeddedIndexBuilder {
        EmbeddedIndexBuilder::new()
    }

    /// Open an existing index from path
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let config = EmbeddedConfig::builder().path(path).build();
        Self::with_config(config)
    }

    /// Add a document
    pub fn add(&mut self, id: DocId, content: impl Into<String>) -> Result<()> {
        let content = content.into();
        if self.config.store_documents {
            self.document_store.insert(id, content.clone());
        }
        self.index.add(id, &content, false)
    }

    /// Add a document with fields
    pub fn add_with_fields(&mut self, id: DocId, fields: Vec<(String, String)>) -> Result<()> {
        let content = fields
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\n");
        if self.config.store_documents {
            self.document_store.insert(id, content.clone());
        }
        self.index.add(id, &content, false)
    }

    /// Update a document (remove then add)
    pub fn update(&mut self, id: DocId, content: impl Into<String>) -> Result<()> {
        let content = content.into();
        if self.config.store_documents {
            self.document_store.insert(id, content.clone());
        }
        self.index.update(id, &content)
    }

    /// Remove a document
    pub fn remove(&mut self, id: DocId) -> Result<()> {
        if self.config.store_documents {
            self.document_store.remove(&id);
        }
        self.index.remove(id, false)
    }

    /// Get a document by ID
    pub fn get(&self, id: DocId) -> Option<&str> {
        self.document_store.get(&id).map(|s| s.as_str())
    }

    /// Check if a document exists
    pub fn contains(&self, id: DocId) -> bool {
        self.index.contains(id)
    }

    /// Search with default limit
    pub fn search(&self, query: impl Into<String>) -> Result<Vec<EmbeddedSearchResult>> {
        let limit = self.config.default_search_limit;
        self.search_with_limit(query, limit)
    }

    /// Search with custom limit
    pub fn search_with_limit(
        &self,
        query: impl Into<String>,
        limit: usize,
    ) -> Result<Vec<EmbeddedSearchResult>> {
        let query_str = query.into();
        let search_opts = SearchOptions {
            query: Some(query_str.clone()),
            limit: Some(limit),
            ..Default::default()
        };

        let result = search(&self.index, &search_opts)?;

        let embedded_results: Vec<EmbeddedSearchResult> = result
            .results
            .into_iter()
            .enumerate()
            .map(|(idx, doc_id)| {
                let content = self
                    .document_store
                    .get(&doc_id)
                    .cloned()
                    .unwrap_or_default();
                let score = 1.0 / (1.0 + idx as f32);
                EmbeddedSearchResult {
                    id: doc_id,
                    content,
                    score,
                    highlights: None,
                }
            })
            .collect();

        Ok(embedded_results)
    }

    /// Get index statistics
    pub fn stats(&self) -> EmbeddedIndexStats {
        EmbeddedIndexStats {
            document_count: self.index.document_count(),
            stored_document_count: self.document_store.len(),
            index_path: self
                .config
                .index_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
        }
    }

    /// Clear all documents from the index
    pub fn clear(&mut self) {
        self.index.clear();
        self.document_store.clear();
    }

    /// Create a batch operation builder
    pub fn batch(&mut self) -> EmbeddedBatch<'_> {
        EmbeddedBatch::new(self)
    }

    /// Get the internal index (for advanced usage)
    pub fn inner(&self) -> &Index {
        &self.index
    }

    /// Get the mutable internal index (for advanced usage)
    pub fn inner_mut(&mut self) -> &mut Index {
        &mut self.index
    }

    /// Get the configuration
    pub fn config(&self) -> &EmbeddedConfig {
        &self.config
    }

    /// Save the index to the configured path
    pub fn save(&self) -> Result<()> {
        let path = self.config.index_path.as_ref().ok_or_else(|| {
            crate::error::InversearchError::Config("No index path configured".to_string())
        })?;
        self.save_to(path)
    }

    /// Save the index to a specific path
    pub fn save_to(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let serialize_config = crate::serialize::SerializeConfig::with_compression(
            crate::serialize::CompressionAlgorithm::Zstd,
            3,
        );

        let index_data = self.index.to_binary(&serialize_config)?;

        let docs_data = if self.config.store_documents {
            serde_json::to_vec(&self.document_store).map_err(|e| {
                crate::error::InversearchError::Serialization(format!(
                    "Failed to serialize document store: {}",
                    e
                ))
            })?
        } else {
            Vec::new()
        };

        let metadata = IndexFileMetadata {
            version: 1,
            has_document_store: self.config.store_documents,
            config: self.config.clone(),
        };
        let metadata_bytes = serde_json::to_vec(&metadata).map_err(|e| {
            crate::error::InversearchError::Serialization(format!(
                "Failed to serialize metadata: {}",
                e
            ))
        })?;

        let mut file_content = Vec::new();
        file_content.extend_from_slice(&(metadata_bytes.len() as u64).to_le_bytes());
        file_content.extend_from_slice(&metadata_bytes);
        file_content.extend_from_slice(&(index_data.len() as u64).to_le_bytes());
        file_content.extend_from_slice(&index_data);
        if self.config.store_documents {
            file_content.extend_from_slice(&(docs_data.len() as u64).to_le_bytes());
            file_content.extend_from_slice(&docs_data);
        }

        std::fs::write(path, file_content)?;

        Ok(())
    }

    /// Load an index from the configured path
    pub fn load(&mut self) -> Result<()> {
        let path = self.config.index_path.clone().ok_or_else(|| {
            crate::error::InversearchError::Config("No index path configured".to_string())
        })?;
        self.load_from(path)
    }

    /// Load an index from a specific path
    pub fn load_from(&mut self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let path = path.as_ref();

        let file_content = std::fs::read(path)?;

        let mut offset = 0;

        let metadata_len =
            u64::from_le_bytes(file_content[offset..offset + 8].try_into().map_err(|_| {
                crate::error::InversearchError::Deserialization("Invalid file format".to_string())
            })?) as usize;
        offset += 8;

        let metadata: IndexFileMetadata =
            serde_json::from_slice(&file_content[offset..offset + metadata_len]).map_err(|e| {
                crate::error::InversearchError::Deserialization(format!(
                    "Failed to deserialize metadata: {}",
                    e
                ))
            })?;
        offset += metadata_len;

        let index_len =
            u64::from_le_bytes(file_content[offset..offset + 8].try_into().map_err(|_| {
                crate::error::InversearchError::Deserialization("Invalid file format".to_string())
            })?) as usize;
        offset += 8;

        let serialize_config = crate::serialize::SerializeConfig::with_compression(
            crate::serialize::CompressionAlgorithm::Zstd,
            3,
        );

        self.index =
            Index::from_binary(&file_content[offset..offset + index_len], &serialize_config)?;
        offset += index_len;

        if metadata.has_document_store {
            let docs_len =
                u64::from_le_bytes(file_content[offset..offset + 8].try_into().map_err(|_| {
                    crate::error::InversearchError::Deserialization(
                        "Invalid file format".to_string(),
                    )
                })?) as usize;
            offset += 8;

            self.document_store = serde_json::from_slice(&file_content[offset..offset + docs_len])
                .map_err(|e| {
                    crate::error::InversearchError::Deserialization(format!(
                        "Failed to deserialize document store: {}",
                        e
                    ))
                })?;
        }

        Ok(())
    }

    /// Open an existing index from path (loads from disk if exists)
    pub fn open_or_create(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();

        if path.exists() {
            let mut index = Self::open(path.clone())?;
            index.load_from(&path)?;
            Ok(index)
        } else {
            Self::create_at(path)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexFileMetadata {
    version: u32,
    has_document_store: bool,
    config: EmbeddedConfig,
}

/// Builder for creating EmbeddedIndex
pub struct EmbeddedIndexBuilder {
    config: EmbeddedConfig,
}

impl EmbeddedIndexBuilder {
    pub fn new() -> Self {
        Self {
            config: EmbeddedConfig::default(),
        }
    }

    pub fn path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.index_path = Some(path.into());
        self
    }

    pub fn resolution(mut self, resolution: usize) -> Self {
        self.config.resolution = resolution;
        self
    }

    pub fn tokenize(mut self, tokenize: TokenizeMode) -> Self {
        self.config.tokenize = tokenize;
        self
    }

    pub fn depth(mut self, depth: usize) -> Self {
        self.config.depth = depth;
        self
    }

    pub fn bidirectional(mut self, bidirectional: bool) -> Self {
        self.config.bidirectional = bidirectional;
        self
    }

    pub fn fastupdate(mut self, fastupdate: bool) -> Self {
        self.config.fastupdate = fastupdate;
        self
    }

    pub fn cache_size(mut self, size: usize) -> Self {
        self.config.cache_size = size;
        self
    }

    pub fn cache_ttl(mut self, ttl: std::time::Duration) -> Self {
        self.config.cache_ttl = Some(ttl);
        self
    }

    pub fn store_documents(mut self, store: bool) -> Self {
        self.config.store_documents = store;
        self
    }

    pub fn enable_highlighting(mut self, enable: bool) -> Self {
        self.config.enable_highlighting = enable;
        self
    }

    pub fn default_search_limit(mut self, limit: usize) -> Self {
        self.config.default_search_limit = limit;
        self
    }

    pub fn build(self) -> Result<EmbeddedIndex> {
        EmbeddedIndex::with_config(self.config)
    }
}

impl Default for EmbeddedIndexBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Batch operation builder
pub struct EmbeddedBatch<'a> {
    index: &'a mut EmbeddedIndex,
    operations: Vec<EmbeddedBatchOperation>,
}

impl<'a> EmbeddedBatch<'a> {
    /// Create a new batch
    pub fn new(index: &'a mut EmbeddedIndex) -> Self {
        Self {
            index,
            operations: Vec::new(),
        }
    }

    /// Add an add operation
    pub fn add(mut self, id: DocId, content: impl Into<String>) -> Self {
        self.operations.push(EmbeddedBatchOperation::Add {
            id,
            content: content.into(),
        });
        self
    }

    /// Add a remove operation
    pub fn remove(mut self, id: DocId) -> Self {
        self.operations.push(EmbeddedBatchOperation::Remove { id });
        self
    }

    /// Execute all operations
    pub fn execute(self) -> EmbeddedBatchResult {
        let mut success_count = 0;
        let mut failed_count = 0;
        let mut errors = Vec::new();

        for op in self.operations {
            match op {
                EmbeddedBatchOperation::Add { id, content } => match self.index.add(id, &content) {
                    Ok(_) => success_count += 1,
                    Err(e) => {
                        failed_count += 1;
                        errors.push(format!("Failed to add {}: {}", id, e));
                    }
                },
                EmbeddedBatchOperation::Remove { id } => match self.index.remove(id) {
                    Ok(_) => success_count += 1,
                    Err(e) => {
                        failed_count += 1;
                        errors.push(format!("Failed to remove {}: {}", id, e));
                    }
                },
            }
        }

        EmbeddedBatchResult {
            success_count,
            failed_count,
            errors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_index_create() {
        let index = EmbeddedIndex::create();
        assert!(index.is_ok());
    }

    #[test]
    fn test_embedded_index_builder() {
        let index = EmbeddedIndex::builder()
            .resolution(12)
            .tokenize(TokenizeMode::Forward)
            .depth(2)
            .cache_size(2000)
            .store_documents(true)
            .default_search_limit(20)
            .build();

        assert!(index.is_ok());
        let index = index.unwrap();
        assert_eq!(index.config().resolution, 12);
        assert_eq!(index.config().tokenize, TokenizeMode::Forward);
        assert_eq!(index.config().default_search_limit, 20);
    }

    #[test]
    fn test_embedded_index_add_and_search() {
        let mut index = EmbeddedIndex::create().unwrap();

        assert!(index.add(1, "Hello world").is_ok());
        assert!(index.add(2, "Rust programming").is_ok());
        assert!(index.add(3, "Hello Rust").is_ok());

        let results = index.search("hello").unwrap();
        assert!(!results.is_empty());

        let first_result = &results[0];
        assert!(!first_result.content.is_empty());
    }

    #[test]
    fn test_embedded_index_document_storage() {
        let mut index = EmbeddedIndex::create().unwrap();

        index.add(1, "Test document content").unwrap();

        assert_eq!(index.get(1), Some("Test document content"));
        assert!(index.contains(1));
    }

    #[test]
    fn test_embedded_index_update() {
        let mut index = EmbeddedIndex::create().unwrap();

        index.add(1, "Original content").unwrap();
        assert_eq!(index.get(1), Some("Original content"));

        index.update(1, "Updated content").unwrap();
        assert_eq!(index.get(1), Some("Updated content"));
    }

    #[test]
    fn test_embedded_index_batch() {
        let mut index = EmbeddedIndex::create().unwrap();

        let result = index
            .batch()
            .add(1, "Document 1")
            .add(2, "Document 2")
            .add(3, "Document 3")
            .execute();

        assert_eq!(result.success_count, 3);
        assert_eq!(result.failed_count, 0);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_embedded_index_remove() {
        let mut index = EmbeddedIndex::create().unwrap();

        let _ = index.add(1, "Test document");
        let _ = index.remove(1);

        assert_eq!(index.get(1), None);

        let results = index.search("test").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_embedded_index_stats() {
        let mut index = EmbeddedIndex::create().unwrap();

        let stats_before = index.stats();
        assert_eq!(stats_before.document_count, 0);
        assert_eq!(stats_before.stored_document_count, 0);

        index.add(1, "Test").unwrap();

        let stats_after = index.stats();
        assert_eq!(stats_after.document_count, 1);
        assert_eq!(stats_after.stored_document_count, 1);
    }

    #[test]
    fn test_embedded_index_clear() {
        let mut index = EmbeddedIndex::create().unwrap();

        index.add(1, "Test 1").unwrap();
        index.add(2, "Test 2").unwrap();

        index.clear();

        let stats = index.stats();
        assert_eq!(stats.document_count, 0);
        assert_eq!(stats.stored_document_count, 0);
    }

    #[test]
    fn test_search_result_content() {
        let mut index = EmbeddedIndex::create().unwrap();

        index.add(1, "The quick brown fox").unwrap();
        index.add(2, "The lazy dog").unwrap();

        let results = index.search("quick").unwrap();

        assert!(!results.is_empty());
        let result = &results[0];
        assert_eq!(result.content, "The quick brown fox");
        assert!(result.score > 0.0);
    }

    #[test]
    fn test_embedded_index_save_and_load() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let index_path = temp_dir.path().join("test_index.idx");

        let mut index = EmbeddedIndex::create_at(&index_path).unwrap();
        index.add(1, "Hello world").unwrap();
        index.add(2, "Rust programming").unwrap();
        index.add(3, "Hello Rust").unwrap();

        index.save_to(&index_path).expect("Failed to save index");

        let mut loaded_index = EmbeddedIndex::open(&index_path).unwrap();
        loaded_index
            .load_from(&index_path)
            .expect("Failed to load index");

        let results = loaded_index.search("hello").unwrap();
        assert_eq!(results.len(), 2);

        assert_eq!(loaded_index.get(1), Some("Hello world"));
        assert_eq!(loaded_index.get(2), Some("Rust programming"));
        assert_eq!(loaded_index.get(3), Some("Hello Rust"));
    }

    #[test]
    fn test_embedded_index_open_or_create() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let index_path = temp_dir.path().join("auto_index.idx");

        {
            let mut index = EmbeddedIndex::open_or_create(&index_path).unwrap();
            index.add(1, "Test document").unwrap();
            index.save().unwrap();
        }

        {
            let index = EmbeddedIndex::open_or_create(&index_path).unwrap();
            let results = index.search("test").unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].content, "Test document");
        }
    }
}
