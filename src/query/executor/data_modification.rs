use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use super::base::storage_processor_executor::{
    StorageProcessorExecutor, StorageProcessorExecutorImpl,
};
use crate::core::{Edge, Value, Vertex, DBError};
use crate::index::Index;
use crate::expression::context::basic_context::BasicExpressionContext;
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::query::context::runtime_context::RuntimeContext;
use crate::query::executor::traits::{DBResult, ExecutionResult, Executor, HasStorage};
use crate::query::parser::expressions::parse_expression_meta_from_string;
use crate::storage::StorageClient;
use crate::utils::safe_lock;

// Executor for inserting new vertices/edges
pub struct InsertExecutor<S: StorageClient> {
    processor: StorageProcessorExecutor<S, usize>,
    vertex_data: Option<Vec<Vertex>>,
    edge_data: Option<Vec<Edge>>,
}

impl<S: StorageClient> InsertExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        vertex_data: Option<Vec<Vertex>>,
        edge_data: Option<Vec<Edge>>,
    ) -> Self {
        let context = RuntimeContext::new_simple();
        Self {
            processor: StorageProcessorExecutor::new(
                id,
                "InsertExecutor".to_string(),
                storage,
                context,
            ),
            vertex_data,
            edge_data,
        }
    }

    pub fn with_vertices(id: i64, storage: Arc<Mutex<S>>, vertex_data: Vec<Vertex>) -> Self {
        let context = RuntimeContext::new_simple();
        Self {
            processor: StorageProcessorExecutor::new(
                id,
                "InsertExecutor".to_string(),
                storage,
                context,
            ),
            vertex_data: Some(vertex_data),
            edge_data: None,
        }
    }

    pub fn with_edges(id: i64, storage: Arc<Mutex<S>>, edge_data: Vec<Edge>) -> Self {
        let context = RuntimeContext::new_simple();
        Self {
            processor: StorageProcessorExecutor::new(
                id,
                "InsertExecutor".to_string(),
                storage,
                context,
            ),
            vertex_data: None,
            edge_data: Some(edge_data),
        }
    }
}

#[async_trait]
impl<S: StorageClient + Send + Sync + 'static> Executor<S> for InsertExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        <InsertExecutor<S> as StorageProcessorExecutorImpl<S, usize>>::execute(self).await
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
        RuntimeContext::arc_plan_id(self.processor.processor().context())
    }

    fn name(&self) -> &str {
        "InsertExecutor"
    }

    fn description(&self) -> &str {
        "Insert executor - inserts vertices and edges into storage"
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.processor.stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.processor.stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for InsertExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.processor.get_storage()
    }
}

#[async_trait]
impl<S: StorageClient + Send + Sync + 'static> StorageProcessorExecutorImpl<S, usize> for InsertExecutor<S> {
    fn get_executor(&mut self) -> &mut StorageProcessorExecutor<S, usize> {
        &mut self.processor
    }

    async fn do_execute(&mut self) -> DBResult<usize> {
        let mut total_inserted = 0;

        if let Some(vertices) = &self.vertex_data {
            let mut storage = safe_lock(self.get_storage())
                .expect("InsertExecutor storage lock should not be poisoned");
            for vertex in vertices {
                storage.insert_vertex("default", vertex.clone())?;
                total_inserted += 1;
            }
        }

        if let Some(edges) = &self.edge_data {
            let mut storage = safe_lock(self.get_storage())
                .expect("InsertExecutor storage lock should not be poisoned");
            for edge in edges {
                storage.insert_edge("default", edge.clone())?;
                total_inserted += 1;
            }
        }

        Ok(total_inserted)
    }
}

// Executor for updating existing vertices/edges
pub struct UpdateExecutor<S: StorageClient> {
    processor: StorageProcessorExecutor<S, usize>,
    vertex_updates: Option<Vec<VertexUpdate>>,
    edge_updates: Option<Vec<EdgeUpdate>>,
    condition: Option<String>,
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
        let context = RuntimeContext::new_simple();
        Self {
            processor: StorageProcessorExecutor::new(
                id,
                "UpdateExecutor".to_string(),
                storage,
                context,
            ),
            vertex_updates,
            edge_updates,
            condition,
        }
    }
}

#[async_trait]
impl<S: StorageClient + Send + Sync + 'static> Executor<S> for UpdateExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        <UpdateExecutor<S> as StorageProcessorExecutorImpl<S, usize>>::execute(self).await
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
        RuntimeContext::arc_plan_id(self.processor.processor().context())
    }

    fn name(&self) -> &str {
        "UpdateExecutor"
    }

    fn description(&self) -> &str {
        "Update executor - updates vertices and edges in storage"
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.processor.stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.processor.stats_mut()
    }
}

#[async_trait]
impl<S: StorageClient + Send + Sync + 'static> StorageProcessorExecutorImpl<S, usize> for UpdateExecutor<S> {
    fn get_executor(&mut self) -> &mut StorageProcessorExecutor<S, usize> {
        &mut self.processor
    }

    async fn do_execute(&mut self) -> DBResult<usize> {
        let mut total_updated = 0;

        let condition_expression = if let Some(ref condition_str) = self.condition {
            Some(parse_expression_meta_from_string(condition_str).map(|meta| meta.into()).map_err(|e| {
                crate::core::error::DBError::Query(crate::core::error::QueryError::ExecutionError(
                    format!("条件解析失败: {}", e),
                ))
            })?)
        } else {
            None
        };

        let mut storage: std::sync::MutexGuard<'_, S> = safe_lock(self.get_storage())
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
                                total_updated += 1;
                            }
                        }
                    }
                } else {
                    if let Some(mut vertex) = storage.get_vertex("default", &update.vertex_id)? {
                        for (key, value) in &update.properties {
                            vertex.properties.insert(key.clone(), value.clone());
                        }
                        storage.update_vertex("default", vertex)?;
                        total_updated += 1;
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
                            total_updated += 1;
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
                        total_updated += 1;
                    }
                }
            }
        }

        Ok(total_updated)
    }
}

impl<S: StorageClient> HasStorage<S> for UpdateExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.processor.get_storage()
    }
}

// Executor for deleting vertices/edges
pub struct DeleteExecutor<S: StorageClient> {
    processor: StorageProcessorExecutor<S, usize>,
    vertex_ids: Option<Vec<Value>>,
    edge_ids: Option<Vec<(Value, Value, String)>>,
    condition: Option<String>,
}

impl<S: StorageClient> DeleteExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        vertex_ids: Option<Vec<Value>>,
        edge_ids: Option<Vec<(Value, Value, String)>>,
        condition: Option<String>,
    ) -> Self {
        let context = RuntimeContext::new_simple();
        Self {
            processor: StorageProcessorExecutor::new(
                id,
                "DeleteExecutor".to_string(),
                storage,
                context,
            ),
            vertex_ids,
            edge_ids,
            condition,
        }
    }
}

#[async_trait]
impl<S: StorageClient + Send + Sync + 'static> Executor<S> for DeleteExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        <DeleteExecutor<S> as StorageProcessorExecutorImpl<S, usize>>::execute(self).await
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
        RuntimeContext::arc_plan_id(self.processor.processor().context())
    }

    fn name(&self) -> &str {
        "DeleteExecutor"
    }

    fn description(&self) -> &str {
        "Delete executor - deletes vertices and edges from storage"
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.processor.stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.processor.stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for DeleteExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.processor.get_storage()
    }
}

#[async_trait]
impl<S: StorageClient + Send + Sync + 'static> StorageProcessorExecutorImpl<S, usize> for DeleteExecutor<S> {
    fn get_executor(&mut self) -> &mut StorageProcessorExecutor<S, usize> {
        &mut self.processor
    }

    async fn do_execute(&mut self) -> DBResult<usize> {
        let mut total_deleted = 0;

        if let Some(ids) = &self.vertex_ids {
            let mut storage = safe_lock(self.get_storage())
                .expect("DeleteExecutor storage lock should not be poisoned");
            for id in ids {
                if storage.delete_vertex("default", id).is_ok() {
                    total_deleted += 1;
                }
            }
        }

        if let Some(edges) = &self.edge_ids {
            let mut storage = safe_lock(self.get_storage())
                .expect("DeleteExecutor storage lock should not be poisoned");
            for (src, dst, edge_type) in edges {
                if storage.delete_edge("default", src, dst, edge_type).is_ok() {
                    total_deleted += 1;
                }
            }
        }

        Ok(total_deleted)
    }
}

// Executor for creating indexes
pub struct CreateIndexExecutor<S: StorageClient> {
    processor: StorageProcessorExecutor<S, ()>,

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
        let context = RuntimeContext::new_simple();
        Self {
            processor: StorageProcessorExecutor::new(
                id,
                "CreateIndexExecutor".to_string(),
                storage,
                context,
            ),
            index_name,
            index_type,
            properties,
            tag_name,
        }
    }
}

impl<S: StorageClient> HasStorage<S> for CreateIndexExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.processor.get_storage()
    }
}

#[async_trait]
impl<S: StorageClient + Send + Sync + 'static> Executor<S> for CreateIndexExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        <CreateIndexExecutor<S> as StorageProcessorExecutorImpl<S, ()>>::execute(self).await
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
        RuntimeContext::arc_plan_id(self.processor.processor().context())
    }

    fn name(&self) -> &str {
        "CreateIndexExecutor"
    }

    fn description(&self) -> &str {
        "Create index executor - creates indexes in storage"
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.processor.stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.processor.stats_mut()
    }
}

#[async_trait]
impl<S: StorageClient + Send + Sync + 'static> StorageProcessorExecutorImpl<S, ()> for CreateIndexExecutor<S> {
    fn get_executor(&mut self) -> &mut StorageProcessorExecutor<S, ()> {
        &mut self.processor
    }

    async fn do_execute(&mut self) -> DBResult<()> {
        let mut storage = safe_lock(self.get_storage())
            .expect("CreateIndexExecutor storage lock should not be poisoned");

        let target_name = self.tag_name.clone()
            .or_else(|| Some(self.index_name.clone()))
            .unwrap_or_default();

        let index_type = self.index_type.clone();
        let index = Index::new(
            0,
            self.index_name.clone(),
            0,
            target_name,
            Vec::new(),
            self.properties.clone(),
            index_type.clone(),
            false,
        );

        match index_type {
            crate::index::IndexType::TagIndex => {
                storage.create_tag_index("default", &index)?;
            }
            crate::index::IndexType::EdgeIndex => {
                storage.create_edge_index("default", &index)?;
            }
            crate::index::IndexType::FulltextIndex => {
                storage.create_tag_index("default", &index)?;
            }
        }

        Ok(())
    }
}

// Executor for dropping indexes
pub struct DropIndexExecutor<S: StorageClient> {
    processor: StorageProcessorExecutor<S, ()>,

    _index_name: String,
}

impl<S: StorageClient> DropIndexExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, _index_name: String) -> Self {
        let context = RuntimeContext::new_simple();
        Self {
            processor: StorageProcessorExecutor::new(
                id,
                "DropIndexExecutor".to_string(),
                storage,
                context,
            ),
            _index_name,
        }
    }
}

impl<S: StorageClient> HasStorage<S> for DropIndexExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.processor.get_storage()
    }
}

#[async_trait]
impl<S: StorageClient + Send + Sync + 'static> Executor<S> for DropIndexExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        <DropIndexExecutor<S> as StorageProcessorExecutorImpl<S, ()>>::execute(self).await
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
        RuntimeContext::arc_plan_id(self.processor.processor().context())
    }

    fn name(&self) -> &str {
        "DropIndexExecutor"
    }

    fn description(&self) -> &str {
        "Drop index executor - drops indexes from storage"
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.processor.stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.processor.stats_mut()
    }
}

#[async_trait]
impl<S: StorageClient + Send + Sync + 'static> StorageProcessorExecutorImpl<S, ()> for DropIndexExecutor<S> {
    fn get_executor(&mut self) -> &mut StorageProcessorExecutor<S, ()> {
        &mut self.processor
    }

    async fn do_execute(&mut self) -> DBResult<()> {
        let mut storage = safe_lock(self.get_storage())
            .expect("DropIndexExecutor storage lock should not be poisoned");

        storage.drop_tag_index("default", &self._index_name)?;

        Ok(())
    }
}
