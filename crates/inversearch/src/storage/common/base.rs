//! Store base class implementations
//!
//! Provide data structures and core logic that are shared across storage implementations

use crate::r#type::{DocId, EnrichedSearchResults, SearchResults};
use crate::Index;

use crate::storage::common::utils::apply_limit_offset;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// Storage base class
///
/// Encapsulate the data structures and core logic shared by all in-memory storage implementations:
/// - Indexed data storage
/// - Contextual data storage
/// - Document Content Storage
/// - Statistics on performance indicators
#[derive(Debug)]
pub struct StorageBase {
    /// Main index data: lexical items -> list of document IDs
    pub data: HashMap<String, Vec<DocId>>,
    /// Context index data: context -> lexical item -> document ID list
    pub context_data: HashMap<String, HashMap<String, Vec<DocId>>>,
    /// Document content storage: document ID -> content
    pub documents: HashMap<DocId, String>,
    /// Memory usage (bytes)
    pub(crate) memory_usage: AtomicUsize,
    /// operation counter
    pub(crate) operation_count: AtomicUsize,
    /// Total delay (microseconds)
    pub(crate) total_latency: AtomicUsize,
}

impl StorageBase {
    /// Creating a new instance of the storage base class
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            context_data: HashMap::new(),
            documents: HashMap::new(),
            memory_usage: AtomicUsize::new(0),
            operation_count: AtomicUsize::new(0),
            total_latency: AtomicUsize::new(0),
        }
    }

    /// Commit data from index to storage
    ///
    /// Exporting data from an index to a storage base class
    pub fn commit_from_index(&mut self, index: &Index) {
        // Exporting data from the primary index
        for doc_ids in index.map.index.values() {
            for (term_str, ids) in doc_ids {
                self.data.insert(term_str.clone(), ids.clone());
            }
        }

        // Exporting data from a contextual index
        for ctx_map in index.ctx.index.values() {
            for (ctx_term, doc_ids) in ctx_map {
                self.context_data
                    .entry("default".to_string())
                    .or_default()
                    .insert(ctx_term.clone(), doc_ids.clone());
            }
        }

        // Exporting document content from indexed documents
        for (id, content) in &index.documents {
            self.documents.insert(*id, content.clone());
        }

        self.update_memory_usage();
    }

    /// Get search results for the specified key
    ///
    /// # Parameters
    /// - `key`: 搜索词项
    /// - `ctx`: 可选的上下文名称
    /// - `limit`: 返回结果数量限制（0表示无限制）
    /// - `offset`: 结果偏移量
    pub fn get(&self, key: &str, ctx: Option<&str>, limit: usize, offset: usize) -> SearchResults {
        let results = if let Some(ctx_key) = ctx {
            // context search
            if let Some(ctx_map) = self.context_data.get(ctx_key) {
                ctx_map.get(key).cloned().unwrap_or_default()
            } else {
                Vec::new()
            }
        } else {
            // General Search
            self.data.get(key).cloned().unwrap_or_default()
        };

        apply_limit_offset(&results, limit, offset)
    }

    /// Fuhua Search Results
    ///
    /// Get complete document content based on a list of document IDs
    pub fn enrich(&self, ids: &[DocId]) -> EnrichedSearchResults {
        let mut results = Vec::new();

        for &id in ids {
            if let Some(content) = self.documents.get(&id) {
                results.push(crate::r#type::EnrichedSearchResult {
                    id,
                    doc: Some(serde_json::json!({
                        "content": content,
                        "id": id
                    })),
                    highlight: None,
                });
            }
        }

        results
    }

    /// Check if the document ID exists
    ///
    /// Search for the specified ID in index data and context data
    pub fn has(&self, id: DocId) -> bool {
        // Checking primary index data
        for doc_ids in self.data.values() {
            if doc_ids.contains(&id) {
                return true;
            }
        }

        // Examining contextual data
        for ctx_map in self.context_data.values() {
            for doc_ids in ctx_map.values() {
                if doc_ids.contains(&id) {
                    return true;
                }
            }
        }

        false
    }

    /// Remove the specified document
    ///
    /// Remove specified IDs from document store, index data and context data
    pub fn remove(&mut self, ids: &[DocId]) {
        for &id in ids {
            self.documents.remove(&id);

            // Remove from primary index data
            for doc_ids in self.data.values_mut() {
                doc_ids.retain(|&doc_id| doc_id != id);
            }

            // Remove from context data
            for ctx_map in self.context_data.values_mut() {
                for doc_ids in ctx_map.values_mut() {
                    doc_ids.retain(|&doc_id| doc_id != id);
                }
            }
        }
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.data.clear();
        self.context_data.clear();
        self.documents.clear();
    }

    /// Get memory usage in bytes
    pub fn get_memory_usage(&self) -> usize {
        self.memory_usage.load(Ordering::Relaxed)
    }

    /// Get Operation Count
    pub fn get_operation_count(&self) -> usize {
        self.operation_count.load(Ordering::Relaxed)
    }

    /// Get total delay (microseconds)
    pub fn get_total_latency(&self) -> usize {
        self.total_latency.load(Ordering::Relaxed)
    }

    /// Calculation of average delay (microseconds)
    pub fn get_average_latency(&self) -> usize {
        let count = self.get_operation_count();
        if count > 0 {
            self.get_total_latency() / count
        } else {
            0
        }
    }

    /// Updating Memory Usage Statistics
    ///
    /// Calculate the memory footprint of all data structures
    pub fn update_memory_usage(&self) {
        let mut total_size = 0;

        // Calculating the primary index data size
        total_size += std::mem::size_of_val(&self.data);
        for (k, v) in &self.data {
            total_size += k.len() + v.len() * std::mem::size_of::<DocId>();
        }

        // Calculating Context Data Size
        total_size += std::mem::size_of_val(&self.context_data);
        for (ctx_key, ctx_map) in &self.context_data {
            total_size += ctx_key.len();
            total_size += std::mem::size_of_val(ctx_map);
            for (term, ids) in ctx_map {
                total_size += term.len() + ids.len() * std::mem::size_of::<DocId>();
            }
        }

        // Calculating Document Storage Size
        total_size += std::mem::size_of_val(&self.documents);
        for (id, content) in &self.documents {
            total_size += std::mem::size_of_val(id) + content.len();
        }

        self.memory_usage.store(total_size, Ordering::Relaxed);
    }

    /// Record operation start time
    ///
    /// Returns the current timestamp, which is used to delay subsequent calculation operations
    pub fn record_operation_start(&self) -> Instant {
        Instant::now()
    }

    /// Record the completion of the operation
    ///
    /// Calculate and record operating delays based on start time
    pub fn record_operation_completion(&self, start_time: Instant) {
        let latency = start_time.elapsed().as_micros() as usize;
        self.operation_count.fetch_add(1, Ordering::Relaxed);
        self.total_latency.fetch_add(latency, Ordering::Relaxed);
    }

    /// Get the number of documents
    pub fn get_document_count(&self) -> usize {
        self.documents.len()
    }

    /// Get the number of indexed items
    pub fn get_index_count(&self) -> usize {
        self.data.len()
    }
}

impl Default for StorageBase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_base_new() {
        let base = StorageBase::new();
        assert_eq!(base.get_memory_usage(), 0);
        assert_eq!(base.get_operation_count(), 0);
        assert_eq!(base.get_document_count(), 0);
        assert_eq!(base.get_index_count(), 0);
    }

    #[test]
    fn test_storage_base_clear() {
        let mut base = StorageBase::new();
        base.data.insert("test".to_string(), vec![1, 2, 3]);
        base.documents.insert(1, "content".to_string());

        base.clear();

        assert!(base.data.is_empty());
        assert!(base.documents.is_empty());
    }

    #[test]
    fn test_storage_base_has() {
        let mut base = StorageBase::new();
        base.data.insert("test".to_string(), vec![1, 2, 3]);

        assert!(base.has(1));
        assert!(base.has(2));
        assert!(!base.has(999));
    }

    #[test]
    fn test_storage_base_remove() {
        let mut base = StorageBase::new();
        base.data.insert("test".to_string(), vec![1, 2, 3]);
        base.documents.insert(1, "doc1".to_string());
        base.documents.insert(2, "doc2".to_string());

        base.remove(&[1]);

        assert!(!base.has(1));
        assert!(base.has(2));
        assert!(!base.documents.contains_key(&1));
        assert!(base.documents.contains_key(&2));
    }

    #[test]
    fn test_storage_base_operation_timing() {
        let base = StorageBase::new();

        let start = base.record_operation_start();
        std::thread::sleep(std::time::Duration::from_millis(1));
        base.record_operation_completion(start);

        assert_eq!(base.get_operation_count(), 1);
        assert!(base.get_total_latency() > 0);
        assert!(base.get_average_latency() > 0);
    }
}
