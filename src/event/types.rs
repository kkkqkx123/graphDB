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

/// 同步模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncMode {
    /// 同步模式：事件发布后立即执行 handler
    Synchronous,
    /// 异步模式：事件发布到队列，后台批量处理
    Asynchronous {
        /// 批量大小
        batch_size: usize,
        /// 刷新间隔
        flush_interval_ms: u64,
    },
}

impl Default for SyncMode {
    fn default() -> Self {
        SyncMode::Synchronous
    }
}

/// 同步配置
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// 同步模式
    pub mode: SyncMode,
    /// 是否启用事件
    pub enabled: bool,
    /// 是否在事务内同步
    pub sync_in_transaction: bool,
    /// 失败重试次数
    pub max_retries: u32,
    /// 重试间隔（毫秒）
    pub retry_interval_ms: u64,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            mode: SyncMode::Synchronous,
            enabled: true,
            sync_in_transaction: true,
            max_retries: 3,
            retry_interval_ms: 1000,
        }
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
