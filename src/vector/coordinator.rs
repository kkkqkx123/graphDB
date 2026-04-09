//! Vector Coordinator
//!
//! Coordinates vector index operations with graph data changes.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::core::error::{VectorCoordinatorError, VectorCoordinatorResult};
use crate::core::{Value, Vertex};
use crate::vector::config::{VectorIndexConfig, VectorIndexMetadata};
use crate::vector::embedding::EmbeddingServiceHandle;
use crate::vector::manager::VectorIndexManager;

use vector_client::types::{SearchQuery, SearchResult, VectorFilter, VectorPoint};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VectorChangeType {
    Insert,
    Update,
    Delete,
}

impl From<crate::coordinator::ChangeType> for VectorChangeType {
    fn from(ct: crate::coordinator::ChangeType) -> Self {
        match ct {
            crate::coordinator::ChangeType::Insert => VectorChangeType::Insert,
            crate::coordinator::ChangeType::Update => VectorChangeType::Update,
            crate::coordinator::ChangeType::Delete => VectorChangeType::Delete,
        }
    }
}

/// 向量索引位置标识
///
/// 用于唯一标识一个向量索引字段，在向量变更、索引管理等场景中复用
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VectorIndexLocation {
    pub space_id: u64,
    pub tag_name: String,
    pub field_name: String,
}

impl VectorIndexLocation {
    pub fn new(space_id: u64, tag_name: impl Into<String>, field_name: impl Into<String>) -> Self {
        Self {
            space_id,
            tag_name: tag_name.into(),
            field_name: field_name.into(),
        }
    }

    /// 生成集合名称（用于 Qdrant 等向量引擎）
    pub fn to_collection_name(&self) -> String {
        format!(
            "space_{}_{}_{}",
            self.space_id, self.tag_name, self.field_name
        )
    }
}

/// 向量变更数据
///
/// 封装向量变更操作的核心数据，与具体的索引位置无关
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorChangeData {
    pub vertex_id: Value,
    pub vector: Option<Vec<f32>>,
    pub payload: HashMap<String, Value>,
}

impl VectorChangeData {
    pub fn new(
        vertex_id: Value,
        vector: Option<Vec<f32>>,
        payload: HashMap<String, Value>,
    ) -> Self {
        Self {
            vertex_id,
            vector,
            payload,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VectorChangeContext {
    pub location: VectorIndexLocation,
    pub data: VectorChangeData,
    pub change_type: VectorChangeType,
}

impl VectorChangeContext {
    pub fn new(
        space_id: u64,
        tag_name: impl Into<String>,
        field_name: impl Into<String>,
        vertex_id: Value,
        vector: Option<Vec<f32>>,
        payload: HashMap<String, Value>,
        change_type: VectorChangeType,
    ) -> Self {
        Self {
            location: VectorIndexLocation::new(space_id, tag_name, field_name),
            data: VectorChangeData::new(vertex_id, vector, payload),
            change_type,
        }
    }

    /// 便捷访问方法
    pub fn space_id(&self) -> u64 {
        self.location.space_id
    }

    pub fn tag_name(&self) -> &str {
        &self.location.tag_name
    }

    pub fn field_name(&self) -> &str {
        &self.location.field_name
    }

    pub fn vertex_id(&self) -> &Value {
        &self.data.vertex_id
    }

    pub fn vector(&self) -> Option<&Vec<f32>> {
        self.data.vector.as_ref()
    }

    pub fn payload(&self) -> &HashMap<String, Value> {
        &self.data.payload
    }
}

#[derive(Debug)]
pub struct VectorCoordinator {
    manager: Arc<VectorIndexManager>,
    embedding_service: Option<EmbeddingServiceHandle>,
}

impl VectorCoordinator {
    pub fn new(manager: Arc<VectorIndexManager>) -> Self {
        Self {
            manager,
            embedding_service: None,
        }
    }

    pub fn with_embedding_service(mut self, service: EmbeddingServiceHandle) -> Self {
        self.embedding_service = Some(service);
        self
    }

    pub fn set_embedding_service(&mut self, service: EmbeddingServiceHandle) {
        self.embedding_service = Some(service);
    }

    pub fn embedding_service(&self) -> Option<&EmbeddingServiceHandle> {
        self.embedding_service.as_ref()
    }

    pub async fn create_vector_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        vector_size: usize,
        distance: crate::vector::config::VectorDistance,
    ) -> VectorCoordinatorResult<String> {
        let config = VectorIndexConfig {
            vector_size,
            distance,
            hnsw: None,
            quantization: None,
        };

        self.manager
            .create_index(space_id, tag_name, field_name, Some(config))
            .await
            .map_err(|e| VectorCoordinatorError::IndexCreationFailed {
                tag_name: tag_name.to_string(),
                field_name: field_name.to_string(),
                reason: e.to_string(),
            })?;

        Ok(format!("{}_{}_{}", space_id, tag_name, field_name))
    }

    pub async fn create_vector_index_with_config(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        config: VectorIndexConfig,
    ) -> VectorCoordinatorResult<String> {
        self.manager
            .create_index(space_id, tag_name, field_name, Some(config))
            .await
            .map_err(|e| VectorCoordinatorError::IndexCreationFailed {
                tag_name: tag_name.to_string(),
                field_name: field_name.to_string(),
                reason: e.to_string(),
            })?;

        Ok(format!("{}_{}_{}", space_id, tag_name, field_name))
    }

    pub async fn drop_vector_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> VectorCoordinatorResult<()> {
        self.manager
            .drop_index(space_id, tag_name, field_name)
            .await
            .map_err(|e| VectorCoordinatorError::IndexDropFailed {
                tag_name: tag_name.to_string(),
                field_name: field_name.to_string(),
                reason: e.to_string(),
            })?;

        Ok(())
    }

    pub async fn on_vertex_inserted(
        &self,
        space_id: u64,
        vertex: &Vertex,
    ) -> VectorCoordinatorResult<()> {
        for tag in &vertex.tags {
            for (field_name, value) in &tag.properties {
                if self.manager.index_exists(space_id, &tag.name, field_name) {
                    if let Some(vector) = value.as_vector() {
                        let point_id = format!("{}", vertex.vid);
                        let mut payload = HashMap::new();
                        payload.insert(
                            "vertex_id".to_string(),
                            serde_json::to_value(&vertex.vid).unwrap_or(serde_json::Value::Null),
                        );

                        let point = VectorPoint::new(point_id, vector).with_payload(payload);

                        self.manager
                            .upsert(space_id, &tag.name, field_name, point)
                            .await?;
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
    ) -> VectorCoordinatorResult<()> {
        for tag in &vertex.tags {
            for field_name in changed_fields {
                if let Some(value) = tag.properties.get(field_name) {
                    if self.manager.index_exists(space_id, &tag.name, field_name) {
                        let point_id = format!("{}", vertex.vid);

                        if let Some(vector) = value.as_vector() {
                            let mut payload = HashMap::new();
                            payload.insert(
                                "vertex_id".to_string(),
                                serde_json::to_value(&vertex.vid)
                                    .unwrap_or(serde_json::Value::Null),
                            );

                            let point = VectorPoint::new(point_id, vector).with_payload(payload);

                            self.manager
                                .upsert(space_id, &tag.name, field_name, point)
                                .await?;
                        } else {
                            self.manager
                                .delete(space_id, &tag.name, field_name, &point_id)
                                .await?;
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
    ) -> VectorCoordinatorResult<()> {
        let point_id = format!("{}", vertex_id);

        let indexes = self.manager.list_indexes();
        for metadata in indexes {
            if metadata.space_id == space_id && metadata.tag_name == tag_name {
                self.manager
                    .delete(
                        space_id,
                        &metadata.tag_name,
                        &metadata.field_name,
                        &point_id,
                    )
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn on_vector_change(&self, ctx: VectorChangeContext) -> VectorCoordinatorResult<()> {
        let point_id = format!("{}", ctx.data.vertex_id);

        match ctx.change_type {
            VectorChangeType::Insert | VectorChangeType::Update => {
                if let Some(vec) = ctx.data.vector {
                    let json_payload: HashMap<String, serde_json::Value> = ctx
                        .data
                        .payload
                        .into_iter()
                        .filter_map(|(k, v)| serde_json::to_value(&v).ok().map(|json| (k, json)))
                        .collect();

                    let point = VectorPoint::new(point_id, vec).with_payload(json_payload);

                    self.manager
                        .upsert(
                            ctx.location.space_id,
                            &ctx.location.tag_name,
                            &ctx.location.field_name,
                            point,
                        )
                        .await?;
                }
            }
            VectorChangeType::Delete => {
                self.manager
                    .delete(
                        ctx.location.space_id,
                        &ctx.location.tag_name,
                        &ctx.location.field_name,
                        &point_id,
                    )
                    .await?;
            }
        }

        Ok(())
    }

    pub async fn search(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> VectorCoordinatorResult<Vec<SearchResult>> {
        let query = SearchQuery::new(query_vector, limit);

        let results = self
            .manager
            .search(space_id, tag_name, field_name, query)
            .await?;

        Ok(results)
    }

    pub async fn search_with_filter(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        query_vector: Vec<f32>,
        limit: usize,
        filter: VectorFilter,
    ) -> VectorCoordinatorResult<Vec<SearchResult>> {
        let query = SearchQuery::new(query_vector, limit).with_filter(filter);

        let results = self
            .manager
            .search(space_id, tag_name, field_name, query)
            .await?;

        Ok(results)
    }

    pub async fn search_with_threshold(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        query_vector: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> VectorCoordinatorResult<Vec<SearchResult>> {
        let query = SearchQuery::new(query_vector, limit).with_score_threshold(threshold);

        let results = self
            .manager
            .search(space_id, tag_name, field_name, query)
            .await?;

        Ok(results)
    }

    pub async fn search_with_threshold_and_filter(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        query_vector: Vec<f32>,
        limit: usize,
        threshold: f32,
        filter: VectorFilter,
    ) -> VectorCoordinatorResult<Vec<SearchResult>> {
        let query = SearchQuery::new(query_vector, limit)
            .with_score_threshold(threshold)
            .with_filter(filter);

        let results = self
            .manager
            .search(space_id, tag_name, field_name, query)
            .await?;

        Ok(results)
    }

    pub fn get_engine(&self) -> &Arc<dyn vector_client::VectorEngine> {
        self.manager.get_engine()
    }

    pub fn get_manager(&self) -> &Arc<VectorIndexManager> {
        &self.manager
    }

    pub fn index_exists(&self, space_id: u64, tag_name: &str, field_name: &str) -> bool {
        self.manager.index_exists(space_id, tag_name, field_name)
    }

    pub fn get_index_metadata(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Option<VectorIndexMetadata> {
        self.manager.get_metadata(space_id, tag_name, field_name)
    }

    pub fn list_indexes(&self) -> Vec<VectorIndexMetadata> {
        self.manager.list_indexes()
    }

    pub async fn health_check(&self) -> VectorCoordinatorResult<bool> {
        self.manager.health_check().await.map_err(Into::into)
    }

    /// Convert text to vector using embedding service
    pub async fn embed_text(&self, text: &str) -> VectorCoordinatorResult<Vec<f32>> {
        if let Some(embedding_service) = &self.embedding_service {
            embedding_service
                .embed(text)
                .await
                .map_err(|e| VectorCoordinatorError::Internal(e.to_string()))
        } else {
            Err(VectorCoordinatorError::EmbeddingServiceNotAvailable)
        }
    }

    pub async fn upsert_batch(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        points: Vec<VectorPoint>,
    ) -> VectorCoordinatorResult<()> {
        self.manager
            .upsert_batch(space_id, tag_name, field_name, points)
            .await?;
        Ok(())
    }

    pub async fn delete_batch(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        point_ids: Vec<&str>,
    ) -> VectorCoordinatorResult<()> {
        self.manager
            .delete_batch(space_id, tag_name, field_name, point_ids)
            .await?;
        Ok(())
    }
}
