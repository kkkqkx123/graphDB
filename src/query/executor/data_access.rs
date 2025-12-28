use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use super::base::BaseExecutor;
use crate::expression::ExpressionContext;
use crate::core::Value;
use crate::query::executor::traits::{
    DBResult, ExecutionResult, Executor, HasStorage,
};
use crate::storage::StorageEngine;
use crate::utils::safe_lock;

// Implementation for a basic GetVertices executor
#[derive(Debug)]
pub struct GetVerticesExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    vertex_ids: Option<Vec<Value>>,
    tag_filter: Option<crate::core::Expression>,
    vertex_filter: Option<crate::core::Expression>,
    limit: Option<usize>,
    tag_processor: crate::query::executor::tag_filter::TagFilterProcessor,
}

impl<S: StorageEngine> GetVerticesExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        vertex_ids: Option<Vec<Value>>,
        tag_filter: Option<crate::core::Expression>,
        vertex_filter: Option<crate::core::Expression>,
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
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for GetVerticesExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let vertices = match &self.vertex_ids {
            Some(ids) => {
                let mut result_vertices = Vec::new();
                let storage = safe_lock(self.get_storage())
                    .expect("GetVerticesExecutor storage lock should not be poisoned");

                for id in ids {
                    if let Some(vertex) = storage.get_node(id)? {
                        let include_vertex = if let Some(ref tag_filter_expr) = self.tag_filter {
                            self.tag_processor
                                .process_tag_filter(tag_filter_expr, &vertex)
                        } else {
                            true
                        };

                        if include_vertex {
                            result_vertices.push(vertex.clone());
                        }
                    }

                    if let Some(limit) = self.limit {
                        if result_vertices.len() >= limit {
                            break;
                        }
                    }
                }
                result_vertices
            }
            None => {
                let storage = safe_lock(self.get_storage())
                    .expect("GetVerticesExecutor storage lock should not be poisoned");

                let mut all_vertices = storage.scan_all_vertices()?;

                if let Some(ref tag_filter_expr) = self.tag_filter {
                    all_vertices = all_vertices
                        .into_iter()
                        .filter(|vertex| {
                            self.tag_processor
                                .process_tag_filter(tag_filter_expr, vertex)
                        })
                        .collect();
                }

                if let Some(ref filter_expr) = self.vertex_filter {
                    let evaluator = crate::expression::evaluator::expression_evaluator::ExpressionEvaluator::new();
                    all_vertices = all_vertices
                        .into_iter()
                        .filter(|vertex| {
                            let mut context =
                                crate::expression::DefaultExpressionContext::new();
                            context.set_variable(
                                "vertex".to_string(),
                                crate::core::Value::Vertex(Box::new(vertex.clone())),
                            );

                            match evaluator.evaluate(filter_expr, &mut context) {
                                Ok(value) => {
                                    match value {
                                        crate::core::Value::Bool(b) => b,
                                        crate::core::Value::Int(i) => i != 0,
                                        crate::core::Value::Float(f) => f != 0.0,
                                        crate::core::Value::String(s) => !s.is_empty(),
                                        crate::core::Value::List(l) => !l.is_empty(),
                                        crate::core::Value::Map(m) => !m.is_empty(),
                                        crate::core::Value::Set(s) => !s.is_empty(),
                                        crate::core::Value::Vertex(_) => true,
                                        crate::core::Value::Edge(_) => true,
                                        crate::core::Value::Path(_) => true,
                                        crate::core::Value::Null(_) => false,
                                        crate::core::Value::Empty => false,
                                        crate::core::Value::Date(_) => true,
                                        crate::core::Value::Time(_) => true,
                                        crate::core::Value::DateTime(_) => true,
                                        crate::core::Value::Geography(_) => true,
                                        crate::core::Value::Duration(_) => true,
                                        crate::core::Value::DataSet(ds) => !ds.rows.is_empty(),
                                    }
                                }
                                Err(e) => {
                                    eprintln!("顶点过滤表达式评估失败: {}", e);
                                    false
                                }
                            }
                        })
                        .collect();
                }

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

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        "Get vertices executor - retrieves vertices from storage"
    }
}

impl<S: StorageEngine> HasStorage<S> for GetVerticesExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.storage.as_ref().expect("GetVerticesExecutor storage should be set")
    }
}

// Implementation for a basic GetEdges executor
#[derive(Debug)]
pub struct GetEdgesExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    
    edge_type: Option<String>,
}

impl<S: StorageEngine> GetEdgesExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, edge_type: Option<String>) -> Self {
        Self {
            base: BaseExecutor::new(id, "GetEdgesExecutor".to_string(), storage),
            edge_type,
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for GetEdgesExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let edges: Vec<crate::core::Value> = Vec::new();
        Ok(ExecutionResult::Values(edges))
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        "Get edges executor - retrieves edges from storage"
    }
}

impl<S: StorageEngine> HasStorage<S> for GetEdgesExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.storage.as_ref().expect("GetEdgesExecutor storage should be set")
    }
}



// Implementation for a basic GetNeighbors executor
#[derive(Debug)]
pub struct GetNeighborsExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    
    vertex_ids: Vec<Value>,
    
    edge_direction: super::base::EdgeDirection, // Direction: In, Out, or Both
    
    edge_types: Option<Vec<String>>,
}

impl<S: StorageEngine> GetNeighborsExecutor<S> {
    pub fn new(
        id: i64,
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
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for GetNeighborsExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let neighbors: Vec<crate::core::Value> = Vec::new();
        Ok(ExecutionResult::Values(neighbors))
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        "Get neighbors executor - retrieves neighboring vertices"
    }
}

impl<S: StorageEngine> HasStorage<S> for GetNeighborsExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.storage.as_ref().expect("GetNeighborsExecutor storage should be set")
    }
}



// Implementation for GetPropExecutor
#[derive(Debug)]
pub struct GetPropExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    
    vertex_ids: Option<Vec<Value>>,
    
    edge_ids: Option<Vec<Value>>,
    
    prop_names: Vec<String>, // List of property names to retrieve
}

impl<S: StorageEngine> GetPropExecutor<S> {
    pub fn new(
        id: i64,
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
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for GetPropExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let props: Vec<crate::core::Value> = Vec::new();
        Ok(ExecutionResult::Values(props))
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        "Get property executor - retrieves properties from vertices or edges"
    }
}

impl<S: StorageEngine> HasStorage<S> for GetPropExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.storage.as_ref().expect("GetPropExecutor storage should be set")
    }
}


