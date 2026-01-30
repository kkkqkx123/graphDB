use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use super::base::BaseExecutor;
use crate::core::{Edge, StorageError, Value, Vertex, DBError};
use crate::core::types::IndexInfo;
use crate::expression::context::basic_context::BasicExpressionContext;
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::query::executor::traits::{DBResult, ExecutionResult, Executor, HasStorage};
use crate::query::parser::expressions::parse_expression_meta_from_string;
use crate::storage::StorageClient;
use crate::utils::safe_lock;

// Executor for inserting new vertices/edges
pub struct InsertExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    vertex_data: Option<Vec<Vertex>>, // Data to be inserted
    edge_data: Option<Vec<Edge>>,
}

impl<S: StorageClient> InsertExecutor<S> {
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
impl<S: StorageClient + Send + 'static> Executor<S> for InsertExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let mut _total_inserted = 0;

        // Insert vertices if provided
        if let Some(vertices) = &self.vertex_data {
            let mut storage = safe_lock(self.get_storage())
                .expect("InsertExecutor storage lock should not be poisoned");
            for vertex in vertices {
                storage.insert_vertex("default", vertex.clone())?;
                _total_inserted += 1;
            }
        }

        // Insert edges if provided
        if let Some(edges) = &self.edge_data {
            let mut storage = safe_lock(self.get_storage())
                .expect("InsertExecutor storage lock should not be poisoned");
            for edge in edges {
                storage.insert_edge("default", edge.clone())?;
                _total_inserted += 1;
            }
        }

        Ok(ExecutionResult::Success)
    }

    fn open(&mut self) -> Result<(), DBError> {
        Ok(())
    }

    fn close(&mut self) -> Result<(), DBError> {
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

impl<S: StorageClient> HasStorage<S> for CreateIndexExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("CreateIndexExecutor storage should be set")
    }
}

impl<S: StorageClient> HasStorage<S> for DropIndexExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("DropIndexExecutor storage should be set")
    }
}

impl<S: StorageClient> HasStorage<S> for InsertExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("InsertExecutor storage should be set")
    }
}

// Executor for updating existing vertices/edges
pub struct UpdateExecutor<S: StorageClient> {
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

impl<S: StorageClient> UpdateExecutor<S> {
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
impl<S: StorageClient + Send + 'static> Executor<S> for UpdateExecutor<S> {
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
                            if let Some(mut vertex) = storage.get_vertex("default", &update.vertex_id)? {
                                for (key, value) in &update.properties {
                                    vertex.properties.insert(key.clone(), value.clone());
                                }
                                storage.update_vertex("default", vertex)?;
                                _total_updated += 1;
                            }
                        }
                    }
                } else {
                    if let Some(mut vertex) = storage.get_vertex("default", &update.vertex_id)? {
                        for (key, value) in &update.properties {
                            vertex.properties.insert(key.clone(), value.clone());
                        }
                        storage.update_vertex("default", vertex)?;
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
                            storage.get_edge("default", &update.src, &update.dst, &update.edge_type)?
                        {
                            for (key, value) in &update.properties {
                                edge.props.insert(key.clone(), value.clone());
                            }
                            storage.delete_edge("default", &update.src, &update.dst, &update.edge_type)?;
                            storage.insert_edge("default", edge)?;
                            _total_updated += 1;
                        }
                    }
                } else {
                    if let Some(mut edge) =
                        storage.get_edge("default", &update.src, &update.dst, &update.edge_type)?
                    {
                        for (key, value) in &update.properties {
                            edge.props.insert(key.clone(), value.clone());
                        }
                        storage.delete_edge("default", &update.src, &update.dst, &update.edge_type)?;
                        storage.insert_edge("default", edge)?;
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

impl<S: StorageClient> HasStorage<S> for UpdateExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("UpdateExecutor storage should be set")
    }
}

impl<S: StorageClient> HasStorage<S> for DeleteExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("DeleteExecutor storage should be set")
    }
}

// Executor for deleting vertices/edges
pub struct DeleteExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    vertex_ids: Option<Vec<Value>>, // IDs of vertices to delete
    edge_ids: Option<Vec<Value>>,   // IDs of edges to delete

    _condition: Option<String>, // Condition for selecting items to delete
}

impl<S: StorageClient> DeleteExecutor<S> {
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
impl<S: StorageClient + Send + 'static> Executor<S> for DeleteExecutor<S> {
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
pub struct CreateIndexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,

    index_name: String,

    index_type: crate::index::IndexType,

    properties: Vec<String>, // Properties to index

    tag_name: Option<String>, // Tag name for vertex indexes
}

impl<S: StorageClient> CreateIndexExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        index_name: String,
        index_type: crate::index::IndexType,
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
impl<S: StorageClient + Send + 'static> Executor<S> for CreateIndexExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, DBError> {
        let mut storage = safe_lock(self.get_storage())
            .expect("CreateIndexExecutor storage lock should not be poisoned");

        let target_type = match self.index_type {
            crate::index::IndexType::TagIndex => "tag",
            crate::index::IndexType::EdgeIndex => "edge",
            crate::index::IndexType::FulltextIndex => "fulltext",
        };

        let target_name = self.tag_name.clone()
            .or_else(|| Some(self.index_name.clone()))
            .unwrap_or_default();

        let index_info = IndexInfo {
            name: self.index_name.clone(),
            space_name: String::new(),
            target_type: target_type.to_string(),
            target_name,
            properties: self.properties.clone(),
            comment: None,
        };

        let result = match self.index_type {
            crate::index::IndexType::TagIndex => {
                storage.create_tag_index("default", &index_info)
            }
            crate::index::IndexType::EdgeIndex => {
                storage.create_edge_index("default", &index_info)
            }
            crate::index::IndexType::FulltextIndex => {
                storage.create_tag_index("default", &index_info)
            }
        };

        result
            .map(|_| ExecutionResult::Success)
            .map_err(|e| DBError::from(e))
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
pub struct DropIndexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,

    _index_name: String,
}

impl<S: StorageClient> DropIndexExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, _index_name: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "DropIndexExecutor".to_string(), storage),
            _index_name,
        }
    }
}

#[async_trait]
impl<S: StorageClient + Send + 'static> Executor<S> for DropIndexExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, DBError> {
        let mut storage = safe_lock(self.get_storage())
            .expect("DropIndexExecutor storage lock should not be poisoned");

        let result = storage.drop_tag_index("", &self._index_name)?;

        if result {
            Ok(ExecutionResult::Success)
        } else {
            Err(DBError::from(StorageError::DbError(
                format!("Index '{}' not found", self._index_name)
            )))
        }
    }

    fn open(&mut self) -> Result<(), DBError> {
        Ok(())
    }

    fn close(&mut self) -> Result<(), DBError> {
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
