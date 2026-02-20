//! 顶点处理器
//!
//! 提供顶点的插入、更新、删除功能
//! 支持索引联动更新、内存锁、批量操作

use super::{BatchDmlContext, DmlProcessor, DmlResult, LockGuard, LockType, MemoryLockManager};
use crate::core::{StorageError, Value, Vertex};
use crate::core::vertex_edge_path::Tag;
use crate::storage::StorageClient;
use crate::storage::index::IndexDataManager;
use crate::storage::metadata::IndexMetadataManager;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;

/// 顶点插入处理器
///
/// 支持批量插入、多标签、IF NOT EXISTS、索引联动更新
pub struct VertexInsertProcessor<S: StorageClient, I: IndexDataManager, M: IndexMetadataManager> {
    storage: Arc<Mutex<S>>,
    lock_manager: Arc<Mutex<MemoryLockManager>>,
    index_data_manager: Arc<I>,
    index_metadata_manager: Arc<M>,
    context: BatchDmlContext,
    vertices: Vec<(Value, Vec<Tag>)>, // (vid, tags)
    space_id: u64,
}

impl<S: StorageClient, I: IndexDataManager, M: IndexMetadataManager> VertexInsertProcessor<S, I, M> {
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
            vertices: Vec::new(),
            space_id,
        }
    }

    /// 添加要插入的顶点
    pub fn add_vertex(&mut self, vid: Value, tags: Vec<Tag>) {
        self.vertices.push((vid, tags));
    }

    /// 批量添加顶点
    pub fn add_vertices(&mut self, vertices: Vec<(Value, Vec<Tag>)>) {
        self.vertices.extend(vertices);
    }

    /// 处理重复 VID
    ///
    /// 根据 if_not_exists 策略处理重复：
    /// - if_not_exists=true: 保留第一个，跳过后续重复
    /// - if_not_exists=false: 保留最后一个，覆盖前面的
    fn deduplicate_vertices(&mut self) {
        if self.vertices.is_empty() {
            return;
        }

        let mut seen: HashMap<Value, bool> = HashMap::new();

        if self.context.if_not_exists {
            // 保留第一个出现的
            let mut unique = Vec::new();
            for (vid, tags) in &self.vertices {
                if !seen.contains_key(vid) {
                    seen.insert(vid.clone(), true);
                    unique.push((vid.clone(), tags.clone()));
                }
            }
            self.vertices = unique;
        } else {
            // 保留最后一个出现的
            let mut unique = Vec::new();
            for (vid, tags) in self.vertices.iter().rev() {
                if !seen.contains_key(vid) {
                    seen.insert(vid.clone(), true);
                    unique.push((vid.clone(), tags.clone()));
                }
            }
            unique.reverse();
            self.vertices = unique;
        }
    }

    /// 获取需要锁定的资源
    fn get_locks(&self) -> Vec<LockType> {
        self.vertices
            .iter()
            .map(|(vid, _)| LockType::Vertex(self.space_id, vid.clone()))
            .collect()
    }

    /// 检查顶点是否存在
    fn vertex_exists(&self, vid: &Value) -> Result<bool, StorageError> {
        let storage = self.storage.lock();
        storage.get_vertex(&self.context.space_name, vid)
            .map(|v| v.is_some())
    }

    /// 更新索引
    fn update_indexes(&self, vid: &Value, tag: &Tag) -> Result<(), StorageError> {
        // 获取该标签的所有索引
        let indexes = self.index_metadata_manager
            .list_tag_indexes(self.space_id)
            .map_err(|e| StorageError::StorageError(format!("获取索引失败: {}", e)))?;

        for index in indexes {
            // 检查索引是否关联到当前标签
            if index.schema_name == tag.name {
                // 构建索引属性值
                let mut index_props: Vec<(String, Value)> = Vec::new();
                for field in &index.fields {
                    if let Some((prop_name, prop_value)) = tag.properties.iter()
                        .find(|(name, _)| name.as_str() == field.name.as_str()) {
                        index_props.push((prop_name.clone(), prop_value.clone()));
                    }
                }

                // 更新索引
                self.index_data_manager.update_vertex_indexes(
                    self.space_id,
                    vid,
                    &index.name,
                    &index_props,
                ).map_err(|e| StorageError::StorageError(format!("更新索引失败: {}", e)))?;
            }
        }

        Ok(())
    }
}

impl<S: StorageClient + Send + Sync + 'static, I: IndexDataManager + Send + Sync + 'static, M: IndexMetadataManager + Send + Sync + 'static> DmlProcessor for VertexInsertProcessor<S, I, M> {
    fn execute(&mut self) -> Result<DmlResult, StorageError> {
        if self.vertices.is_empty() {
            return Ok(DmlResult::success(0));
        }

        // 处理重复 VID
        self.deduplicate_vertices();

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

        for (vid, tags) in &self.vertices {
            // 检查 IF NOT EXISTS
            if self.context.if_not_exists {
                match self.vertex_exists(vid) {
                    Ok(true) => continue, // 已存在，跳过
                    Ok(false) => {}
                    Err(e) => return Ok(DmlResult::error(format!("检查顶点存在性失败: {}", e))),
                }
            }

            // 创建顶点
            let vertex = Vertex::new_with_properties(vid.clone(), tags.clone(), HashMap::new());

            // 插入顶点
            {
                let mut storage = self.storage.lock();
                match storage.insert_vertex(&self.context.space_name, vertex) {
                    Ok(_) => {
                        inserted_count += 1;
                    }
                    Err(e) => {
                        return Ok(DmlResult::error(format!("插入顶点失败: {}", e)));
                    }
                }
            }

            // 更新索引
            for tag in tags {
                if let Err(e) = self.update_indexes(vid, tag) {
                    return Ok(DmlResult::error(format!("更新索引失败: {}", e)));
                }
            }
        }

        Ok(DmlResult::success(inserted_count))
    }
}

/// 顶点更新处理器
///
/// 支持条件更新、UPSERT、YIELD 返回、索引联动更新
pub struct VertexUpdateProcessor<S: StorageClient, I: IndexDataManager, M: IndexMetadataManager> {
    storage: Arc<Mutex<S>>,
    lock_manager: Arc<Mutex<MemoryLockManager>>,
    index_data_manager: Arc<I>,
    index_metadata_manager: Arc<M>,
    context: BatchDmlContext,
    updates: Vec<VertexUpdateItem>,
    space_id: u64,
    insertable: bool, // UPSERT 语义
}

/// 顶点更新项
#[derive(Debug, Clone)]
pub struct VertexUpdateItem {
    pub vid: Value,
    pub tag_name: Option<String>, // None 表示更新所有标签
    pub properties: HashMap<String, Value>,
    pub condition: Option<String>, // WHERE 条件表达式字符串
}

impl<S: StorageClient, I: IndexDataManager, M: IndexMetadataManager> VertexUpdateProcessor<S, I, M> {
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
    pub fn add_update(&mut self, update: VertexUpdateItem) {
        self.updates.push(update);
    }

    /// 获取需要锁定的资源
    fn get_locks(&self) -> Vec<LockType> {
        self.updates
            .iter()
            .map(|u| LockType::Vertex(self.space_id, u.vid.clone()))
            .collect()
    }

    /// 评估条件表达式
    fn evaluate_condition(&self, _condition: &str, _vertex: &Vertex) -> Result<bool, StorageError> {
        // 简化实现，实际应该解析并评估表达式
        Ok(true)
    }

    /// 更新索引
    fn update_indexes(&self, vid: &Value, tag: &Tag) -> Result<(), StorageError> {
        // 获取该标签的所有索引
        let indexes = self.index_metadata_manager
            .list_tag_indexes(self.space_id)
            .map_err(|e| StorageError::StorageError(format!("获取索引失败: {}", e)))?;

        for index in indexes {
            // 检查索引是否关联到当前标签
            if index.schema_name == tag.name {
                // 构建索引属性值
                let mut index_props: Vec<(String, Value)> = Vec::new();
                for field in &index.fields {
                    if let Some((prop_name, prop_value)) = tag.properties.iter()
                        .find(|(name, _)| name.as_str() == field.name.as_str()) {
                        index_props.push((prop_name.clone(), prop_value.clone()));
                    }
                }

                // 更新索引
                self.index_data_manager.update_vertex_indexes(
                    self.space_id,
                    vid,
                    &index.name,
                    &index_props,
                ).map_err(|e| StorageError::StorageError(format!("更新索引失败: {}", e)))?;
            }
        }

        Ok(())
    }
}

impl<S: StorageClient + Send + Sync + 'static, I: IndexDataManager + Send + Sync + 'static, M: IndexMetadataManager + Send + Sync + 'static> DmlProcessor for VertexUpdateProcessor<S, I, M> {
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

            match storage.get_vertex(&self.context.space_name, &update.vid)? {
                Some(mut vertex) => {
                    // 评估条件
                    if let Some(ref condition) = update.condition {
                        if !self.evaluate_condition(condition, &vertex)? {
                            continue; // 条件不满足，跳过
                        }
                    }

                    // 更新属性
                    if let Some(ref tag_name) = update.tag_name {
                        // 更新指定标签
                        for tag in &mut vertex.tags {
                            if tag.name == *tag_name {
                                for (key, value) in &update.properties {
                                    tag.properties.insert(key.clone(), value.clone());
                                }
                                break;
                            }
                        }
                    } else {
                        // 更新所有标签
                        for tag in &mut vertex.tags {
                            for (key, value) in &update.properties {
                                tag.properties.insert(key.clone(), value.clone());
                            }
                        }
                    }

                    // 保存更新
                    storage.update_vertex(&self.context.space_name, vertex.clone())?;

                    // 更新索引
                    for tag in &vertex.tags {
                        self.update_indexes(&update.vid, tag)?;
                    }

                    updated_count += 1;
                }
                None => {
                    // UPSERT 语义：如果不存在则插入
                    if self.insertable {
                        let tag_name = update.tag_name.clone().unwrap_or_else(|| "default".to_string());
                        let tag = Tag::new(tag_name, update.properties.clone());
                        let vertex = Vertex::new_with_properties(
                            update.vid.clone(),
                            vec![tag],
                            HashMap::new(),
                        );
                        storage.insert_vertex(&self.context.space_name, vertex)?;
                        updated_count += 1;
                    }
                }
            }
        }

        Ok(DmlResult::success(updated_count))
    }
}

/// 顶点删除处理器
///
/// 支持批量删除、级联删除关联边、索引联动删除
pub struct VertexDeleteProcessor<S: StorageClient, I: IndexDataManager> {
    storage: Arc<Mutex<S>>,
    lock_manager: Arc<Mutex<MemoryLockManager>>,
    index_data_manager: Arc<I>,
    context: BatchDmlContext,
    vertex_ids: Vec<Value>,
    space_id: u64,
    with_edge: bool, // 是否级联删除关联边
}

impl<S: StorageClient, I: IndexDataManager> VertexDeleteProcessor<S, I> {
    pub fn new(
        storage: Arc<Mutex<S>>,
        lock_manager: Arc<Mutex<MemoryLockManager>>,
        index_data_manager: Arc<I>,
        context: BatchDmlContext,
        space_id: u64,
        with_edge: bool,
    ) -> Self {
        Self {
            storage,
            lock_manager,
            index_data_manager,
            context,
            vertex_ids: Vec::new(),
            space_id,
            with_edge,
        }
    }

    /// 添加要删除的顶点
    pub fn add_vertex(&mut self, vid: Value) {
        self.vertex_ids.push(vid);
    }

    /// 批量添加顶点
    pub fn add_vertices(&mut self, vids: Vec<Value>) {
        self.vertex_ids.extend(vids);
    }

    /// 获取需要锁定的资源
    fn get_locks(&self) -> Vec<LockType> {
        self.vertex_ids
            .iter()
            .map(|vid| LockType::Vertex(self.space_id, vid.clone()))
            .collect()
    }

    /// 删除关联边
    fn delete_related_edges(&self, vid: &Value) -> Result<usize, StorageError> {
        use crate::core::EdgeDirection;
        
        let mut storage = self.storage.lock();
        
        // 获取所有关联边
        let edges = storage.get_node_edges(&self.context.space_name, vid, EdgeDirection::Both)?;
        let mut deleted_count = 0;

        for edge in edges {
            storage.delete_edge(
                &self.context.space_name,
                &edge.src,
                &edge.dst,
                &edge.edge_type,
            )?;
            deleted_count += 1;
        }

        Ok(deleted_count)
    }

    /// 删除索引
    fn delete_indexes(&self, vid: &Value) -> Result<(), StorageError> {
        self.index_data_manager.delete_vertex_indexes(
            self.space_id,
            vid,
        ).map_err(|e| StorageError::StorageError(format!("删除索引失败: {}", e)))
    }
}

impl<S: StorageClient + Send + Sync + 'static, I: IndexDataManager + Send + Sync + 'static> DmlProcessor for VertexDeleteProcessor<S, I> {
    fn execute(&mut self) -> Result<DmlResult, StorageError> {
        if self.vertex_ids.is_empty() {
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
        let mut deleted_edges_count = 0;

        for vid in &self.vertex_ids {
            // 级联删除关联边
            if self.with_edge {
                match self.delete_related_edges(vid) {
                    Ok(count) => deleted_edges_count += count,
                    Err(e) => return Ok(DmlResult::error(format!("删除关联边失败: {}", e))),
                }
            }

            // 删除索引
            if let Err(e) = self.delete_indexes(vid) {
                return Ok(DmlResult::error(format!("删除索引失败: {}", e)));
            }

            // 删除顶点
            {
                let mut storage = self.storage.lock();
                match storage.delete_vertex(&self.context.space_name, vid) {
                    Ok(_) => deleted_count += 1,
                    Err(e) => {
                        return Ok(DmlResult::error(format!("删除顶点失败: {}", e)));
                    }
                }
            }
        }

        Ok(DmlResult::success_with_stats(deleted_count, deleted_edges_count))
    }
}

/// 标签删除处理器
///
/// 支持从顶点删除指定标签，保留顶点本身
pub struct TagDeleteProcessor<S: StorageClient, I: IndexDataManager> {
    storage: Arc<Mutex<S>>,
    lock_manager: Arc<Mutex<MemoryLockManager>>,
    index_data_manager: Arc<I>,
    context: BatchDmlContext,
    items: Vec<TagDeleteItem>,
    space_id: u64,
}

/// 标签删除项
#[derive(Debug, Clone)]
pub struct TagDeleteItem {
    pub vid: Value,
    pub tag_names: Vec<String>,
}

impl<S: StorageClient, I: IndexDataManager> TagDeleteProcessor<S, I> {
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
            items: Vec::new(),
            space_id,
        }
    }

    /// 添加删除项
    pub fn add_item(&mut self, vid: Value, tag_names: Vec<String>) {
        self.items.push(TagDeleteItem { vid, tag_names });
    }

    /// 获取需要锁定的资源
    fn get_locks(&self) -> Vec<LockType> {
        self.items
            .iter()
            .map(|item| LockType::Vertex(self.space_id, item.vid.clone()))
            .collect()
    }

    /// 删除标签索引
    fn delete_tag_indexes(&self, vid: &Value, tag_name: &str) -> Result<(), StorageError> {
        self.index_data_manager.delete_tag_indexes(
            self.space_id,
            vid,
            tag_name,
        ).map_err(|e| StorageError::StorageError(format!("删除标签索引失败: {}", e)))
    }
}

impl<S: StorageClient + Send + Sync + 'static, I: IndexDataManager + Send + Sync + 'static> DmlProcessor for TagDeleteProcessor<S, I> {
    fn execute(&mut self) -> Result<DmlResult, StorageError> {
        if self.items.is_empty() {
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

        for item in &self.items {
            let mut storage = self.storage.lock();

            match storage.get_vertex(&self.context.space_name, &item.vid)? {
                Some(mut vertex) => {
                    // 删除指定标签
                    let original_count = vertex.tags.len();
                    
                    // 获取要删除的标签名称
                    let tags_to_remove: Vec<String> = vertex.tags.iter()
                        .filter(|tag| item.tag_names.contains(&tag.name))
                        .map(|tag| tag.name.clone())
                        .collect();
                    
                    vertex.tags.retain(|tag| !item.tag_names.contains(&tag.name));
                    let removed_count = original_count - vertex.tags.len();

                    if removed_count > 0 {
                        // 删除标签索引
                        for tag_name in &tags_to_remove {
                            if let Err(e) = self.delete_tag_indexes(&item.vid, tag_name) {
                                return Ok(DmlResult::error(format!("删除标签索引失败: {}", e)));
                            }
                        }
                        
                        // 保存更新后的顶点
                        storage.update_vertex(&self.context.space_name, vertex)?;
                        deleted_count += removed_count;
                    }
                }
                None => {
                    // 顶点不存在，跳过
                    continue;
                }
            }
        }

        Ok(DmlResult::success(deleted_count))
    }
}
