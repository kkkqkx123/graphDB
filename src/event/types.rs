//! 事件类型定义

use crate::core::{Edge, Value, Vertex};
use std::fmt;

/// 存储操作事件
#[derive(Debug, Clone)]
pub enum StorageEvent {
    /// 顶点插入事件
    VertexInserted {
        space_id: u64,
        vertex: Vertex,
        timestamp: u64,
    },
    /// 顶点更新事件
    VertexUpdated {
        space_id: u64,
        old_vertex: Vertex,
        new_vertex: Vertex,
        changed_fields: Vec<String>,
        timestamp: u64,
    },
    /// 顶点删除事件
    VertexDeleted {
        space_id: u64,
        vertex_id: Value,
        tag_name: String,
        timestamp: u64,
    },
    /// 边插入事件
    EdgeInserted {
        space_id: u64,
        edge: Edge,
        timestamp: u64,
    },
    /// 边删除事件
    EdgeDeleted {
        space_id: u64,
        src: Value,
        dst: Value,
        edge_type: String,
        rank: i64,
        timestamp: u64,
    },
}

impl StorageEvent {
    /// 获取事件时间戳
    pub fn timestamp(&self) -> u64 {
        match self {
            StorageEvent::VertexInserted { timestamp, .. } => *timestamp,
            StorageEvent::VertexUpdated { timestamp, .. } => *timestamp,
            StorageEvent::VertexDeleted { timestamp, .. } => *timestamp,
            StorageEvent::EdgeInserted { timestamp, .. } => *timestamp,
            StorageEvent::EdgeDeleted { timestamp, .. } => *timestamp,
        }
    }

    /// 获取空间 ID
    pub fn space_id(&self) -> u64 {
        match self {
            StorageEvent::VertexInserted { space_id, .. } => *space_id,
            StorageEvent::VertexUpdated { space_id, .. } => *space_id,
            StorageEvent::VertexDeleted { space_id, .. } => *space_id,
            StorageEvent::EdgeInserted { space_id, .. } => *space_id,
            StorageEvent::EdgeDeleted { space_id, .. } => *space_id,
        }
    }
}

/// 事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    /// 顶点事件
    VertexEvent,
    /// 边事件
    EdgeEvent,
}

impl From<&StorageEvent> for EventType {
    fn from(event: &StorageEvent) -> Self {
        match event {
            StorageEvent::VertexInserted { .. }
            | StorageEvent::VertexUpdated { .. }
            | StorageEvent::VertexDeleted { .. } => EventType::VertexEvent,
            StorageEvent::EdgeInserted { .. } | StorageEvent::EdgeDeleted { .. } => {
                EventType::EdgeEvent
            }
        }
    }
}

/// 订阅 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(pub u64);

impl fmt::Display for SubscriptionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SubscriptionId({})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_from_storage_event() {
        let vertex_event = StorageEvent::VertexInserted {
            space_id: 1,
            vertex: create_test_vertex(),
            timestamp: 0,
        };
        assert_eq!(EventType::from(&vertex_event), EventType::VertexEvent);
    }

    fn create_test_vertex() -> Vertex {
        use std::collections::HashMap;
        Vertex {
            vid: Box::new(Value::Int64(1)),
            id: 1,
            tags: vec![],
            properties: HashMap::new(),
        }
    }
}
