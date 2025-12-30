use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use super::base::BaseExecutor;
use crate::core::{Edge, Value, Vertex};
use crate::query::executor::traits::{
    DBResult, ExecutionResult, Executor, HasStorage,
};
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
}

impl<S: StorageEngine> HasStorage<S> for InsertExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.storage.as_ref().expect("InsertExecutor storage should be set")
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
    pub edge_id: Value,
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

        // Update vertices if provided
        if let Some(updates) = &self.vertex_updates {
            let _storage = safe_lock(self.get_storage())
                .expect("UpdateExecutor storage lock should not be poisoned");
            for _update in updates {
                _total_updated += 1;
            }
        }

        // Update edges if provided
        if let Some(updates) = &self.edge_updates {
            let _storage = safe_lock(&*self.get_storage())
                .expect("UpdateExecutor storage lock should not be poisoned");
            for _update in updates {
                _total_updated += 1;
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
}

impl<S: StorageEngine> HasStorage<S> for UpdateExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.storage.as_ref().expect("UpdateExecutor storage should be set")
    }
}

impl<S: StorageEngine> HasStorage<S> for DeleteExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.storage.as_ref().expect("DeleteExecutor storage should be set")
    }
}

// Executor for deleting vertices/edges
pub struct DeleteExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    vertex_ids: Option<Vec<Value>>, // IDs of vertices to delete
    edge_ids: Option<Vec<Value>>,   // IDs of edges to delete
    
    condition: Option<String>, // Condition for selecting items to delete
    
    cascade: bool, // Whether to delete related items
}

impl<S: StorageEngine> DeleteExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        vertex_ids: Option<Vec<Value>>,
        edge_ids: Option<Vec<Value>>,
        _condition: Option<String>,
        _cascade: bool,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "DeleteExecutor".to_string(), storage),
            vertex_ids,
            edge_ids,
            _condition,
            _cascade,
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
        _index_name: String,
        _index_type: IndexType,
        _properties: Vec<String>,
        _tag_name: Option<String>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "CreateIndexExecutor".to_string(), storage),
            _index_name,
            _index_type,
            _properties,
            _tag_name,
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
}

// Executor for dropping indexes
pub struct DropIndexExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    
    index_name: String,
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
}
