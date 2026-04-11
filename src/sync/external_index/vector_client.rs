use std::sync::Arc;

use async_trait::async_trait;

use super::error::{ExternalIndexError, IndexResult};
use super::trait_def::{IndexData, IndexStats, ExternalIndexClient};

pub struct VectorClient {
    space_id: u64,
    tag_name: String,
    field_name: String,
    vector_manager: Arc<vector_client::VectorManager>,
}

impl std::fmt::Debug for VectorClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VectorClient")
            .field("space_id", &self.space_id)
            .field("tag_name", &self.tag_name)
            .field("field_name", &self.field_name)
            .finish()
    }
}

impl VectorClient {
    pub fn new(
        space_id: u64,
        tag_name: String,
        field_name: String,
        vector_manager: Arc<vector_client::VectorManager>,
    ) -> Self {
        Self {
            space_id,
            tag_name,
            field_name,
            vector_manager,
        }
    }

    fn collection_name(&self) -> String {
        format!("{}_{}_{}", self.space_id, self.tag_name, self.field_name)
    }
}

#[async_trait]
impl ExternalIndexClient for VectorClient {
    fn client_type(&self) -> &'static str {
        "vector"
    }

    fn index_key(&self) -> (u64, String, String) {
        (self.space_id, self.tag_name.clone(), self.field_name.clone())
    }

    async fn insert(&self, id: &str, data: &IndexData) -> IndexResult<()> {
        if let IndexData::Vector(vector) = data {
            let point = vector_client::types::VectorPoint::new(
                id.to_string(),
                vector.clone(),
            );

            self.vector_manager
                .upsert(&self.collection_name(), point)
                .await
                .map_err(|e: vector_client::error::VectorClientError| ExternalIndexError::InsertError(e.to_string()))
        } else {
            Err(ExternalIndexError::InvalidData(
                "Expected vector data".to_string(),
            ))
        }
    }

    async fn insert_batch(&self, items: Vec<(String, IndexData)>) -> IndexResult<()> {
        let points: Vec<vector_client::types::VectorPoint> = items
            .into_iter()
            .filter_map(|(id, data)| {
                if let IndexData::Vector(vector) = data {
                    Some(vector_client::types::VectorPoint::new(
                        id,
                        vector,
                    ))
                } else {
                    None
                }
            })
            .collect();

        if points.is_empty() {
            return Ok(());
        }

        self.vector_manager
            .upsert_batch(&self.collection_name(), points)
            .await
            .map_err(|e: vector_client::error::VectorClientError| ExternalIndexError::InsertError(e.to_string()))
    }

    async fn delete(&self, id: &str) -> IndexResult<()> {
        self.vector_manager
            .delete(&self.collection_name(), id)
            .await
            .map_err(|e: vector_client::error::VectorClientError| ExternalIndexError::DeleteError(e.to_string()))
    }

    async fn delete_batch(&self, ids: &[&str]) -> IndexResult<()> {
        self.vector_manager
            .delete_batch(&self.collection_name(), ids.to_vec())
            .await
            .map_err(|e: vector_client::error::VectorClientError| ExternalIndexError::DeleteError(e.to_string()))
    }

    async fn commit(&self) -> IndexResult<()> {
        Ok(())
    }

    async fn rollback(&self) -> IndexResult<()> {
        Ok(())
    }

    async fn stats(&self) -> IndexResult<IndexStats> {
        let count = self
            .vector_manager
            .count(&self.collection_name())
            .await
            .map_err(|e: vector_client::error::VectorClientError| ExternalIndexError::StatsError(e.to_string()))?;

        Ok(IndexStats {
            doc_count: count as usize,
            index_size_bytes: 0,
            last_commit_time: None,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
