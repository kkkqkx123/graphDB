use std::collections::HashMap;
use std::sync::Arc;

use crate::core::{Value, Vertex};
use crate::search::engine::EngineType;
use crate::search::error::SearchError;
use crate::search::manager::FulltextIndexManager;
use crate::search::metadata::IndexMetadata;
use crate::search::result::SearchResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ChangeType {
    Insert,
    Update,
    Delete,
}

#[derive(Debug)]
pub struct FulltextCoordinator {
    manager: Arc<FulltextIndexManager>,
}

impl FulltextCoordinator {
    pub fn new(manager: Arc<FulltextIndexManager>) -> Self {
        Self { manager }
    }

    pub async fn create_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        engine_type: Option<EngineType>,
    ) -> Result<String, SearchError> {
        self.manager
            .create_index(space_id, tag_name, field_name, engine_type)
            .await
    }

    pub async fn drop_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Result<(), SearchError> {
        self.manager
            .drop_index(space_id, tag_name, field_name)
            .await
    }

    pub async fn on_vertex_inserted(
        &self,
        space_id: u64,
        vertex: &Vertex,
    ) -> Result<(), SearchError> {
        for tag in &vertex.tags {
            for (field_name, value) in &tag.properties {
                if let Some(engine) = self.manager.get_engine(space_id, &tag.name, field_name) {
                    if let Value::String(text) = value {
                        let doc_id = vertex.vid.to_string();
                        engine.index(&doc_id, text).await?;
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn on_vertex_updated(
        &self,
        space_id: u64,
        vertex: &Vertex,
        changed_fields: &[String],
    ) -> Result<(), SearchError> {
        for tag in &vertex.tags {
            for field_name in changed_fields {
                if let Some(value) = tag.properties.get(field_name) {
                    if let Some(engine) = self.manager.get_engine(space_id, &tag.name, field_name) {
                        if let Value::String(text) = value {
                            let doc_id = vertex.vid.to_string();
                            engine.delete(&doc_id).await?;
                            engine.index(&doc_id, text).await?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn on_vertex_deleted(
        &self,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
    ) -> Result<(), SearchError> {
        let doc_id = ToString::to_string(&vertex_id);

        let indexes = self.manager.get_space_indexes(space_id);
        for metadata in indexes {
            if metadata.tag_name == tag_name {
                if let Some(engine) =
                    self.manager
                        .get_engine(space_id, &metadata.tag_name, &metadata.field_name)
                {
                    engine.delete(&doc_id).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn on_vertex_change(
        &self,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
        properties: &HashMap<String, Value>,
        change_type: ChangeType,
    ) -> Result<(), SearchError> {
        let doc_id = ToString::to_string(&vertex_id);

        for (field_name, value) in properties {
            if let Some(engine) = self.manager.get_engine(space_id, tag_name, field_name) {
                match change_type {
                    ChangeType::Insert | ChangeType::Update => {
                        if let Value::String(text) = value {
                            engine.index(&doc_id, text).await?;
                        }
                    }
                    ChangeType::Delete => {
                        engine.delete(&doc_id).await?;
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn search(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, SearchError> {
        self.manager
            .search(space_id, tag_name, field_name, query, limit)
            .await
    }

    pub fn get_engine(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Option<Arc<dyn crate::search::engine::SearchEngine>> {
        self.manager.get_engine(space_id, tag_name, field_name)
    }

    pub async fn rebuild_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Result<(), SearchError> {
        let engine = self
            .manager
            .get_engine(space_id, tag_name, field_name)
            .ok_or_else(|| {
                SearchError::IndexNotFound(format!("{}.{}.{}", space_id, tag_name, field_name))
            })?;

        engine.commit().await?;
        Ok(())
    }

    pub fn list_indexes(&self) -> Vec<IndexMetadata> {
        self.manager.list_indexes()
    }

    pub async fn commit_all(&self) -> Result<(), SearchError> {
        self.manager.commit_all().await
    }
}
