//! Full-text Storage Manager
//!
//! Provides storage layer abstraction for full-text search functionality.
//! This module wraps the FulltextIndexManager and provides a unified interface
//! for vertex and edge full-text indexing operations.

use crate::core::error::StorageError;
use crate::core::{Value, Vertex};
use crate::search::engine::EngineType;
use crate::search::manager::FulltextIndexManager;
use crate::search::result::{IndexStats, SearchResult};
use std::sync::Arc;

/// Full-text Storage Manager
///
/// Responsible for full-text index management and document indexing operations.
/// This struct provides a storage-layer abstraction over the search engine,
/// similar to how VertexStorage and EdgeStorage work for graph data.
#[derive(Clone)]
pub struct FulltextStorage {
    manager: Arc<FulltextIndexManager>,
}

impl std::fmt::Debug for FulltextStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FulltextStorage")
            .field("manager", &self.manager)
            .finish()
    }
}

impl FulltextStorage {
    /// Create a new full-text storage instance
    pub fn new(manager: Arc<FulltextIndexManager>) -> Self {
        Self { manager }
    }

    /// Get the underlying manager
    pub fn manager(&self) -> &Arc<FulltextIndexManager> {
        &self.manager
    }

    // ==================== Index Management ====================

    /// Create a new full-text index
    pub async fn create_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        engine_type: Option<EngineType>,
    ) -> Result<String, StorageError> {
        self.manager
            .create_index(space_id, tag_name, field_name, engine_type)
            .await
            .map_err(|e| StorageError::DbError(format!("Failed to create fulltext index: {}", e)))
    }

    /// Drop a full-text index
    pub async fn drop_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Result<(), StorageError> {
        self.manager
            .drop_index(space_id, tag_name, field_name)
            .await
            .map_err(|e| StorageError::DbError(format!("Failed to drop fulltext index: {}", e)))
    }

    /// Check if an index exists
    pub fn has_index(&self, space_id: u64, tag_name: &str, field_name: &str) -> bool {
        self.manager.has_index(space_id, tag_name, field_name)
    }

    /// List all indexes for a space
    pub fn list_indexes(&self, space_id: u64) -> Vec<crate::search::metadata::IndexMetadata> {
        self.manager.get_space_indexes(space_id)
    }

    /// Get index statistics
    pub async fn get_index_stats(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Result<IndexStats, StorageError> {
        self.manager
            .get_stats(space_id, tag_name, field_name)
            .await
            .map_err(|e| StorageError::DbError(format!("Failed to get index stats: {}", e)))
    }

    // ==================== Vertex Indexing ====================

    /// Index a vertex document
    ///
    /// This method indexes all string properties of a vertex that have corresponding
    /// full-text indexes configured.
    pub async fn index_vertex(&self, space_id: u64, vertex: &Vertex) -> Result<(), StorageError> {
        for tag in &vertex.tags {
            for (field_name, value) in &tag.properties {
                if let Value::String(text) = value {
                    if self.manager.has_index(space_id, &tag.name, field_name) {
                        let doc_id = vertex.vid.to_string();
                        if let Some(engine) =
                            self.manager.get_engine(space_id, &tag.name, field_name)
                        {
                            engine.index(&doc_id, text).await.map_err(|e| {
                                StorageError::DbError(format!(
                                    "Failed to index vertex document: {}",
                                    e
                                ))
                            })?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Delete a vertex document from all indexes
    pub async fn delete_vertex(
        &self,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
    ) -> Result<(), StorageError> {
        let doc_id = vertex_id.to_string().map_err(|e| {
            StorageError::DbError(format!("Failed to convert vertex_id to string: {}", e))
        })?;
        let indexes = self.manager.get_space_indexes(space_id);

        for metadata in indexes {
            if metadata.tag_name == tag_name {
                if let Some(engine) =
                    self.manager
                        .get_engine(space_id, &metadata.tag_name, &metadata.field_name)
                {
                    engine.delete(&doc_id).await.ok(); // Ignore deletion errors
                }
            }
        }
        Ok(())
    }

    /// Update vertex full-text index
    ///
    /// This method re-indexes changed fields for a vertex.
    #[allow(clippy::collapsible_if)]
    pub async fn update_vertex(
        &self,
        space_id: u64,
        vertex: &Vertex,
        changed_fields: &[String],
    ) -> Result<(), StorageError> {
        let doc_id = vertex.vid.to_string();

        for tag in &vertex.tags {
            for field_name in changed_fields {
                if let Some(value) = tag.properties.get(field_name) {
                    if let Value::String(text) = value {
                        if let Some(engine) =
                            self.manager.get_engine(space_id, &tag.name, field_name)
                        {
                            // Delete old document first
                            engine.delete(&doc_id).await.ok();
                            // Index new content
                            engine.index(&doc_id, text).await.map_err(|e| {
                                StorageError::DbError(format!(
                                    "Failed to update vertex document: {}",
                                    e
                                ))
                            })?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    // ==================== Edge Indexing ====================

    /// Index an edge property
    pub async fn index_edge(
        &self,
        space_id: u64,
        edge_type: &str,
        field_name: &str,
        doc_id: &str,
        text: &str,
    ) -> Result<(), StorageError> {
        if self.manager.has_index(space_id, edge_type, field_name) {
            if let Some(engine) = self.manager.get_engine(space_id, edge_type, field_name) {
                engine.index(doc_id, text).await.map_err(|e| {
                    StorageError::DbError(format!("Failed to index edge document: {}", e))
                })?;
            }
        }
        Ok(())
    }

    /// Delete an edge document from all indexes
    pub async fn delete_edge(
        &self,
        space_id: u64,
        edge_type: &str,
        doc_id: &str,
    ) -> Result<(), StorageError> {
        let edge_indexes: Vec<_> = self
            .manager
            .list_indexes()
            .into_iter()
            .filter(|metadata| metadata.space_id == space_id && metadata.tag_name == edge_type)
            .collect();

        for metadata in edge_indexes {
            if let Some(engine) =
                self.manager
                    .get_engine(space_id, &metadata.tag_name, &metadata.field_name)
            {
                engine.delete(doc_id).await.ok(); // Ignore deletion errors
            }
        }
        Ok(())
    }

    // ==================== Search Operations ====================

    /// Search full-text index
    pub async fn search(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, StorageError> {
        self.manager
            .search(space_id, tag_name, field_name, query, limit)
            .await
            .map_err(|e| StorageError::DbError(format!("Fulltext search failed: {}", e)))
    }

    // ==================== Transaction Support ====================

    /// Commit all pending changes
    pub async fn commit_all(&self) -> Result<(), StorageError> {
        self.manager
            .commit_all()
            .await
            .map_err(|e| StorageError::DbError(format!("Failed to commit fulltext changes: {}", e)))
    }

    /// Close all indexes
    pub async fn close_all(&self) -> Result<(), StorageError> {
        self.manager
            .close_all()
            .await
            .map_err(|e| StorageError::DbError(format!("Failed to close fulltext indexes: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::config::FulltextConfig;

    #[tokio::test]
    async fn test_fulltext_storage_creation() {
        let config = FulltextConfig::default();
        let manager = Arc::new(FulltextIndexManager::new(config).unwrap());
        let storage = FulltextStorage::new(manager);

        // Just verify it can be created
        assert!(storage.manager().list_indexes().is_empty());
    }
}
