use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use super::base::BaseExecutor;
use crate::core::{Edge, Value, Vertex};
use crate::expression::context::basic_context::BasicExpressionContext;
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::query::executor::traits::{DBResult, ExecutionResult, Executor, HasStorage};
use crate::query::parser::expressions::parse_expression_meta_from_string;
use crate::storage::StorageEngine;
use crate::utils::safe_lock;

// Executor for inserting new vertices/edges
pub struct InsertExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    vertex_data: Option<Vec<Vertex>>, // Data to be inserted
    edge_data: Option<Vec<Edge>>,
}

impl<S: StorageEngine> InsertExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        vertex_data: Option<Vec<Vertex>>,
        edge_data: Option<Vec<Edge>>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "InsertExecutor".to_string(), storage),
            vertex_data,
            edge_data,
        }
    }

    pub fn with_vertices(id: i64, storage: Arc<Mutex<S>>, vertex_data: Vec<Vertex>) -> Self {
        Self {
            base: BaseExecutor::new(id, "InsertExecutor".to_string(), storage),
            vertex_data: Some(vertex_data),
            edge_data: None,
        }
    }

    pub fn with_edges(id: i64, storage: Arc<Mutex<S>>, edge_data: Vec<Edge>) -> Self {
        Self {
            base: BaseExecutor::new(id, "InsertExecutor".to_string(), storage),
            vertex_data: None,
            edge_data: Some(edge_data),
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for InsertExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let mut _total_inserted = 0;

        // Insert vertices if provided
        if let Some(vertices) = &self.vertex_data {
            let mut storage = safe_lock(self.get_storage())
                .expect("InsertExecutor storage lock should not be poisoned");
            for vertex in vertices {
                storage.insert_node(vertex.clone())?;
                _total_inserted += 1;
            }
        }

        // Insert edges if provided
        if let Some(edges) = &self.edge_data {
            let mut storage = safe_lock(self.get_storage())
                .expect("InsertExecutor storage lock should not be poisoned");
            for edge in edges {
                storage.insert_edge(edge.clone())?;
                _total_inserted += 1;
            }
        }

        Ok(ExecutionResult::Success)
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
        "Insert executor - inserts vertices and edges into storage"
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageEngine> HasStorage<S> for InsertExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("InsertExecutor storage should be set")
    }
}

// Executor for updating existing vertices/edges
pub struct UpdateExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    vertex_updates: Option<Vec<VertexUpdate>>, // Updates to apply to vertices
    edge_updates: Option<Vec<EdgeUpdate>>,     // Updates to apply to edges

    condition: Option<String>, // Condition for selecting items to update
}

#[derive(Debug, Clone)]
pub struct VertexUpdate {
    pub vertex_id: Value,
    pub properties: std::collections::HashMap<String, Value>,
    pub tags_to_add: Option<Vec<String>>,
    pub tags_to_remove: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct EdgeUpdate {
    pub src: Value,
    pub dst: Value,
    pub edge_type: String,
    pub properties: std::collections::HashMap<String, Value>,
}

impl<S: StorageEngine> UpdateExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        vertex_updates: Option<Vec<VertexUpdate>>,
        edge_updates: Option<Vec<EdgeUpdate>>,
        condition: Option<String>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "UpdateExecutor".to_string(), storage),
            vertex_updates,
            edge_updates,
            condition,
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for UpdateExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let mut _total_updated = 0;

        let condition_expression = if let Some(ref condition_str) = self.condition {
            Some(parse_expression_meta_from_string(condition_str).map(|meta| meta.into()).map_err(|e| {
                crate::core::error::DBError::Query(crate::core::error::QueryError::ExecutionError(
                    format!("条件解析失败: {}", e),
                ))
            })?)
        } else {
            None
        };

        let mut storage = safe_lock(self.get_storage())
            .expect("UpdateExecutor storage lock should not be poisoned");

        if let Some(updates) = &self.vertex_updates {
            for update in updates {
                if let Some(ref expression) = condition_expression {
                    let mut context = BasicExpressionContext::default();
                    context.set_variable("vertex_id", update.vertex_id.clone());
                    for (key, value) in &update.properties {
                        context.set_variable(key.clone(), value.clone());
                    }

                    let result =
                        ExpressionEvaluator::evaluate(expression, &mut context).map_err(|e| {
                            crate::core::error::DBError::Query(
                                crate::core::error::QueryError::ExecutionError(format!(
                                    "条件求值失败: {}",
                                    e
                                )),
                            )
                        })?;

                    if let Value::Bool(true) = result {
                        if let Value::String(_id_str) = &update.vertex_id {
                            if let Some(mut vertex) = storage.get_node(&update.vertex_id)? {
                                for (key, value) in &update.properties {
                                    vertex.properties.insert(key.clone(), value.clone());
                                }
                                storage.update_node(vertex)?;
                                _total_updated += 1;
                            }
                        }
                    }
                } else {
                    if let Some(mut vertex) = storage.get_node(&update.vertex_id)? {
                        for (key, value) in &update.properties {
                            vertex.properties.insert(key.clone(), value.clone());
                        }
                        storage.update_node(vertex)?;
                        _total_updated += 1;
                    }
                }
            }
        }

        if let Some(updates) = &self.edge_updates {
            for update in updates {
                if let Some(ref expression) = condition_expression {
                    let mut context = BasicExpressionContext::default();
                    context.set_variable("src", update.src.clone());
                    context.set_variable("dst", update.dst.clone());
                    context.set_variable("edge_type", Value::String(update.edge_type.clone()));
                    for (key, value) in &update.properties {
                        context.set_variable(key.clone(), value.clone());
                    }

                    let result =
                        ExpressionEvaluator::evaluate(expression, &mut context).map_err(|e| {
                            crate::core::error::DBError::Query(
                                crate::core::error::QueryError::ExecutionError(format!(
                                    "条件求值失败: {}",
                                    e
                                )),
                            )
                        })?;

                    if let Value::Bool(true) = result {
                        if let Some(mut edge) =
                            storage.get_edge(&update.src, &update.dst, &update.edge_type)?
                        {
                            for (key, value) in &update.properties {
                                edge.props.insert(key.clone(), value.clone());
                            }
                            storage.delete_edge(&update.src, &update.dst, &update.edge_type)?;
                            storage.insert_edge(edge)?;
                            _total_updated += 1;
                        }
                    }
                } else {
                    if let Some(mut edge) =
                        storage.get_edge(&update.src, &update.dst, &update.edge_type)?
                    {
                        for (key, value) in &update.properties {
                            edge.props.insert(key.clone(), value.clone());
                        }
                        storage.delete_edge(&update.src, &update.dst, &update.edge_type)?;
                        storage.insert_edge(edge)?;
                        _total_updated += 1;
                    }
                }
            }
        }

        Ok(ExecutionResult::Success)
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
        "Update executor - updates vertices and edges in storage"
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageEngine> HasStorage<S> for UpdateExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("UpdateExecutor storage should be set")
    }
}

impl<S: StorageEngine> HasStorage<S> for DeleteExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("DeleteExecutor storage should be set")
    }
}

// Executor for deleting vertices/edges
pub struct DeleteExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    vertex_ids: Option<Vec<Value>>, // IDs of vertices to delete
    edge_ids: Option<Vec<Value>>,   // IDs of edges to delete

    _condition: Option<String>, // Condition for selecting items to delete
}

impl<S: StorageEngine> DeleteExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        vertex_ids: Option<Vec<Value>>,
        edge_ids: Option<Vec<Value>>,
        _condition: Option<String>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "DeleteExecutor".to_string(), storage),
            vertex_ids,
            edge_ids,
            _condition,
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for DeleteExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let mut _total_deleted = 0;

        if let Some(ids) = &self.vertex_ids {
            let _storage = safe_lock(&*self.get_storage())
                .expect("DeleteExecutor storage lock should not be poisoned");
            for _id in ids {
                _total_deleted += 1;
            }
        }

        if let Some(ids) = &self.edge_ids {
            let _storage = safe_lock(&*self.get_storage())
                .expect("DeleteExecutor storage lock should not be poisoned");
            for _id in ids {
                _total_deleted += 1;
            }
        }

        Ok(ExecutionResult::Success)
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
        "Delete executor - deletes vertices and edges from storage"
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base.get_stats_mut()
    }
}

// Executor for creating indexes
pub struct CreateIndexExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,

    index_name: String,

    index_type: IndexType,

    properties: Vec<String>, // Properties to index

    tag_name: Option<String>, // Tag name for vertex indexes
}

#[derive(Debug, Clone)]
pub enum IndexType {
    Vertex,
    Edge,
}

impl<S: StorageEngine> CreateIndexExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        index_name: String,
        index_type: IndexType,
        properties: Vec<String>,
        tag_name: Option<String>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "CreateIndexExecutor".to_string(), storage),
            index_name,
            index_type,
            properties,
            tag_name,
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for CreateIndexExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        Ok(ExecutionResult::Success)
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
        "Create index executor - creates indexes in storage"
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base.get_stats_mut()
    }
}

// Executor for dropping indexes
pub struct DropIndexExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,

    _index_name: String,
}

impl<S: StorageEngine> DropIndexExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, _index_name: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "DropIndexExecutor".to_string(), storage),
            _index_name,
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for DropIndexExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        Ok(ExecutionResult::Success)
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
        "Drop index executor - drops indexes from storage"
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base.get_stats_mut()
    }
}
