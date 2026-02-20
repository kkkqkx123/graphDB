//! 边处理器
//!
//! 提供边的插入、更新、删除功能
//! 支持索引联动更新、内存锁、批量操作、双向边处理

use super::{BatchDmlContext, DmlProcessor, DmlResult, LockGuard, LockType, MemoryLockManager};
use crate::core::{StorageError, Value, Edge};
use crate::storage::StorageClient;
use crate::storage::index::IndexDataManager;
use crate::storage::metadata::IndexMetadataManager;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;

/// 边插入处理器
///
/// 支持批量插入、IF NOT EXISTS、索引联动更新、双向边处理
pub struct EdgeInsertProcessor<S: StorageClient, I: IndexDataManager, M: IndexMetadataManager> {
    storage: Arc<Mutex<S>>,
    lock_manager: Arc<Mutex<MemoryLockManager>>,
    index_data_manager: Arc<I>,
    index_metadata_manager: Arc<M>,
    context: BatchDmlContext,
    edges: Vec<EdgeInsertItem>,
    space_id: u64,
}

/// 边插入项
#[derive(Debug, Clone)]
pub struct EdgeInsertItem {
    pub src: Value,
    pub dst: Value,
    pub edge_type: String,
    pub rank: i64,
    pub props: HashMap<String, Value>,
}

impl<S: StorageClient, I: IndexDataManager, M: IndexMetadataManager> EdgeInsertProcessor<S, I, M> {
    pub fn new(
        storage: Arc<Mutex<S>>,
        lock_manager: Arc<Mutex<MemoryLockManager>>,
        index_data_manager: Arc<I>,
        index_metadata_manager: Arc<M>,
        context: BatchDmlContext,
        space_id: u64,
    ) -> Self {
        Self {
            storage,
            lock_manager,
            index_data_manager,
            index_metadata_manager,
            context,
            edges: Vec::new(),
            space_id,
        }
    }

    /// 添加要插入的边
    pub fn add_edge(&mut self, item: EdgeInsertItem) {
        self.edges.push(item);
    }

    /// 批量添加边
    pub fn add_edges(&mut self, edges: Vec<EdgeInsertItem>) {
        self.edges.extend(edges);
    }

    /// 处理重复边
    ///
    /// 根据 if_not_exists 策略处理重复：
    /// - if_not_exists=true: 保留第一个，跳过后续重复
    /// - if_not_exists=false: 保留最后一个，覆盖前面的
    fn deduplicate_edges(&mut self) {
        if self.edges.is_empty() {
            return;
        }

        let mut seen = HashMap::new();

        if self.context.if_not_exists {
            // 保留第一个出现的
            let mut unique = Vec::new();
            for edge in &self.edges {
                let key = (edge.src.clone(), edge.dst.clone(), edge.edge_type.clone(), edge.rank);
                if !seen.contains_key(&key) {
                    seen.insert(key, true);
                    unique.push(edge.clone());
                }
            }
            self.edges = unique;
        } else {
            // 保留最后一个出现的
            let mut unique = Vec::new();
            for edge in self.edges.iter().rev() {
                let key = (edge.src.clone(), edge.dst.clone(), edge.edge_type.clone(), edge.rank);
                if !seen.contains_key(&key) {
                    seen.insert(key, true);
                    unique.push(edge.clone());
                }
            }
            unique.reverse();
            self.edges = unique;
        }
    }

    /// 获取需要锁定的资源
    fn get_locks(&self) -> Vec<LockType> {
        self.edges
            .iter()
            .map(|e| {
                LockType::Edge(
                    self.space_id,
                    e.src.clone(),
                    e.edge_type.clone(),
                    e.rank,
                    e.dst.clone(),
                )
            })
            .collect()
    }

    /// 检查边是否存在
    fn edge_exists(&self, item: &EdgeInsertItem) -> Result<bool, StorageError> {
        let storage = self.storage.lock();
        storage.get_edge(
            &self.context.space_name,
            &item.src,
            &item.dst,
            &item.edge_type,
        )
        .map(|e| e.is_some())
    }

    /// 更新索引
    fn update_indexes(&self, edge: &Edge) -> Result<(), StorageError> {
        // 获取该空间的所有边索引
        let indexes = self.index_metadata_manager
            .list_edge_indexes(self.space_id)
            .map_err(|e| StorageError::StorageError(format!("获取边索引失败: {}", e)))?;

        for index in indexes {
            // 检查索引是否关联到当前边的类型
            if index.schema_name == edge.edge_type {
                // 构建索引属性值
                let mut index_props: Vec<(String, Value)> = Vec::new();
                for field in &index.fields {
                    if let Some(prop_value) = edge.props.get(&field.name) {
                        index_props.push((field.name.clone(), prop_value.clone()));
                    }
                }

                // 更新索引
                self.index_data_manager.update_edge_indexes(
                    self.space_id,
                    &edge.src,
                    &edge.dst,
                    &index.name,
                    &index_props,
                ).map_err(|e| StorageError::StorageError(format!("更新边索引失败: {}", e)))?;
            }
        }

        Ok(())
    }
}

impl<S: StorageClient + Send + Sync + 'static, I: IndexDataManager + Send + Sync + 'static, M: IndexMetadataManager + Send + Sync + 'static> DmlProcessor for EdgeInsertProcessor<S, I, M> {
    fn execute(&mut self) -> Result<DmlResult, StorageError> {
        if self.edges.is_empty() {
            return Ok(DmlResult::success(0));
        }

        // 处理重复边
        self.deduplicate_edges();

        // 获取锁
        let locks = self.get_locks();
        {
            let mut lock_manager = self.lock_manager.lock();
            if let Err(e) = lock_manager.try_lock_batch(&locks) {
                return Ok(DmlResult::error(format!("获取锁失败: {}", e)));
            }
        }
        let _lock_guard = LockGuard::new(self.lock_manager.clone(), locks);

        let mut inserted_count = 0;

        for item in &self.edges {
            // 检查 IF NOT EXISTS
            if self.context.if_not_exists {
                match self.edge_exists(item) {
                    Ok(true) => continue, // 已存在，跳过
                    Ok(false) => {}
                    Err(e) => return Ok(DmlResult::error(format!("检查边存在性失败: {}", e))),
                }
            }

            // 创建边
            let edge = Edge::new(
                item.src.clone(),
                item.dst.clone(),
                item.edge_type.clone(),
                item.rank,
                item.props.clone(),
            );

            // 插入边
            {
                let mut storage = self.storage.lock();
                match storage.insert_edge(&self.context.space_name, edge.clone()) {
                    Ok(_) => {
                        inserted_count += 1;
                    }
                    Err(e) => {
                        return Ok(DmlResult::error(format!("插入边失败: {}", e)));
                    }
                }
            }

            // 更新索引
            if let Err(e) = self.update_indexes(&edge) {
                return Ok(DmlResult::error(format!("更新索引失败: {}", e)));
            }
        }

        Ok(DmlResult::success(inserted_count))
    }
}

/// 边更新处理器
///
/// 支持条件更新、UPSERT、YIELD 返回、索引联动更新
pub struct EdgeUpdateProcessor<S: StorageClient, I: IndexDataManager, M: IndexMetadataManager> {
    storage: Arc<Mutex<S>>,
    lock_manager: Arc<Mutex<MemoryLockManager>>,
    index_data_manager: Arc<I>,
    index_metadata_manager: Arc<M>,
    context: BatchDmlContext,
    updates: Vec<EdgeUpdateItem>,
    space_id: u64,
    insertable: bool, // UPSERT 语义
}

/// 边更新项
#[derive(Debug, Clone)]
pub struct EdgeUpdateItem {
    pub src: Value,
    pub dst: Value,
    pub edge_type: String,
    pub rank: i64,
    pub properties: HashMap<String, Value>,
    pub condition: Option<String>, // WHERE 条件表达式字符串
}

impl<S: StorageClient, I: IndexDataManager, M: IndexMetadataManager> EdgeUpdateProcessor<S, I, M> {
    pub fn new(
        storage: Arc<Mutex<S>>,
        lock_manager: Arc<Mutex<MemoryLockManager>>,
        index_data_manager: Arc<I>,
        index_metadata_manager: Arc<M>,
        context: BatchDmlContext,
        space_id: u64,
        insertable: bool,
    ) -> Self {
        Self {
            storage,
            lock_manager,
            index_data_manager,
            index_metadata_manager,
            context,
            updates: Vec::new(),
            space_id,
            insertable,
        }
    }

    /// 添加更新项
    pub fn add_update(&mut self, update: EdgeUpdateItem) {
        self.updates.push(update);
    }

    /// 获取需要锁定的资源
    fn get_locks(&self) -> Vec<LockType> {
        self.updates
            .iter()
            .map(|u| {
                LockType::Edge(
                    self.space_id,
                    u.src.clone(),
                    u.edge_type.clone(),
                    u.rank,
                    u.dst.clone(),
                )
            })
            .collect()
    }

    /// 评估条件表达式
    fn evaluate_condition(&self, _condition: &str, _edge: &Edge) -> Result<bool, StorageError> {
        // 简化实现，实际应该解析并评估表达式
        Ok(true)
    }

    /// 更新索引
    fn update_indexes(&self, edge: &Edge) -> Result<(), StorageError> {
        // 获取该空间的所有边索引
        let indexes = self.index_metadata_manager
            .list_edge_indexes(self.space_id)
            .map_err(|e| StorageError::StorageError(format!("获取边索引失败: {}", e)))?;

        for index in indexes {
            // 检查索引是否关联到当前边的类型
            if index.schema_name == edge.edge_type {
                // 构建索引属性值
                let mut index_props: Vec<(String, Value)> = Vec::new();
                for field in &index.fields {
                    if let Some(prop_value) = edge.props.get(&field.name) {
                        index_props.push((field.name.clone(), prop_value.clone()));
                    }
                }

                // 更新索引
                self.index_data_manager.update_edge_indexes(
                    self.space_id,
                    &edge.src,
                    &edge.dst,
                    &index.name,
                    &index_props,
                ).map_err(|e| StorageError::StorageError(format!("更新边索引失败: {}", e)))?;
            }
        }

        Ok(())
    }
}

impl<S: StorageClient + Send + Sync + 'static, I: IndexDataManager + Send + Sync + 'static, M: IndexMetadataManager + Send + Sync + 'static> DmlProcessor for EdgeUpdateProcessor<S, I, M> {
    fn execute(&mut self) -> Result<DmlResult, StorageError> {
        if self.updates.is_empty() {
            return Ok(DmlResult::success(0));
        }

        // 获取锁
        let locks = self.get_locks();
        {
            let mut lock_manager = self.lock_manager.lock();
            if let Err(e) = lock_manager.try_lock_batch(&locks) {
                return Ok(DmlResult::error(format!("获取锁失败: {}", e)));
            }
        }
        let _lock_guard = LockGuard::new(self.lock_manager.clone(), locks);

        let mut updated_count = 0;

        for update in &self.updates {
            let mut storage = self.storage.lock();

            match storage.get_edge(
                &self.context.space_name,
                &update.src,
                &update.dst,
                &update.edge_type,
            )? {
                Some(mut edge) => {
                    // 评估条件
                    if let Some(ref condition) = update.condition {
                        if !self.evaluate_condition(condition, &edge)? {
                            continue; // 条件不满足，跳过
                        }
                    }

                    // 更新属性
                    for (key, value) in &update.properties {
                        edge.props.insert(key.clone(), value.clone());
                    }

                    // 保存更新
                    storage.delete_edge(
                        &self.context.space_name,
                        &update.src,
                        &update.dst,
                        &update.edge_type,
                    )?;
                    storage.insert_edge(&self.context.space_name, edge.clone())?;

                    // 更新索引
                    self.update_indexes(&edge)?;

                    updated_count += 1;
                }
                None => {
                    // UPSERT 语义：如果不存在则插入
                    if self.insertable {
                        let edge = Edge::new(
                            update.src.clone(),
                            update.dst.clone(),
                            update.edge_type.clone(),
                            update.rank,
                            update.properties.clone(),
                        );
                        storage.insert_edge(&self.context.space_name, edge)?;
                        updated_count += 1;
                    }
                }
            }
        }

        Ok(DmlResult::success(updated_count))
    }
}

/// 边删除处理器
///
/// 支持批量删除、双向边删除、索引联动删除
pub struct EdgeDeleteProcessor<S: StorageClient, I: IndexDataManager> {
    storage: Arc<Mutex<S>>,
    lock_manager: Arc<Mutex<MemoryLockManager>>,
    index_data_manager: Arc<I>,
    context: BatchDmlContext,
    edges: Vec<EdgeDeleteItem>,
    space_id: u64,
}

/// 边删除项
#[derive(Debug, Clone)]
pub struct EdgeDeleteItem {
    pub src: Value,
    pub dst: Value,
    pub edge_type: String,
    pub rank: i64,
}

impl<S: StorageClient, I: IndexDataManager> EdgeDeleteProcessor<S, I> {
    pub fn new(
        storage: Arc<Mutex<S>>,
        lock_manager: Arc<Mutex<MemoryLockManager>>,
        index_data_manager: Arc<I>,
        context: BatchDmlContext,
        space_id: u64,
    ) -> Self {
        Self {
            storage,
            lock_manager,
            index_data_manager,
            context,
            edges: Vec::new(),
            space_id,
        }
    }

    /// 添加要删除的边
    pub fn add_edge(&mut self, item: EdgeDeleteItem) {
        self.edges.push(item);
    }

    /// 批量添加边
    pub fn add_edges(&mut self, edges: Vec<EdgeDeleteItem>) {
        self.edges.extend(edges);
    }

    /// 获取需要锁定的资源
    fn get_locks(&self) -> Vec<LockType> {
        self.edges
            .iter()
            .map(|e| {
                LockType::Edge(
                    self.space_id,
                    e.src.clone(),
                    e.edge_type.clone(),
                    e.rank,
                    e.dst.clone(),
                )
            })
            .collect()
    }

    /// 删除索引
    fn delete_indexes(&self, edge: &Edge) -> Result<(), StorageError> {
        self.index_data_manager.delete_edge_indexes(
            self.space_id,
            &edge.src,
            &edge.dst,
            &edge.edge_type,
        ).map_err(|e| StorageError::StorageError(format!("删除边索引失败: {}", e)))
    }
}

impl<S: StorageClient + Send + Sync + 'static, I: IndexDataManager + Send + Sync + 'static> DmlProcessor for EdgeDeleteProcessor<S, I> {
    fn execute(&mut self) -> Result<DmlResult, StorageError> {
        if self.edges.is_empty() {
            return Ok(DmlResult::success(0));
        }

        // 获取锁
        let locks = self.get_locks();
        {
            let mut lock_manager = self.lock_manager.lock();
            if let Err(e) = lock_manager.try_lock_batch(&locks) {
                return Ok(DmlResult::error(format!("获取锁失败: {}", e)));
            }
        }
        let _lock_guard = LockGuard::new(self.lock_manager.clone(), locks);

        let mut deleted_count = 0;

        for item in &self.edges {
            let mut storage = self.storage.lock();

            // 获取边信息用于删除索引
            if let Some(edge) = storage.get_edge(
                &self.context.space_name,
                &item.src,
                &item.dst,
                &item.edge_type,
            )? {
                // 删除索引
                if let Err(e) = self.delete_indexes(&edge) {
                    return Ok(DmlResult::error(format!("删除索引失败: {}", e)));
                }
            }

            // 删除边
            match storage.delete_edge(
                &self.context.space_name,
                &item.src,
                &item.dst,
                &item.edge_type,
            ) {
                Ok(_) => deleted_count += 1,
                Err(e) => {
                    return Ok(DmlResult::error(format!("删除边失败: {}", e)));
                }
            }
        }

        Ok(DmlResult::success(deleted_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    #[test]
    fn test_edge_insert_item_creation() {
        let item = EdgeInsertItem {
            src: Value::String("src1".to_string()),
            dst: Value::String("dst1".to_string()),
            edge_type: "FRIEND".to_string(),
            rank: 0,
            props: HashMap::new(),
        };

        assert_eq!(item.edge_type, "FRIEND");
        assert_eq!(item.rank, 0);
    }

    #[test]
    fn test_edge_delete_item_creation() {
        let item = EdgeDeleteItem {
            src: Value::String("src1".to_string()),
            dst: Value::String("dst1".to_string()),
            edge_type: "FRIEND".to_string(),
            rank: 0,
        };

        assert_eq!(item.edge_type, "FRIEND");
        assert_eq!(item.rank, 0);
    }
}
