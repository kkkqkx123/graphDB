use crate::r#type::{DocId, EnrichedSearchResults, SearchResults};
use crate::Index;
use crate::storage::common::utils::apply_limit_offset;
use std::collections::HashMap;

#[derive(Debug)]
pub struct StorageBase {
    pub data: HashMap<String, Vec<DocId>>,
    pub context_data: HashMap<String, HashMap<String, Vec<DocId>>>,
    pub documents: HashMap<DocId, String>,
}

impl StorageBase {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            context_data: HashMap::new(),
            documents: HashMap::new(),
        }
    }

    pub fn commit_from_index(&mut self, index: &Index) {
        for doc_ids in index.map.index.values() {
            for (term_str, ids) in doc_ids {
                self.data.insert(term_str.clone(), ids.clone());
            }
        }

        for ctx_map in index.ctx.index.values() {
            for (ctx_term, doc_ids) in ctx_map {
                self.context_data
                    .entry("default".to_string())
                    .or_default()
                    .insert(ctx_term.clone(), doc_ids.clone());
            }
        }

        for (id, content) in &index.documents {
            self.documents.insert(*id, content.clone());
        }
    }

    pub fn get(&self, key: &str, ctx: Option<&str>, limit: usize, offset: usize) -> SearchResults {
        let results = if let Some(ctx_key) = ctx {
            if let Some(ctx_map) = self.context_data.get(ctx_key) {
                ctx_map.get(key).cloned().unwrap_or_default()
            } else {
                Vec::new()
            }
        } else {
            self.data.get(key).cloned().unwrap_or_default()
        };

        apply_limit_offset(&results, limit, offset)
    }

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

    pub fn has(&self, id: DocId) -> bool {
        for doc_ids in self.data.values() {
            if doc_ids.contains(&id) {
                return true;
            }
        }

        for ctx_map in self.context_data.values() {
            for doc_ids in ctx_map.values() {
                if doc_ids.contains(&id) {
                    return true;
                }
            }
        }

        false
    }

    pub fn remove(&mut self, ids: &[DocId]) {
        for &id in ids {
            self.documents.remove(&id);

            for doc_ids in self.data.values_mut() {
                doc_ids.retain(|&doc_id| doc_id != id);
            }

            for ctx_map in self.context_data.values_mut() {
                for doc_ids in ctx_map.values_mut() {
                    doc_ids.retain(|&doc_id| doc_id != id);
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.context_data.clear();
        self.documents.clear();
    }

    pub fn get_document_count(&self) -> usize {
        self.documents.len()
    }

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
}
