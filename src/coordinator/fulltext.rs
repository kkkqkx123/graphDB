use std::collections::HashMap;
use std::sync::Arc;

use crate::core::error::{CoordinatorError, CoordinatorResult, FulltextError};
use crate::core::{Value, Vertex};
use crate::search::engine::EngineType;
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
    ) -> CoordinatorResult<String> {
        self.manager
            .create_index(space_id, tag_name, field_name, engine_type)
            .await
            .map_err(FulltextError::from)?;
        Ok(format!("{}_{}_{}", space_id, tag_name, field_name))
    }

    pub async fn drop_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> CoordinatorResult<()> {
        self.manager
            .drop_index(space_id, tag_name, field_name)
            .await
            .map_err(FulltextError::from)?;
        Ok(())
    }

    pub async fn on_vertex_inserted(
        &self,
        space_id: u64,
        vertex: &Vertex,
    ) -> CoordinatorResult<()> {
        for tag in &vertex.tags {
            for (field_name, value) in &tag.properties {
                if let Some(engine) = self.manager.get_engine(space_id, &tag.name, field_name) {
                    if let Value::String(text) = value {
                        let doc_id = vertex.vid.to_string();
                        engine
                            .index(&doc_id, text)
                            .await
                            .map_err(FulltextError::from)?;
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
    ) -> CoordinatorResult<()> {
        for tag in &vertex.tags {
            for field_name in changed_fields {
                if let Some(value) = tag.properties.get(field_name) {
                    if let Some(engine) = self.manager.get_engine(space_id, &tag.name, field_name) {
                        if let Value::String(text) = value {
                            let doc_id = vertex.vid.to_string();
                            engine.delete(&doc_id).await.map_err(FulltextError::from)?;
                            engine
                                .index(&doc_id, text)
                                .await
                                .map_err(FulltextError::from)?;
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
    ) -> CoordinatorResult<()> {
        let doc_id = ToString::to_string(&vertex_id);

        let indexes = self.manager.get_space_indexes(space_id);
        for metadata in indexes {
            if metadata.tag_name == tag_name {
                if let Some(engine) =
                    self.manager
                        .get_engine(space_id, &metadata.tag_name, &metadata.field_name)
                {
                    engine.delete(&doc_id).await.map_err(FulltextError::from)?;
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
    ) -> CoordinatorResult<()> {
        let doc_id = ToString::to_string(&vertex_id);

        for (field_name, value) in properties {
            if let Some(engine) = self.manager.get_engine(space_id, tag_name, field_name) {
                match change_type {
                    ChangeType::Insert => {
                        if let Value::String(text) = value {
                            engine
                                .index(&doc_id, text)
                                .await
                                .map_err(FulltextError::from)?;
                        }
                    }
                    ChangeType::Update => {
                        engine.delete(&doc_id).await.ok();
                        if let Value::String(text) = value {
                            engine
                                .index(&doc_id, text)
                                .await
                                .map_err(FulltextError::from)?;
                        }
                    }
                    ChangeType::Delete => {
                        engine.delete(&doc_id).await.map_err(FulltextError::from)?;
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
    ) -> CoordinatorResult<Vec<SearchResult>> {
        self.manager
            .search(space_id, tag_name, field_name, query, limit)
            .await
            .map_err(FulltextError::from)
            .map_err(CoordinatorError::from)
    }

    pub fn get_engine(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Option<Arc<dyn crate::search::engine::SearchEngine>> {
        self.manager.get_engine(space_id, tag_name, field_name)
    }

    pub fn get_manager(&self) -> &Arc<FulltextIndexManager> {
        &self.manager
    }

    pub async fn rebuild_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> CoordinatorResult<()> {
        let engine = self
            .manager
            .get_engine(space_id, tag_name, field_name)
            .ok_or_else(|| CoordinatorError::FieldNotIndexed {
                tag_name: tag_name.to_string(),
                field_name: field_name.to_string(),
            })?;

        engine.commit().await.map_err(FulltextError::from)?;
        Ok(())
    }

    pub fn list_indexes(&self) -> Vec<IndexMetadata> {
        self.manager.list_indexes()
    }

    pub async fn commit_all(&self) -> CoordinatorResult<()> {
        self.manager
            .commit_all()
            .await
            .map_err(FulltextError::from)?;
        Ok(())
    }
}
