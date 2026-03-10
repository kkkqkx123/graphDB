//! 批量任务管理器

use crate::api::server::batch::types::*;
use crate::api::core::{CoreError, CoreResult};
use crate::core::{Edge, Value, Vertex};
use crate::storage::StorageClient;
use dashmap::DashMap;
use parking_lot::Mutex;
use std::sync::Arc;
use uuid::Uuid;

/// 批量任务管理器
pub struct BatchManager<S: StorageClient + Clone + 'static> {
    /// 存储所有批量任务
    tasks: Arc<DashMap<BatchId, BatchTask>>,
    /// 存储客户端
    storage: Arc<Mutex<S>>,
}

impl<S: StorageClient + Clone + 'static> BatchManager<S> {
    /// 创建新的批量任务管理器
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        Self {
            tasks: Arc::new(DashMap::new()),
            storage,
        }
    }

    /// 创建批量任务
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

    /// 获取批量任务
    pub fn get_task(&self, batch_id: &str) -> Option<BatchTask> {
        self.tasks.get(batch_id).map(|t| t.clone())
    }

    /// 添加批量项
    pub fn add_items(&self, batch_id: &str, items: Vec<BatchItem>) -> CoreResult<usize> {
        let mut task = self.tasks
            .get_mut(batch_id)
            .ok_or_else(|| CoreError::InvalidParameter(format!("批量任务不存在: {}", batch_id)))?;

        if task.status != BatchStatus::Created {
            return Err(CoreError::InvalidParameter(
                format!("批量任务状态不正确: {:?}", task.status)
            ));
        }

        let count = task.add_items(items);
        Ok(count)
    }

    /// 执行批量任务
    pub async fn execute_task(&self, batch_id: &str, space_name: &str) -> CoreResult<BatchResultData> {
        let task = self.tasks
            .get(batch_id)
            .ok_or_else(|| CoreError::InvalidParameter(format!("批量任务不存在: {}", batch_id)))?;

        if task.status != BatchStatus::Created {
            return Err(CoreError::InvalidParameter(
                format!("批量任务状态不正确: {:?}", task.status)
            ));
        }

        // 更新状态为运行中
        {
            let mut task = self.tasks
                .get_mut(batch_id)
                .expect("任务应该存在");
            task.update_status(BatchStatus::Running);
        }

        // 获取所有缓冲的项
        let items = {
            let mut task = self.tasks
                .get_mut(batch_id)
                .expect("任务应该存在");
            task.take_buffered_items()
        };

        // 执行批量插入
        let result = self.process_items(items, space_name).await;

        // 更新任务状态和结果
        {
            let mut task = self.tasks
                .get_mut(batch_id)
                .expect("任务应该存在");
            
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

    /// 取消批量任务
    pub fn cancel_task(&self, batch_id: &str) -> CoreResult<()> {
        let mut task = self.tasks
            .get_mut(batch_id)
            .ok_or_else(|| CoreError::InvalidParameter(format!("批量任务不存在: {}", batch_id)))?;

        match task.status {
            BatchStatus::Created | BatchStatus::Running => {
                task.update_status(BatchStatus::Cancelled);
                Ok(())
            }
            _ => Err(CoreError::InvalidParameter(
                format!("无法取消状态为 {:?} 的任务", task.status)
            )),
        }
    }

    /// 删除批量任务
    pub fn remove_task(&self, batch_id: &str) -> CoreResult<()> {
        self.tasks
            .remove(batch_id)
            .ok_or_else(|| CoreError::InvalidParameter(format!("批量任务不存在: {}", batch_id)))?;
        Ok(())
    }

    /// 处理批量项
    async fn process_items(
        &self,
        items: Vec<BatchItem>,
        space_name: &str,
    ) -> CoreResult<BatchResultData> {
        let mut vertices = Vec::new();
        let mut edges = Vec::new();

        // 分类顶点与边
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

        // 批量插入顶点
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

        // 批量插入边
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

    /// 转换顶点数据
    fn convert_vertex_data(&self, data: VertexData) -> Option<Vertex> {
        let vid = json_to_value(data.vid)?;
        
        // 构建标签列表
        let tags: Vec<crate::core::vertex_edge_path::Tag> = data.tags
            .into_iter()
            .map(|name| crate::core::vertex_edge_path::Tag::new(name, std::collections::HashMap::new()))
            .collect();

        // 转换属性
        let properties: std::collections::HashMap<String, Value> = data.properties
            .into_iter()
            .filter_map(|(k, v)| json_to_value(v).map(|val| (k, val)))
            .collect();

        Some(Vertex::new_with_properties(vid, tags, properties))
    }

    /// 转换边数据
    fn convert_edge_data(&self, data: EdgeData) -> Option<Edge> {
        let src_vid = json_to_value(data.src_vid)?;
        let dst_vid = json_to_value(data.dst_vid)?;
        
        // 转换属性
        let props: std::collections::HashMap<String, Value> = data.properties
            .into_iter()
            .filter_map(|(k, v)| json_to_value(v).map(|val| (k, val)))
            .collect();

        Some(Edge::new(
            src_vid,
            dst_vid,
            data.edge_type,
            0, // ranking 默认为0
            props,
        ))
    }

    /// 插入顶点
    async fn insert_vertices(
        &self,
        space_name: &str,
        vertices: Vec<Vertex>,
    ) -> CoreResult<usize> {
        let count = vertices.len();

        let mut storage = self.storage.lock();
        match storage.batch_insert_vertices(space_name, vertices) {
            Ok(_) => Ok(count),
            Err(e) => Err(CoreError::StorageError(e.to_string())),
        }
    }

    /// 插入边
    async fn insert_edges(
        &self,
        space_name: &str,
        edges: Vec<Edge>,
    ) -> CoreResult<usize> {
        let count = edges.len();

        let mut storage = self.storage.lock();
        match storage.batch_insert_edges(space_name, edges) {
            Ok(_) => Ok(count),
            Err(e) => Err(CoreError::StorageError(e.to_string())),
        }
    }
}

/// 将JSON值转换为Core Value
fn json_to_value(json: serde_json::Value) -> Option<Value> {
    match json {
        serde_json::Value::Null => Some(Value::Null(crate::core::NullType::Null)),
        serde_json::Value::Bool(b) => Some(Value::Bool(b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(Value::Int(i))
            } else if let Some(f) = n.as_f64() {
                Some(Value::Float(f))
            } else {
                None
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