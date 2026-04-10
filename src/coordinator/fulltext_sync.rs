//! 全文索引自动数据同步处理器
//!
//! 监听存储层事件，自动同步到全文索引

use crate::coordinator::fulltext::{ChangeType, FulltextCoordinator};
use crate::core::Value;
use crate::event::hub::EventHub;
use crate::event::{EventError, EventType, StorageEvent};
use std::collections::HashMap;
use std::sync::Arc;

/// 全文索引同步处理器
pub struct FulltextSyncHandler {
    coordinator: Arc<FulltextCoordinator>,
}

impl FulltextSyncHandler {
    /// 创建新的同步处理器
    pub fn new(coordinator: Arc<FulltextCoordinator>) -> Self {
        Self { coordinator }
    }

    /// 处理存储事件
    pub fn handle_event(&self, event: &StorageEvent) -> Result<(), EventError> {
        match event {
            StorageEvent::VertexInserted {
                space_id, vertex, ..
            } => self.on_vertex_inserted(*space_id, vertex),
            StorageEvent::VertexUpdated {
                space_id,
                new_vertex,
                changed_fields,
                ..
            } => self.on_vertex_updated(*space_id, new_vertex, changed_fields),
            StorageEvent::VertexDeleted {
                space_id,
                tag_name,
                vertex_id,
                ..
            } => self.on_vertex_deleted(*space_id, tag_name, vertex_id),
            // 处理边事件
            StorageEvent::EdgeInserted { space_id, edge, .. } => {
                self.on_edge_inserted(*space_id, edge)
            }
            StorageEvent::EdgeDeleted {
                space_id,
                src,
                dst,
                edge_type,
                rank,
                ..
            } => self.on_edge_deleted(*space_id, src, dst, edge_type, *rank),
        }
    }

    /// 处理顶点插入事件
    fn on_vertex_inserted(
        &self,
        space_id: u64,
        vertex: &crate::core::Vertex,
    ) -> Result<(), EventError> {
        for tag in &vertex.tags {
            let mut properties = HashMap::new();

            for (field_name, value) in &tag.properties {
                if let Value::String(_) = value {
                    properties.insert(field_name.clone(), value.clone());
                }
            }

            if !properties.is_empty() {
                futures::executor::block_on(self.coordinator.on_vertex_change(
                    space_id,
                    &tag.name,
                    &vertex.vid,
                    &properties,
                    ChangeType::Insert,
                ))
                .map_err(|e| EventError::HandlerError(e.to_string()))?;
            }
        }

        Ok(())
    }

    /// 处理顶点更新事件
    fn on_vertex_updated(
        &self,
        space_id: u64,
        vertex: &crate::core::Vertex,
        changed_fields: &[String],
    ) -> Result<(), EventError> {
        for tag in &vertex.tags {
            let mut properties = HashMap::new();

            for field_name in changed_fields {
                if let Some(value) = tag.properties.get(field_name) {
                    if let Value::String(_) = value {
                        properties.insert(field_name.clone(), value.clone());
                    }
                }
            }

            if !properties.is_empty() {
                futures::executor::block_on(self.coordinator.on_vertex_change(
                    space_id,
                    &tag.name,
                    &vertex.vid,
                    &properties,
                    ChangeType::Update,
                ))
                .map_err(|e| EventError::HandlerError(e.to_string()))?;
            }
        }

        Ok(())
    }

    /// 处理顶点删除事件
    fn on_vertex_deleted(
        &self,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
    ) -> Result<(), EventError> {
        futures::executor::block_on(
            self.coordinator
                .on_vertex_deleted(space_id, tag_name, vertex_id),
        )
        .map_err(|e| EventError::HandlerError(e.to_string()))?;

        Ok(())
    }

    /// 处理边插入事件
    fn on_edge_inserted(&self, space_id: u64, edge: &crate::core::Edge) -> Result<(), EventError> {
        // 为边的字符串属性建立索引
        let mut properties = std::collections::HashMap::new();

        for (field_name, value) in &edge.props {
            if let crate::core::Value::String(_) = value {
                properties.insert(field_name.clone(), value.clone());
            }
        }

        if !properties.is_empty() {
            // 使用边的唯一标识作为文档 ID
            let edge_doc_id = format!(
                "edge_{}_{}_{}_{}",
                edge.edge_type, edge.src, edge.dst, edge.ranking
            );

            for (field_name, value) in &properties {
                if let crate::core::Value::String(text) = value {
                    futures::executor::block_on(
                        self.coordinator.get_manager().index_edge_property(
                            space_id,
                            &edge.edge_type,
                            field_name,
                            &edge_doc_id,
                            text,
                        ),
                    )
                    .map_err(|e| EventError::HandlerError(e.to_string()))?;
                }
            }
        }

        Ok(())
    }

    /// 处理边删除事件
    fn on_edge_deleted(
        &self,
        space_id: u64,
        src: &crate::core::Value,
        dst: &crate::core::Value,
        edge_type: &str,
        rank: i64,
    ) -> Result<(), EventError> {
        // 使用边的唯一标识作为文档 ID
        let edge_doc_id = format!("edge_{}_{}_{}_{}", edge_type, src, dst, rank);

        // 删除边的所有全文索引
        futures::executor::block_on(self.coordinator.get_manager().delete_edge_index(
            space_id,
            edge_type,
            &edge_doc_id,
        ))
        .map_err(|e| EventError::HandlerError(e.to_string()))?;

        Ok(())
    }
}

/// 注册全文索引同步处理器到事件总线
pub fn register_fulltext_sync(
    coordinator: Arc<FulltextCoordinator>,
    event_hub: Arc<crate::event::MemoryEventHub>,
) -> Result<crate::event::SubscriptionId, EventError> {
    let handler = FulltextSyncHandler::new(coordinator);

    let subscription_id = event_hub.subscribe(EventType::VertexEvent, move |event| {
        handler.handle_event(event)
    })?;

    Ok(subscription_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_sync_handler_creation() {
        // 注意：这个测试需要 FulltextCoordinator 的实例
        // 实际测试需要在集成测试中完成
        let _config = crate::search::FulltextConfig::default();
        // 这里只是演示，实际使用需要正确初始化
    }
}
