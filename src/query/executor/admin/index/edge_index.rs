//! 边索引执行器
//!
//! 提供边索引的创建、删除、描述和列出功能。

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::core::{DataSet, Value};
use crate::storage::iterator::Row;
use crate::core::types::metadata::IndexInfo;
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;

/// 边索引描述信息
#[derive(Debug, Clone)]
pub struct EdgeIndexDesc {
    pub index_id: i32,
    pub index_name: String,
    pub edge_name: String,
    pub fields: Vec<String>,
    pub comment: Option<String>,
}

impl EdgeIndexDesc {
    pub fn from_metadata(info: &IndexInfo) -> Self {
        Self {
            index_id: 0,
            index_name: info.name.clone(),
            edge_name: info.target_name.clone(),
            fields: info.properties.clone(),
            comment: info.comment.clone(),
        }
    }
}

impl From<&EdgeIndexDesc> for IndexInfo {
    fn from(desc: &EdgeIndexDesc) -> Self {
        IndexInfo {
            space_name: String::new(),
            name: desc.index_name.clone(),
            target_type: "edge".to_string(),
            target_name: desc.edge_name.clone(),
            properties: desc.fields.clone(),
            comment: desc.comment.clone(),
        }
    }
}

/// 创建边索引执行器
#[derive(Debug)]
pub struct CreateEdgeIndexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    index_info: IndexInfo,
    if_not_exists: bool,
}

impl<S: StorageClient> CreateEdgeIndexExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, index_info: IndexInfo) -> Self {
        Self {
            base: BaseExecutor::new(id, "CreateEdgeIndexExecutor".to_string(), storage),
            index_info,
            if_not_exists: false,
        }
    }

    pub fn with_if_not_exists(id: i64, storage: Arc<Mutex<S>>, index_info: IndexInfo) -> Self {
        Self {
            base: BaseExecutor::new(id, "CreateEdgeIndexExecutor".to_string(), storage),
            index_info,
            if_not_exists: true,
        }
    }
}

#[async_trait]
impl<S: StorageClient + Send + Sync + 'static> Executor<S> for CreateEdgeIndexExecutor<S> {
    async fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let mut storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::Storage(
                crate::core::error::StorageError::DbError(format!("Storage lock poisoned: {}", e))
            )
        })?;

        let result = storage_guard.create_edge_index(&self.index_info);

        match result {
            Ok(true) => Ok(ExecutionResult::Success),
            Ok(false) => {
                if self.if_not_exists {
                    Ok(ExecutionResult::Success)
                } else {
                    Ok(ExecutionResult::Error(format!("Index '{}' already exists", self.index_info.name)))
                }
            }
            Err(e) => Ok(ExecutionResult::Error(format!("Failed to create edge index: {}", e))),
        }
    }

    fn open(&mut self) -> crate::query::executor::base::DBResult<()> { self.base.open() }
    fn close(&mut self) -> crate::query::executor::base::DBResult<()> { self.base.close() }
    fn is_open(&self) -> bool { self.base.is_open() }
    fn id(&self) -> i64 { self.base.id }
    fn name(&self) -> &str { "CreateEdgeIndexExecutor" }
    fn description(&self) -> &str { "Creates an edge index" }
    fn stats(&self) -> &crate::query::executor::base::ExecutorStats { self.base.get_stats() }
    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats { self.base.get_stats_mut() }
}

impl<S: StorageClient> crate::query::executor::base::HasStorage<S> for ShowEdgeIndexesExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient> crate::query::executor::base::HasStorage<S> for CreateEdgeIndexExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

/// 删除边索引执行器
#[derive(Debug)]
pub struct DropEdgeIndexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    space_name: String,
    index_name: String,
    if_exists: bool,
}

impl<S: StorageClient> DropEdgeIndexExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, space_name: String, index_name: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "DropEdgeIndexExecutor".to_string(), storage),
            space_name,
            index_name,
            if_exists: false,
        }
    }

    pub fn with_if_exists(id: i64, storage: Arc<Mutex<S>>, space_name: String, index_name: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "DropEdgeIndexExecutor".to_string(), storage),
            space_name,
            index_name,
            if_exists: true,
        }
    }
}

#[async_trait]
impl<S: StorageClient + Send + Sync + 'static> Executor<S> for DropEdgeIndexExecutor<S> {
    async fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let mut storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::Storage(
                crate::core::error::StorageError::DbError(format!("Storage lock poisoned: {}", e))
            )
        })?;

        let result = storage_guard.drop_edge_index(&self.space_name, &self.index_name);

        match result {
            Ok(true) => Ok(ExecutionResult::Success),
            Ok(false) => {
                if self.if_exists {
                    Ok(ExecutionResult::Success)
                } else {
                    Ok(ExecutionResult::Error(format!("Index '{}' not found", self.index_name)))
                }
            }
            Err(e) => Ok(ExecutionResult::Error(format!("Failed to drop edge index: {}", e))),
        }
    }

    fn open(&mut self) -> crate::query::executor::base::DBResult<()> { self.base.open() }
    fn close(&mut self) -> crate::query::executor::base::DBResult<()> { self.base.close() }
    fn is_open(&self) -> bool { self.base.is_open() }
    fn id(&self) -> i64 { self.base.id }
    fn name(&self) -> &str { "DropEdgeIndexExecutor" }
    fn description(&self) -> &str { "Drops an edge index" }
    fn stats(&self) -> &crate::query::executor::base::ExecutorStats { self.base.get_stats() }
    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats { self.base.get_stats_mut() }
}

impl<S: StorageClient> crate::query::executor::base::HasStorage<S> for DropEdgeIndexExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

/// 描述边索引执行器
#[derive(Debug)]
pub struct DescEdgeIndexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    space_name: String,
    index_name: String,
}

impl<S: StorageClient> DescEdgeIndexExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, space_name: String, index_name: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "DescEdgeIndexExecutor".to_string(), storage),
            space_name,
            index_name,
        }
    }
}

#[async_trait]
impl<S: StorageClient + Send + Sync + 'static> Executor<S> for DescEdgeIndexExecutor<S> {
    async fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::Storage(
                crate::core::error::StorageError::DbError(format!("Storage lock poisoned: {}", e))
            )
        })?;

        let result = storage_guard.get_edge_index(&self.space_name, &self.index_name);

        match result {
            Ok(Some(desc)) => {
                let desc = EdgeIndexDesc::from_metadata(&desc);
                let rows = vec![
                    vec![
                        Value::String(desc.index_name),
                        Value::String(desc.edge_name),
                        Value::String(desc.fields.join(", ")),
                        Value::String(desc.comment.unwrap_or_else(|| "".to_string())),
                    ]
                ];

                let dataset = DataSet {
                    col_names: vec!["Index Name".to_string(), "Edge Name".to_string(), "Fields".to_string(), "Comment".to_string()],
                    rows,
                };
                Ok(ExecutionResult::DataSet(dataset))
            }
            Ok(None) => Ok(ExecutionResult::Error(format!("Index '{}' not found", self.index_name))),
            Err(e) => Ok(ExecutionResult::Error(format!("Failed to describe edge index: {}", e))),
        }
    }

    fn open(&mut self) -> crate::query::executor::base::DBResult<()> { self.base.open() }
    fn close(&mut self) -> crate::query::executor::base::DBResult<()> { self.base.close() }
    fn is_open(&self) -> bool { self.base.is_open() }
    fn id(&self) -> i64 { self.base.id }
    fn name(&self) -> &str { "DescEdgeIndexExecutor" }
    fn description(&self) -> &str { "Describes an edge index" }
    fn stats(&self) -> &crate::query::executor::base::ExecutorStats { self.base.get_stats() }
    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats { self.base.get_stats_mut() }
}

impl<S: StorageClient> crate::query::executor::base::HasStorage<S> for DescEdgeIndexExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

/// 列出边索引执行器
#[derive(Debug)]
pub struct ShowEdgeIndexesExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    space_name: String,
}

impl<S: StorageClient> ShowEdgeIndexesExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, space_name: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "ShowEdgeIndexesExecutor".to_string(), storage),
            space_name,
        }
    }
}

#[async_trait]
impl<S: StorageClient + Send + Sync + 'static> Executor<S> for ShowEdgeIndexesExecutor<S> {
    async fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::Storage(
                crate::core::error::StorageError::DbError(format!("Storage lock poisoned: {}", e))
            )
        })?;

        let result = storage_guard.list_edge_indexes(&self.space_name);

        match result {
            Ok(indexes) => {
                let rows: Vec<Row> = indexes
                    .iter()
                    .map(|desc| {
                        let desc = EdgeIndexDesc::from_metadata(desc);
                        vec![
                            Value::String(desc.index_name.clone()),
                            Value::String(desc.edge_name.clone()),
                            Value::String(desc.fields.join(", ")),
                        ]
                    })
                    .collect();

                let dataset = DataSet {
                    col_names: vec!["Index Name".to_string(), "Edge Name".to_string(), "Fields".to_string()],
                    rows,
                };
                Ok(ExecutionResult::DataSet(dataset))
            }
            Err(e) => Ok(ExecutionResult::Error(format!("Failed to show edge indexes: {}", e))),
        }
    }

    fn open(&mut self) -> crate::query::executor::base::DBResult<()> { self.base.open() }
    fn close(&mut self) -> crate::query::executor::base::DBResult<()> { self.base.close() }
    fn is_open(&self) -> bool { self.base.is_open() }
    fn id(&self) -> i64 { self.base.id }
    fn name(&self) -> &str { "ShowEdgeIndexesExecutor" }
    fn description(&self) -> &str { "Shows all edge indexes" }
    fn stats(&self) -> &crate::query::executor::base::ExecutorStats { self.base.get_stats() }
    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats { self.base.get_stats_mut() }
}
