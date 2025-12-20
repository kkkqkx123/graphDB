use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use super::base::BaseExecutor;
use crate::core::Value;
use crate::expression::context::ExpressionContextCore;
use crate::query::executor::traits::{
    DBResult, ExecutionResult, Executor, ExecutorCore, ExecutorLifecycle, ExecutorMetadata,
};
use crate::storage::StorageEngine;
use crate::utils::safe_lock;

// Implementation for a basic GetVertices executor
#[derive(Debug)]
pub struct GetVerticesExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    vertex_ids: Option<Vec<Value>>,
    tag_filter: Option<crate::expression::Expression>,
    vertex_filter: Option<crate::expression::Expression>,
    limit: Option<usize>,
    tag_processor: crate::query::executor::tag_filter::TagFilterProcessor,
}

impl<S: StorageEngine> GetVerticesExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        vertex_ids: Option<Vec<Value>>,
        tag_filter: Option<crate::expression::Expression>,
        vertex_filter: Option<crate::expression::Expression>,
        limit: Option<usize>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "GetVerticesExecutor".to_string(), storage),
            vertex_ids,
            tag_filter,
            vertex_filter,
            limit,
            tag_processor: crate::query::executor::tag_filter::TagFilterProcessor::new(),
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ExecutorCore for GetVerticesExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let vertices = match &self.vertex_ids {
            Some(ids) => {
                // 获取特定顶点ID的顶点
                let mut result_vertices = Vec::new();
                let storage = safe_lock(&self.base.storage)
                    .expect("GetVerticesExecutor storage lock should not be poisoned");

                for id in ids {
                    if let Some(vertex) = storage.get_node(id)? {
                        // 应用标签过滤表达式（如果存在）
                        let include_vertex = if let Some(ref tag_filter_expr) = self.tag_filter {
                            self.tag_processor
                                .process_tag_filter(tag_filter_expr, &vertex)
                        } else {
                            true // 没有标签过滤器，包含所有顶点
                        };

                        if include_vertex {
                            result_vertices.push(vertex.clone());
                        }
                    }

                    // Apply limit if specified
                    if let Some(limit) = self.limit {
                        if result_vertices.len() >= limit {
                            break;
                        }
                    }
                }
                result_vertices
            }
            None => {
                // ScanVertices操作：扫描所有顶点
                let storage = safe_lock(&self.base.storage)
                    .expect("GetVerticesExecutor storage lock should not be poisoned");

                // 获取所有顶点
                let mut all_vertices = storage.scan_all_vertices()?;

                // 应用标签过滤表达式
                if let Some(ref tag_filter_expr) = self.tag_filter {
                    all_vertices = all_vertices
                        .into_iter()
                        .filter(|vertex| {
                            self.tag_processor
                                .process_tag_filter(tag_filter_expr, vertex)
                        })
                        .collect();
                }

                // 应用顶点过滤表达式
                if let Some(ref filter_expr) = self.vertex_filter {
                    let evaluator = crate::expression::ExpressionEvaluator::new();
                    all_vertices = all_vertices
                        .into_iter()
                        .filter(|vertex| {
                            // 创建评估上下文
                            let mut context = crate::expression::ExpressionContext::default();
                            context.set_variable(
                                "vertex".to_string(),
                                crate::core::Value::Vertex(Box::new(vertex.clone())),
                            );

                            // 评估过滤表达式
                            match evaluator.evaluate(filter_expr, &context) {
                                Ok(value) => {
                                    // 将 Value 转换为 bool
                                    match value {
                                        crate::core::Value::Bool(b) => b,
                                        crate::core::Value::Int(i) => i != 0,
                                        crate::core::Value::Float(f) => f != 0.0,
                                        crate::core::Value::String(s) => !s.is_empty(),
                                        crate::core::Value::List(l) => !l.is_empty(),
                                        crate::core::Value::Map(m) => !m.is_empty(),
                                        crate::core::Value::Set(s) => !s.is_empty(),
                                        crate::core::Value::Vertex(_) => true, // 顶点对象视为true
                                        crate::core::Value::Edge(_) => true,   // 边对象视为true
                                        crate::core::Value::Path(_) => true,   // 路径对象视为true
                                        crate::core::Value::Null(_) => false,  // null视为false
                                        crate::core::Value::Empty => false,    // empty视为false
                                        crate::core::Value::Date(_) => true,   // 日期对象视为true
                                        crate::core::Value::Time(_) => true,   // 时间对象视为true
                                        crate::core::Value::DateTime(_) => true, // 日期时间对象视为true
                                        crate::core::Value::Geography(_) => true, // 地理对象视为true
                                        crate::core::Value::Duration(_) => true, // 持续时间对象视为true
                                        crate::core::Value::DataSet(ds) => !ds.rows.is_empty(), // 数据集非空视为true
                                    }
                                }
                                Err(e) => {
                                    eprintln!("顶点过滤表达式评估失败: {}", e);
                                    false // 过滤失败时默认排除该顶点
                                }
                            }
                        })
                        .collect();
                }

                // 应用limit限制
                if let Some(limit) = self.limit {
                    all_vertices.into_iter().take(limit).collect()
                } else {
                    all_vertices
                }
            }
        };

        Ok(ExecutionResult::Values(
            vertices
                .into_iter()
                .map(|v| crate::core::Value::Vertex(Box::new(v)))
                .collect(),
        ))
    }
}

impl<S: StorageEngine> ExecutorLifecycle for GetVerticesExecutor<S> {
    fn open(&mut self) -> DBResult<()> {
        // Initialize any resources needed for vertex retrieval
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        // Clean up any resources
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }
}

impl<S: StorageEngine> ExecutorMetadata for GetVerticesExecutor<S> {
    fn id(&self) -> usize {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        "Get vertices executor - retrieves vertices from storage"
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for GetVerticesExecutor<S> {
    fn storage(&self) -> &Arc<Mutex<S>> {
        &self.base.storage
    }
}

// Implementation for a basic GetEdges executor
#[derive(Debug)]
pub struct GetEdgesExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    #[allow(dead_code)]
    edge_type: Option<String>,
}

impl<S: StorageEngine> GetEdgesExecutor<S> {
    pub fn new(id: usize, storage: Arc<Mutex<S>>, edge_type: Option<String>) -> Self {
        Self {
            base: BaseExecutor::new(id, "GetEdgesExecutor".to_string(), storage),
            edge_type,
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ExecutorCore for GetEdgesExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        // In a real implementation, this would fetch edges based on the edge_type
        // For now return empty list
        let edges: Vec<crate::core::Value> = Vec::new();

        Ok(ExecutionResult::Values(edges))
    }
}

impl<S: StorageEngine> ExecutorLifecycle for GetEdgesExecutor<S> {
    fn open(&mut self) -> DBResult<()> {
        // Initialize any resources needed for edge retrieval
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        // Clean up any resources
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }
}

impl<S: StorageEngine> ExecutorMetadata for GetEdgesExecutor<S> {
    fn id(&self) -> usize {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        "Get edges executor - retrieves edges from storage"
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for GetEdgesExecutor<S> {
    fn storage(&self) -> &Arc<Mutex<S>> {
        &self.base.storage
    }
}

// Implementation for a basic GetNeighbors executor
#[derive(Debug)]
pub struct GetNeighborsExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    #[allow(dead_code)]
    vertex_ids: Vec<Value>,
    #[allow(dead_code)]
    edge_direction: super::base::EdgeDirection, // Direction: In, Out, or Both
    #[allow(dead_code)]
    edge_types: Option<Vec<String>>,
}

impl<S: StorageEngine> GetNeighborsExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        vertex_ids: Vec<Value>,
        edge_direction: super::base::EdgeDirection,
        edge_types: Option<Vec<String>>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "GetNeighborsExecutor".to_string(), storage),
            vertex_ids,
            edge_direction,
            edge_types,
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ExecutorCore for GetNeighborsExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        // In a real implementation, this would fetch neighboring vertices based on edges
        // For now return empty list
        let neighbors: Vec<crate::core::Value> = Vec::new();

        Ok(ExecutionResult::Values(neighbors))
    }
}

impl<S: StorageEngine> ExecutorLifecycle for GetNeighborsExecutor<S> {
    fn open(&mut self) -> DBResult<()> {
        // Initialize any resources needed for neighbor retrieval
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        // Clean up any resources
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }
}

impl<S: StorageEngine> ExecutorMetadata for GetNeighborsExecutor<S> {
    fn id(&self) -> usize {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        "Get neighbors executor - retrieves neighboring vertices"
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for GetNeighborsExecutor<S> {
    fn storage(&self) -> &Arc<Mutex<S>> {
        &self.base.storage
    }
}

// Implementation for GetPropExecutor
#[derive(Debug)]
pub struct GetPropExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    #[allow(dead_code)]
    vertex_ids: Option<Vec<Value>>,
    #[allow(dead_code)]
    edge_ids: Option<Vec<Value>>,
    #[allow(dead_code)]
    prop_names: Vec<String>, // List of property names to retrieve
}

impl<S: StorageEngine> GetPropExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        vertex_ids: Option<Vec<Value>>,
        edge_ids: Option<Vec<Value>>,
        prop_names: Vec<String>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "GetPropExecutor".to_string(), storage),
            vertex_ids,
            edge_ids,
            prop_names,
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ExecutorCore for GetPropExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        // In a real implementation, this would fetch specific properties from vertices or edges
        // For now, return empty list
        let props: Vec<crate::core::Value> = Vec::new();

        Ok(ExecutionResult::Values(props))
    }
}

impl<S: StorageEngine> ExecutorLifecycle for GetPropExecutor<S> {
    fn open(&mut self) -> DBResult<()> {
        // Initialize any resources needed for property retrieval
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        // Clean up any resources
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }
}

impl<S: StorageEngine> ExecutorMetadata for GetPropExecutor<S> {
    fn id(&self) -> usize {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        "Get property executor - retrieves properties from vertices or edges"
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for GetPropExecutor<S> {
    fn storage(&self) -> &Arc<Mutex<S>> {
        &self.base.storage
    }
}
