//! 标签索引执行器
//!
//! 提供标签索引的创建、删除、描述和列出功能。

use std::sync::{Arc, Mutex};

use crate::core::{DataSet, Value};
use crate::index::{Index, IndexType};
use crate::storage::iterator::Row;
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;

/// 标签索引描述信息
#[derive(Debug, Clone)]
pub struct TagIndexDesc {
    pub index_id: i32,
    pub index_name: String,
    pub tag_name: String,
    pub fields: Vec<String>,
    pub comment: Option<String>,
}

impl TagIndexDesc {
    pub fn from_metadata(info: &Index) -> Self {
        Self {
            index_id: info.id,
            index_name: info.name.clone(),
            tag_name: info.schema_name.clone(),
            fields: info.properties.clone(),
            comment: info.comment.clone(),
        }
    }
}

impl From<&TagIndexDesc> for Index {
    fn from(desc: &TagIndexDesc) -> Self {
        Index::new(
            0,
            desc.index_name.clone(),
            0,
            desc.tag_name.clone(),
            Vec::new(),
            desc.fields.clone(),
            IndexType::TagIndex,
            false,
        )
    }
}

/// 创建标签索引执行器
#[derive(Debug)]
pub struct CreateTagIndexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    index_info: Index,
    if_not_exists: bool,
}

impl<S: StorageClient> CreateTagIndexExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, index_info: Index) -> Self {
        Self {
            base: BaseExecutor::new(id, "CreateTagIndexExecutor".to_string(), storage),
            index_info,
            if_not_exists: false,
        }
    }

    pub fn with_if_not_exists(id: i64, storage: Arc<Mutex<S>>, index_info: Index) -> Self {
        Self {
            base: BaseExecutor::new(id, "CreateTagIndexExecutor".to_string(), storage),
            index_info,
            if_not_exists: true,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for CreateTagIndexExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let mut storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::Storage(
                crate::core::error::StorageError::DbError(format!("Storage lock poisoned: {}", e))
            )
        })?;

        let result = storage_guard.create_tag_index("default", &self.index_info);

        match result {
            Ok(true) => Ok(ExecutionResult::Success),
            Ok(false) => {
                if self.if_not_exists {
                    Ok(ExecutionResult::Success)
                } else {
                    Ok(ExecutionResult::Error(format!("Index '{}' already exists", self.index_info.name)))
                }
            }
            Err(e) => Ok(ExecutionResult::Error(format!("Failed to create tag index: {}", e))),
        }
    }

    fn open(&mut self) -> crate::query::executor::base::DBResult<()> { self.base.open() }
    fn close(&mut self) -> crate::query::executor::base::DBResult<()> { self.base.close() }
    fn is_open(&self) -> bool { self.base.is_open() }
    fn id(&self) -> i64 { self.base.id }
    fn name(&self) -> &str { "CreateTagIndexExecutor" }
    fn description(&self) -> &str { "Creates a tag index" }
    fn stats(&self) -> &crate::query::executor::base::ExecutorStats { self.base.get_stats() }
    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats { self.base.get_stats_mut() }
}

impl<S: StorageClient> crate::query::executor::base::HasStorage<S> for ShowTagIndexesExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient> crate::query::executor::base::HasStorage<S> for CreateTagIndexExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

/// 删除标签索引执行器
#[derive(Debug)]
pub struct DropTagIndexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    space_name: String,
    index_name: String,
    if_exists: bool,
}

impl<S: StorageClient> DropTagIndexExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, space_name: String, index_name: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "DropTagIndexExecutor".to_string(), storage),
            space_name,
            index_name,
            if_exists: false,
        }
    }

    pub fn with_if_exists(id: i64, storage: Arc<Mutex<S>>, space_name: String, index_name: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "DropTagIndexExecutor".to_string(), storage),
            space_name,
            index_name,
            if_exists: true,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for DropTagIndexExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let mut storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::Storage(
                crate::core::error::StorageError::DbError(format!("Storage lock poisoned: {}", e))
            )
        })?;

        let result = storage_guard.drop_tag_index(&self.space_name, &self.index_name);

        match result {
            Ok(true) => Ok(ExecutionResult::Success),
            Ok(false) => {
                if self.if_exists {
                    Ok(ExecutionResult::Success)
                } else {
                    Ok(ExecutionResult::Error(format!("Index '{}' not found", self.index_name)))
                }
            }
            Err(e) => Ok(ExecutionResult::Error(format!("Failed to drop tag index: {}", e))),
        }
    }

    fn open(&mut self) -> crate::query::executor::base::DBResult<()> { self.base.open() }
    fn close(&mut self) -> crate::query::executor::base::DBResult<()> { self.base.close() }
    fn is_open(&self) -> bool { self.base.is_open() }
    fn id(&self) -> i64 { self.base.id }
    fn name(&self) -> &str { "DropTagIndexExecutor" }
    fn description(&self) -> &str { "Drops a tag index" }
    fn stats(&self) -> &crate::query::executor::base::ExecutorStats { self.base.get_stats() }
    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats { self.base.get_stats_mut() }
}

impl<S: StorageClient> crate::query::executor::base::HasStorage<S> for DropTagIndexExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

/// 描述标签索引执行器
#[derive(Debug)]
pub struct DescTagIndexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    space_name: String,
    index_name: String,
}

impl<S: StorageClient> DescTagIndexExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, space_name: String, index_name: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "DescTagIndexExecutor".to_string(), storage),
            space_name,
            index_name,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for DescTagIndexExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::Storage(
                crate::core::error::StorageError::DbError(format!("Storage lock poisoned: {}", e))
            )
        })?;

        let result = storage_guard.get_tag_index(&self.space_name, &self.index_name);

        match result {
            Ok(Some(desc)) => {
                let desc = TagIndexDesc::from_metadata(&desc);
                let rows = vec![
                    vec![
                        Value::String(desc.index_name),
                        Value::String(desc.tag_name),
                        Value::String(desc.fields.join(", ")),
                        Value::String(desc.comment.unwrap_or_else(|| "".to_string())),
                    ]
                ];

                let dataset = DataSet {
                    col_names: vec!["Index Name".to_string(), "Tag Name".to_string(), "Fields".to_string(), "Comment".to_string()],
                    rows,
                };
                Ok(ExecutionResult::DataSet(dataset))
            }
            Ok(None) => Ok(ExecutionResult::Error(format!("Index '{}' not found", self.index_name))),
            Err(e) => Ok(ExecutionResult::Error(format!("Failed to describe tag index: {}", e))),
        }
    }

    fn open(&mut self) -> crate::query::executor::base::DBResult<()> { self.base.open() }
    fn close(&mut self) -> crate::query::executor::base::DBResult<()> { self.base.close() }
    fn is_open(&self) -> bool { self.base.is_open() }
    fn id(&self) -> i64 { self.base.id }
    fn name(&self) -> &str { "DescTagIndexExecutor" }
    fn description(&self) -> &str { "Describes a tag index" }
    fn stats(&self) -> &crate::query::executor::base::ExecutorStats { self.base.get_stats() }
    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats { self.base.get_stats_mut() }
}

impl<S: StorageClient> crate::query::executor::base::HasStorage<S> for DescTagIndexExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

/// 列出标签索引执行器
#[derive(Debug)]
pub struct ShowTagIndexesExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    space_name: String,
}

impl<S: StorageClient> ShowTagIndexesExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, space_name: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "ShowTagIndexesExecutor".to_string(), storage),
            space_name,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for ShowTagIndexesExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::Storage(
                crate::core::error::StorageError::DbError(format!("Storage lock poisoned: {}", e))
            )
        })?;

        let result = storage_guard.list_tag_indexes(&self.space_name);

        match result {
            Ok(indexes) => {
                let rows: Vec<Row> = indexes
                    .iter()
                    .map(|desc| {
                        let desc = TagIndexDesc::from_metadata(desc);
                        vec![
                            Value::String(desc.index_name.clone()),
                            Value::String(desc.tag_name.clone()),
                            Value::String(desc.fields.join(", ")),
                        ]
                    })
                    .collect();

                let dataset = DataSet {
                    col_names: vec!["Index Name".to_string(), "Tag Name".to_string(), "Fields".to_string()],
                    rows,
                };
                Ok(ExecutionResult::DataSet(dataset))
            }
            Err(e) => Ok(ExecutionResult::Error(format!("Failed to show tag indexes: {}", e))),
        }
    }

    fn open(&mut self) -> crate::query::executor::base::DBResult<()> { self.base.open() }
    fn close(&mut self) -> crate::query::executor::base::DBResult<()> { self.base.close() }
    fn is_open(&self) -> bool { self.base.is_open() }
    fn id(&self) -> i64 { self.base.id }
    fn name(&self) -> &str { "ShowTagIndexesExecutor" }
    fn description(&self) -> &str { "Shows all tag indexes" }
    fn stats(&self) -> &crate::query::executor::base::ExecutorStats { self.base.get_stats() }
    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats { self.base.get_stats_mut() }
}
