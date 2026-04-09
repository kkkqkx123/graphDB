//! 存储层事件包装器
//!
//! 包装 StorageClient，在存储操作时自动发布事件

use crate::core::{Edge, StorageError, Value, Vertex};
use crate::event::{EventHub, StorageEvent};
use crate::storage::StorageClient;
use std::sync::Arc;

/// 获取当前时间戳（秒）
fn get_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// 计算两个顶点之间的变更字段
fn compute_changed_fields(_old: &Vertex, new: &Vertex) -> Vec<String> {
    let mut changed = Vec::new();

    // 简单实现：检测所有字段
    // TODO: 实现更精细的字段级比较
    for tag in &new.tags {
        for (field_name, _) in &tag.properties {
            changed.push(field_name.clone());
        }
    }

    changed
}

/// 事件发射存储包装器
pub struct EventEmittingStorage<S: StorageClient> {
    inner: S,
    event_hub: Arc<crate::event::MemoryEventHub>,
    enabled: bool,
}

impl<S: StorageClient> EventEmittingStorage<S> {
    /// 创建新的事件包装存储
    pub fn new(storage: S, event_hub: Arc<crate::event::MemoryEventHub>) -> Self {
        Self {
            inner: storage,
            event_hub,
            enabled: false,
        }
    }

    /// 启用/禁用事件发布
    pub fn enable_events(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 检查事件是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 获取内部存储的引用
    pub fn inner(&self) -> &S {
        &self.inner
    }

    /// 获取内部存储的可变引用
    pub fn inner_mut(&mut self) -> &mut S {
        &mut self.inner
    }

    /// 发布事件（如果启用）
    fn publish_event(&self, event: StorageEvent) -> Result<(), StorageError> {
        if self.enabled {
            self.event_hub
                .publish(event)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        Ok(())
    }
}

impl<S: StorageClient> StorageClient for EventEmittingStorage<S> {
    fn get_vertex(&self, space: &str, id: &Value) -> Result<Option<Vertex>, StorageError> {
        self.inner.get_vertex(space, id)
    }

    fn scan_vertices(&self, space: &str) -> Result<Vec<Vertex>, StorageError> {
        self.inner.scan_vertices(space)
    }

    fn scan_vertices_by_tag(&self, space: &str, tag: &str) -> Result<Vec<Vertex>, StorageError> {
        self.inner.scan_vertices_by_tag(space, tag)
    }

    fn scan_vertices_by_prop(
        &self,
        space: &str,
        tag: &str,
        prop: &str,
        value: &Value,
    ) -> Result<Vec<Vertex>, StorageError> {
        self.inner.scan_vertices_by_prop(space, tag, prop, value)
    }

    fn get_edge(
        &self,
        space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
        rank: i64,
    ) -> Result<Option<Edge>, StorageError> {
        self.inner.get_edge(space, src, dst, edge_type, rank)
    }

    fn get_node_edges(
        &self,
        space: &str,
        node_id: &Value,
        direction: crate::core::EdgeDirection,
    ) -> Result<Vec<Edge>, StorageError> {
        self.inner.get_node_edges(space, node_id, direction)
    }

    fn get_node_edges_filtered<F>(
        &self,
        space: &str,
        node_id: &Value,
        direction: crate::core::EdgeDirection,
        filter: Option<F>,
    ) -> Result<Vec<Edge>, StorageError>
    where
        F: Fn(&Edge) -> bool,
    {
        self.inner.get_node_edges_filtered(space, node_id, direction, filter)
    }

    fn scan_edges_by_type(&self, space: &str, edge_type: &str) -> Result<Vec<Edge>, StorageError> {
        self.inner.scan_edges_by_type(space, edge_type)
    }

    fn scan_all_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError> {
        self.inner.scan_all_edges(space)
    }

    fn insert_vertex(&mut self, space: &str, vertex: Vertex) -> Result<Value, StorageError> {
        let result = self.inner.insert_vertex(space, vertex.clone())?;

        if self.enabled {
            let space_id = self.inner.get_space_id(space)?;
            let event = StorageEvent::VertexInserted {
                space_id,
                vertex,
                timestamp: get_timestamp(),
            };
            self.publish_event(event)?;
        }

        Ok(result)
    }

    fn update_vertex(&mut self, space: &str, vertex: Vertex) -> Result<(), StorageError> {
        let old_vertex = self
            .inner
            .get_vertex(space, &vertex.vid)?
            .ok_or_else(|| StorageError::NodeNotFound(*vertex.vid.clone()))?;

        let changed_fields = compute_changed_fields(&old_vertex, &vertex);

        self.inner.update_vertex(space, vertex.clone())?;

        if self.enabled {
            let space_id = self.inner.get_space_id(space)?;
            let event = StorageEvent::VertexUpdated {
                space_id,
                old_vertex,
                new_vertex: vertex,
                changed_fields,
                timestamp: get_timestamp(),
            };
            self.publish_event(event)?;
        }

        Ok(())
    }

    fn delete_vertex(&mut self, space: &str, id: &Value) -> Result<(), StorageError> {
        let vertex = self
            .inner
            .get_vertex(space, id)?
            .ok_or_else(|| StorageError::NodeNotFound(id.clone()))?;

        self.inner.delete_vertex(space, id)?;

        if self.enabled {
            let space_id = self.inner.get_space_id(space)?;
            for tag in &vertex.tags {
                let event = StorageEvent::VertexDeleted {
                    space_id,
                    vertex_id: id.clone(),
                    tag_name: tag.name.clone(),
                    timestamp: get_timestamp(),
                };
                self.publish_event(event)?;
            }
        }

        Ok(())
    }

    fn delete_vertex_with_edges(&mut self, space: &str, id: &Value) -> Result<(), StorageError> {
        let vertex = self
            .inner
            .get_vertex(space, id)?
            .ok_or_else(|| StorageError::NodeNotFound(id.clone()))?;

        self.inner.delete_vertex_with_edges(space, id)?;

        if self.enabled {
            let space_id = self.inner.get_space_id(space)?;
            for tag in &vertex.tags {
                let event = StorageEvent::VertexDeleted {
                    space_id,
                    vertex_id: id.clone(),
                    tag_name: tag.name.clone(),
                    timestamp: get_timestamp(),
                };
                self.publish_event(event)?;
            }
        }

        Ok(())
    }

    fn batch_insert_vertices(
        &mut self,
        space: &str,
        vertices: Vec<Vertex>,
    ) -> Result<Vec<Value>, StorageError> {
        let results = self.inner.batch_insert_vertices(space, vertices.clone())?;

        if self.enabled {
            let space_id = self.inner.get_space_id(space)?;
            for vertex in vertices {
                let event = StorageEvent::VertexInserted {
                    space_id,
                    vertex,
                    timestamp: get_timestamp(),
                };
                self.publish_event(event)?;
            }
        }

        Ok(results)
    }

    fn delete_tags(
        &mut self,
        space: &str,
        vertex_id: &Value,
        tag_names: &[String],
    ) -> Result<usize, StorageError> {
        self.inner.delete_tags(space, vertex_id, tag_names)
    }

    fn insert_edge(&mut self, space: &str, edge: Edge) -> Result<(), StorageError> {
        self.inner.insert_edge(space, edge.clone())?;

        if self.enabled {
            let space_id = self.inner.get_space_id(space)?;
            let event = StorageEvent::EdgeInserted {
                space_id,
                edge,
                timestamp: get_timestamp(),
            };
            self.publish_event(event)?;
        }

        Ok(())
    }

    fn delete_edge(
        &mut self,
        space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
        rank: i64,
    ) -> Result<(), StorageError> {
        self.inner.delete_edge(space, src, dst, edge_type, rank)?;

        if self.enabled {
            let space_id = self.inner.get_space_id(space)?;
            let event = StorageEvent::EdgeDeleted {
                space_id,
                src: src.clone(),
                dst: dst.clone(),
                edge_type: edge_type.to_string(),
                rank,
                timestamp: get_timestamp(),
            };
            self.publish_event(event)?;
        }

        Ok(())
    }

    fn batch_insert_edges(&mut self, space: &str, edges: Vec<Edge>) -> Result<(), StorageError> {
        self.inner.batch_insert_edges(space, edges.clone())?;

        if self.enabled {
            let space_id = self.inner.get_space_id(space)?;
            for edge in edges {
                let event = StorageEvent::EdgeInserted {
                    space_id,
                    edge,
                    timestamp: get_timestamp(),
                };
                self.publish_event(event)?;
            }
        }

        Ok(())
    }

    fn create_space(&mut self, space: &crate::core::types::SpaceInfo) -> Result<bool, StorageError> {
        self.inner.create_space(space)
    }

    fn drop_space(&mut self, space: &str) -> Result<bool, StorageError> {
        self.inner.drop_space(space)
    }

    fn get_space(&self, space: &str) -> Result<Option<crate::core::types::SpaceInfo>, StorageError> {
        self.inner.get_space(space)
    }

    fn get_space_by_id(
        &self,
        space_id: u64,
    ) -> Result<Option<crate::core::types::SpaceInfo>, StorageError> {
        self.inner.get_space_by_id(space_id)
    }

    fn list_spaces(&self) -> Result<Vec<crate::core::types::SpaceInfo>, StorageError> {
        self.inner.list_spaces()
    }

    fn get_space_id(&self, space: &str) -> Result<u64, StorageError> {
        self.inner.get_space_id(space)
    }

    fn space_exists(&self, space: &str) -> bool {
        self.inner.space_exists(space)
    }

    fn clear_space(&mut self, space: &str) -> Result<bool, StorageError> {
        self.inner.clear_space(space)
    }

    fn alter_space_comment(
        &mut self,
        space_id: u64,
        comment: String,
    ) -> Result<bool, StorageError> {
        self.inner.alter_space_comment(space_id, comment)
    }

    fn create_tag(
        &mut self,
        space: &str,
        tag: &crate::core::types::TagInfo,
    ) -> Result<bool, StorageError> {
        self.inner.create_tag(space, tag)
    }

    fn alter_tag(
        &mut self,
        space: &str,
        tag: &str,
        additions: Vec<crate::core::types::PropertyDef>,
        deletions: Vec<String>,
    ) -> Result<bool, StorageError> {
        self.inner.alter_tag(space, tag, additions, deletions)
    }

    fn drop_tag(&mut self, space: &str, tag: &str) -> Result<bool, StorageError> {
        self.inner.drop_tag(space, tag)
    }

    fn get_tag(&self, space: &str, tag: &str) -> Result<Option<crate::core::types::TagInfo>, StorageError> {
        self.inner.get_tag(space, tag)
    }

    fn list_tags(&self, space: &str) -> Result<Vec<crate::core::types::TagInfo>, StorageError> {
        self.inner.list_tags(space)
    }

    fn create_edge_type(
        &mut self,
        space: &str,
        edge: &crate::core::types::EdgeTypeInfo,
    ) -> Result<bool, StorageError> {
        self.inner.create_edge_type(space, edge)
    }

    fn drop_edge_type(&mut self, space: &str, edge: &str) -> Result<bool, StorageError> {
        self.inner.drop_edge_type(space, edge)
    }

    fn get_edge_type(
        &self,
        space: &str,
        edge: &str,
    ) -> Result<Option<crate::core::types::EdgeTypeInfo>, StorageError> {
        self.inner.get_edge_type(space, edge)
    }

    fn list_edge_types(&self, space: &str) -> Result<Vec<crate::core::types::EdgeTypeInfo>, StorageError> {
        self.inner.list_edge_types(space)
    }

    fn alter_edge_type(
        &mut self,
        space: &str,
        edge_type: &str,
        additions: Vec<crate::core::types::PropertyDef>,
        deletions: Vec<String>,
    ) -> Result<bool, StorageError> {
        self.inner.alter_edge_type(space, edge_type, additions, deletions)
    }

    fn create_tag_index(&mut self, space: &str, info: &crate::core::types::Index) -> Result<bool, StorageError> {
        self.inner.create_tag_index(space, info)
    }

    fn drop_tag_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        self.inner.drop_tag_index(space, index)
    }

    fn get_tag_index(&self, space: &str, index: &str) -> Result<Option<crate::core::types::Index>, StorageError> {
        self.inner.get_tag_index(space, index)
    }

    fn list_tag_indexes(&self, space: &str) -> Result<Vec<crate::core::types::Index>, StorageError> {
        self.inner.list_tag_indexes(space)
    }

    fn rebuild_tag_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        self.inner.rebuild_tag_index(space, index)
    }

    fn create_edge_index(&mut self, space: &str, info: &crate::core::types::Index) -> Result<bool, StorageError> {
        self.inner.create_edge_index(space, info)
    }

    fn drop_edge_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        self.inner.drop_edge_index(space, index)
    }

    fn get_edge_index(&self, space: &str, index: &str) -> Result<Option<crate::core::types::Index>, StorageError> {
        self.inner.get_edge_index(space, index)
    }

    fn list_edge_indexes(&self, space: &str) -> Result<Vec<crate::core::types::Index>, StorageError> {
        self.inner.list_edge_indexes(space)
    }

    fn rebuild_edge_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        self.inner.rebuild_edge_index(space, index)
    }

    fn insert_vertex_data(
        &mut self,
        space: &str,
        info: &crate::core::types::InsertVertexInfo,
    ) -> Result<bool, StorageError> {
        self.inner.insert_vertex_data(space, info)
    }

    fn insert_edge_data(
        &mut self,
        space: &str,
        info: &crate::core::types::InsertEdgeInfo,
    ) -> Result<bool, StorageError> {
        self.inner.insert_edge_data(space, info)
    }

    fn delete_vertex_data(&mut self, space: &str, vertex_id: &str) -> Result<bool, StorageError> {
        self.inner.delete_vertex_data(space, vertex_id)
    }

    fn delete_edge_data(
        &mut self,
        space: &str,
        src: &str,
        dst: &str,
        rank: i64,
    ) -> Result<bool, StorageError> {
        self.inner.delete_edge_data(space, src, dst, rank)
    }

    fn update_data(&mut self, space: &str, info: &crate::core::types::UpdateInfo) -> Result<bool, StorageError> {
        self.inner.update_data(space, info)
    }

    fn change_password(&mut self, info: &crate::core::types::PasswordInfo) -> Result<bool, StorageError> {
        self.inner.change_password(info)
    }

    fn create_user(&mut self, info: &crate::core::types::UserInfo) -> Result<bool, StorageError> {
        self.inner.create_user(info)
    }

    fn alter_user(&mut self, info: &crate::core::types::UserAlterInfo) -> Result<bool, StorageError> {
        self.inner.alter_user(info)
    }

    fn drop_user(&mut self, username: &str) -> Result<bool, StorageError> {
        self.inner.drop_user(username)
    }

    fn grant_role(
        &mut self,
        username: &str,
        space_id: u64,
        role: crate::core::RoleType,
    ) -> Result<bool, StorageError> {
        self.inner.grant_role(username, space_id, role)
    }

    fn revoke_role(&mut self, username: &str, space_id: u64) -> Result<bool, StorageError> {
        self.inner.revoke_role(username, space_id)
    }

    fn lookup_index(
        &self,
        space: &str,
        index: &str,
        value: &Value,
    ) -> Result<Vec<Value>, StorageError> {
        self.inner.lookup_index(space, index, value)
    }

    fn lookup_index_with_score(
        &self,
        space: &str,
        index: &str,
        value: &Value,
    ) -> Result<Vec<(Value, f32)>, StorageError> {
        self.inner.lookup_index_with_score(space, index, value)
    }

    fn get_vertex_with_schema(
        &self,
        space: &str,
        tag: &str,
        id: &Value,
    ) -> Result<Option<(crate::storage::Schema, Vec<u8>)>, StorageError> {
        self.inner.get_vertex_with_schema(space, tag, id)
    }

    fn get_edge_with_schema(
        &self,
        space: &str,
        edge_type: &str,
        src: &Value,
        dst: &Value,
    ) -> Result<Option<(crate::storage::Schema, Vec<u8>)>, StorageError> {
        self.inner.get_edge_with_schema(space, edge_type, src, dst)
    }

    fn scan_vertices_with_schema(
        &self,
        space: &str,
        tag: &str,
    ) -> Result<Vec<(crate::storage::Schema, Vec<u8>)>, StorageError> {
        self.inner.scan_vertices_with_schema(space, tag)
    }

    fn scan_edges_with_schema(
        &self,
        space: &str,
        edge_type: &str,
    ) -> Result<Vec<(crate::storage::Schema, Vec<u8>)>, StorageError> {
        self.inner.scan_edges_with_schema(space, edge_type)
    }

    fn load_from_disk(&mut self) -> Result<(), StorageError> {
        self.inner.load_from_disk()
    }

    fn save_to_disk(&self) -> Result<(), StorageError> {
        self.inner.save_to_disk()
    }

    fn get_storage_stats(&self) -> crate::storage::StorageStats {
        self.inner.get_storage_stats()
    }

    fn find_dangling_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError> {
        self.inner.find_dangling_edges(space)
    }

    fn repair_dangling_edges(&mut self, space: &str) -> Result<usize, StorageError> {
        self.inner.repair_dangling_edges(space)
    }

    fn get_db_path(&self) -> &str {
        self.inner.get_db_path()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::test_mock::MockStorage;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_event_emitting_storage_insert() {
        let inner = MockStorage::new();
        let event_hub = Arc::new(crate::event::MemoryEventHub::new());
        let mut storage = EventEmittingStorage::new(inner, event_hub.clone());
        storage.enable_events(true);

        let counter = Arc::new(AtomicUsize::new(0));
        let c = counter.clone();

        event_hub
            .subscribe(crate::event::EventType::VertexEvent, move |_| {
                c.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .unwrap();

        let vertex = create_test_vertex();
        storage
            .insert_vertex("test_space", vertex)
            .expect("insert should succeed");

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    fn create_test_vertex() -> Vertex {
        Vertex {
            vid: Value::Int64(1),
            tags: vec![],
        }
    }
}
