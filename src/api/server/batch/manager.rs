//! Batch Task Manager

use crate::api::core::{CoreError, CoreResult};
use crate::api::server::batch::types::*;
use crate::core::{Edge, Value, Vertex};
use crate::storage::StorageClient;
use dashmap::DashMap;
use parking_lot::Mutex;
use std::sync::Arc;
use uuid::Uuid;

/// Batch Task Manager
pub struct BatchManager<S: StorageClient + Clone + 'static> {
    /// Store all batch jobs
    tasks: Arc<DashMap<BatchId, BatchTask>>,
    /// Storage Client
    storage: Arc<Mutex<S>>,
}

impl<S: StorageClient + Clone + 'static> BatchManager<S> {
    /// Creating a new batch task manager
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        Self {
            tasks: Arc::new(DashMap::new()),
            storage,
        }
    }

    /// Creating Batch Tasks
    pub fn create_task(
        &self,
        space_id: u64,
        batch_type: BatchType,
        batch_size: usize,
    ) -> CoreResult<BatchTask> {
        let batch_id = Uuid::new_v4().to_string();
        let task = BatchTask::new(batch_id.clone(), space_id, batch_type, batch_size);

        self.tasks.insert(batch_id.clone(), task.clone());

        Ok(task)
    }

    /// Get Batch Tasks
    pub fn get_task(&self, batch_id: &str) -> Option<BatchTask> {
        self.tasks.get(batch_id).map(|t| t.clone())
    }

    /// Adding Batch Items
    pub fn add_items(&self, batch_id: &str, items: Vec<BatchItem>) -> CoreResult<usize> {
        let mut task = self
            .tasks
            .get_mut(batch_id)
            .ok_or_else(|| CoreError::InvalidParameter(format!("批量任务不存在: {}", batch_id)))?;

        if task.status != BatchStatus::Created {
            return Err(CoreError::InvalidParameter(format!(
                "批量任务状态不正确: {:?}",
                task.status
            )));
        }

        let count = task.add_items(items);
        Ok(count)
    }

    /// Perform batch tasks
    pub async fn execute_task(
        &self,
        batch_id: &str,
        space_name: &str,
    ) -> CoreResult<BatchResultData> {
        let task = self
            .tasks
            .get(batch_id)
            .ok_or_else(|| CoreError::InvalidParameter(format!("批量任务不存在: {}", batch_id)))?;

        if task.status != BatchStatus::Created {
            return Err(CoreError::InvalidParameter(format!(
                "批量任务状态不正确: {:?}",
                task.status
            )));
        }

        // Update status to running
        {
            let mut task = self.tasks.get_mut(batch_id).expect("任务应该存在");
            task.update_status(BatchStatus::Running);
        }

        // Get all buffered items
        let items = {
            let mut task = self.tasks.get_mut(batch_id).expect("任务应该存在");
            task.take_buffered_items()
        };

        // Perform batch insertion
        let result = self.process_items(items, space_name).await;

        // Update task status and results
        {
            let mut task = self.tasks.get_mut(batch_id).expect("任务应该存在");

            match &result {
                Ok(data) => {
                    let status = if data.errors.is_empty() {
                        BatchStatus::Completed
                    } else {
                        BatchStatus::Failed
                    };
                    task.update_status(status);
                    task.set_result(data.clone());
                }
                Err(e) => {
                    task.update_status(BatchStatus::Failed);
                    task.set_result(BatchResultData {
                        vertices_inserted: 0,
                        edges_inserted: 0,
                        errors: vec![BatchErrorData {
                            index: 0,
                            item_type: BatchItemType::Vertex,
                            error: e.to_string(),
                        }],
                    });
                }
            }
        }

        result
    }

    /// Cancel Batch Tasks
    pub fn cancel_task(&self, batch_id: &str) -> CoreResult<()> {
        let mut task = self
            .tasks
            .get_mut(batch_id)
            .ok_or_else(|| CoreError::InvalidParameter(format!("批量任务不存在: {}", batch_id)))?;

        match task.status {
            BatchStatus::Created | BatchStatus::Running => {
                task.update_status(BatchStatus::Cancelled);
                Ok(())
            }
            _ => Err(CoreError::InvalidParameter(format!(
                "无法取消状态为 {:?} 的任务",
                task.status
            ))),
        }
    }

    /// Delete Batch Tasks
    pub fn remove_task(&self, batch_id: &str) -> CoreResult<()> {
        self.tasks
            .remove(batch_id)
            .ok_or_else(|| CoreError::InvalidParameter(format!("批量任务不存在: {}", batch_id)))?;
        Ok(())
    }

    /// Processing of batch items
    async fn process_items(
        &self,
        items: Vec<BatchItem>,
        space_name: &str,
    ) -> CoreResult<BatchResultData> {
        let mut vertices = Vec::new();
        let mut edges = Vec::new();

        // Categorize vertices and edges
        for item in items {
            match item {
                BatchItem::Vertex(data) => {
                    if let Some(vertex) = self.convert_vertex_data(data) {
                        vertices.push(vertex);
                    }
                }
                BatchItem::Edge(data) => {
                    if let Some(edge) = self.convert_edge_data(data) {
                        edges.push(edge);
                    }
                }
            }
        }

        let mut result = BatchResultData {
            vertices_inserted: 0,
            edges_inserted: 0,
            errors: Vec::new(),
        };

        // Batch insertion of vertices
        if !vertices.is_empty() {
            match self.insert_vertices(space_name, vertices).await {
                Ok(count) => {
                    result.vertices_inserted = count;
                }
                Err(e) => {
                    result.errors.push(BatchErrorData {
                        index: 0,
                        item_type: BatchItemType::Vertex,
                        error: format!("批量插入顶点失败: {}", e),
                    });
                }
            }
        }

        // Batch insertion of edges
        if !edges.is_empty() {
            match self.insert_edges(space_name, edges).await {
                Ok(count) => {
                    result.edges_inserted = count;
                }
                Err(e) => {
                    result.errors.push(BatchErrorData {
                        index: 0,
                        item_type: BatchItemType::Edge,
                        error: format!("批量插入边失败: {}", e),
                    });
                }
            }
        }

        Ok(result)
    }

    /// Converting Vertex Data
    fn convert_vertex_data(&self, data: VertexData) -> Option<Vertex> {
        let vid = json_to_value(data.vid)?;

        // Building a list of tags
        let tags: Vec<crate::core::vertex_edge_path::Tag> = data
            .tags
            .into_iter()
            .map(|name| {
                crate::core::vertex_edge_path::Tag::new(name, std::collections::HashMap::new())
            })
            .collect();

        // Converting Attributes
        let properties: std::collections::HashMap<String, Value> = data
            .properties
            .into_iter()
            .filter_map(|(k, v)| json_to_value(v).map(|val| (k, val)))
            .collect();

        Some(Vertex::new_with_properties(vid, tags, properties))
    }

    /// Conversion side data
    fn convert_edge_data(&self, data: EdgeData) -> Option<Edge> {
        let src_vid = json_to_value(data.src_vid)?;
        let dst_vid = json_to_value(data.dst_vid)?;

        // Converting Attributes
        let props: std::collections::HashMap<String, Value> = data
            .properties
            .into_iter()
            .filter_map(|(k, v)| json_to_value(v).map(|val| (k, val)))
            .collect();

        Some(Edge::new(
            src_vid,
            dst_vid,
            data.edge_type,
            0, // Ranking defaults to 0
            props,
        ))
    }

    /// Insert vertex
    async fn insert_vertices(&self, space_name: &str, vertices: Vec<Vertex>) -> CoreResult<usize> {
        let count = vertices.len();

        let mut storage = self.storage.lock();
        match storage.batch_insert_vertices(space_name, vertices) {
            Ok(_) => Ok(count),
            Err(e) => Err(CoreError::StorageError(e.to_string())),
        }
    }

    /// insertion side
    async fn insert_edges(&self, space_name: &str, edges: Vec<Edge>) -> CoreResult<usize> {
        let count = edges.len();

        let mut storage = self.storage.lock();
        match storage.batch_insert_edges(space_name, edges) {
            Ok(_) => Ok(count),
            Err(e) => Err(CoreError::StorageError(e.to_string())),
        }
    }
}

/// Converting JSON Values to Core Value
fn json_to_value(json: serde_json::Value) -> Option<Value> {
    match json {
        serde_json::Value::Null => Some(Value::Null(crate::core::NullType::Null)),
        serde_json::Value::Bool(b) => Some(Value::Bool(b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(Value::Int(i))
            } else {
                n.as_f64().map(Value::Float)
            }
        }
        serde_json::Value::String(s) => Some(Value::String(s)),
        serde_json::Value::Array(arr) => {
            let values: Vec<Value> = arr.into_iter().filter_map(json_to_value).collect();
            Some(Value::List(crate::core::List::from(values)))
        }
        serde_json::Value::Object(map) => {
            let result: std::collections::HashMap<String, Value> = map
                .into_iter()
                .filter_map(|(k, v)| json_to_value(v).map(|val| (k, val)))
                .collect();
            Some(Value::Map(result))
        }
    }
}
