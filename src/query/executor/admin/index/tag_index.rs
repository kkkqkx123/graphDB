//! 标签索引执行器
//!
//! 提供标签索引的创建、删除、描述和列出功能。

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::core::{DataSet, Row, Value};
use crate::query::executor::base::{BaseExecutor, Executor, HasStorage};
use crate::storage::StorageEngine;

/// 索引信息
#[derive(Debug, Clone)]
pub struct IndexInfo {
    pub space_name: String,
    pub index_name: String,
    pub tag_name: String,
    pub fields: Vec<String>,
    pub comment: Option<String>,
}

impl IndexInfo {
    pub fn new(space_name: String, index_name: String, tag_name: String) -> Self {
        Self {
            space_name,
            index_name,
            tag_name,
            fields: Vec::new(),
            comment: None,
        }
    }

    pub fn with_fields(mut self, fields: Vec<String>) -> Self {
        self.fields = fields;
        self
    }

    pub fn with_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }
}

/// 标签索引描述信息
#[derive(Debug, Clone)]
pub struct TagIndexDesc {
    pub index_id: i32,
    pub index_name: String,
    pub tag_name: String,
    pub fields: Vec<String>,
    pub comment: Option<String>,
}

/// 创建标签索引执行器
#[derive(Debug)]
pub struct CreateTagIndexExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    index_info: IndexInfo,
    if_not_exists: bool,
}

impl<S: StorageEngine> CreateTagIndexExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, index_info: IndexInfo) -> Self {
        Self {
            base: BaseExecutor::new(id, "CreateTagIndexExecutor".to_string(), storage),
            index_info,
            if_not_exists: false,
        }
    }

    pub fn with_if_not_exists(id: i64, storage: Arc<Mutex<S>>, index_info: IndexInfo) -> Self {
        Self {
            base: BaseExecutor::new(id, "CreateTagIndexExecutor".to_string(), storage),
            index_info,
            if_not_exists: true,
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for CreateTagIndexExecutor<S> {
    async fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::StorageError(format!("Storage lock poisoned: {}", e))
        })?;

        let result = storage_guard.create_tag_index(&self.index_info);

        match result {
            Ok(true) => Ok(ExecutionResult::Success),
            Ok(false) => {
                if self.if_not_exists {
                    Ok(ExecutionResult::Success)
                } else {
                    Ok(ExecutionResult::Error(format!("Index '{}' already exists", self.index_info.index_name)))
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

/// 删除标签索引执行器
#[derive(Debug)]
pub struct DropTagIndexExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    space_name: String,
    index_name: String,
    if_exists: bool,
}

impl<S: StorageEngine> DropTagIndexExecutor<S> {
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

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for DropTagIndexExecutor<S> {
    async fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::StorageError(format!("Storage lock poisoned: {}", e))
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

/// 描述标签索引执行器
#[derive(Debug)]
pub struct DescTagIndexExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    space_name: String,
    index_name: String,
}

impl<S: StorageEngine> DescTagIndexExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, space_name: String, index_name: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "DescTagIndexExecutor".to_string(), storage),
            space_name,
            index_name,
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for DescTagIndexExecutor<S> {
    async fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::StorageError(format!("Storage lock poisoned: {}", e))
        })?;

        let result = storage_guard.get_tag_index_desc(&self.space_name, &self.index_name);

        match result {
            Ok(Some(desc)) => {
                let rows = vec![Row::new(vec![
                    Value::String(desc.index_name),
                    Value::String(desc.tag_name),
                    Value::String(desc.fields.join(", ")),
                    Value::String(desc.comment.unwrap_or_else(|| "".to_string())),
                ])];

                let dataset = DataSet {
                    columns: vec!["Index Name".to_string(), "Tag Name".to_string(), "Fields".to_string(), "Comment".to_string()],
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

/// 列出标签索引执行器
#[derive(Debug)]
pub struct ShowTagIndexesExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    space_name: String,
}

impl<S: StorageEngine> ShowTagIndexesExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, space_name: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "ShowTagIndexesExecutor".to_string(), storage),
            space_name,
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for ShowTagIndexesExecutor<S> {
    async fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::StorageError(format!("Storage lock poisoned: {}", e))
        })?;

        let result = storage_guard.list_tag_indexes(&self.space_name);

        match result {
            Ok(indexes) => {
                let rows: Vec<Row> = indexes
                    .iter()
                    .map(|desc| {
                        Row::new(vec![
                            Value::String(desc.index_name.clone()),
                            Value::String(desc.tag_name.clone()),
                            Value::String(desc.fields.join(", ")),
                        ])
                    })
                    .collect();

                let dataset = DataSet {
                    columns: vec!["Index Name".to_string(), "Tag Name".to_string(), "Fields".to_string()],
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
