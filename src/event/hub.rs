//! 事件总线实现

use crate::event::{EventError, EventType, StorageEvent, SubscriptionId};
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// 事件处理器类型
type EventHandler = Arc<dyn Fn(&StorageEvent) -> Result<(), EventError> + Send + Sync>;

/// 事件总线 trait
pub trait EventHub: Send + Sync {
    /// 发布事件
    fn publish(&self, event: StorageEvent) -> Result<(), EventError>;

    /// 订阅事件
    fn subscribe<F>(
        &self,
        event_type: EventType,
        handler: F,
    ) -> Result<SubscriptionId, EventError>
    where
        F: Fn(&StorageEvent) -> Result<(), EventError> + Send + Sync + 'static;

    /// 取消订阅
    fn unsubscribe(&self, subscription_id: SubscriptionId) -> Result<(), EventError>;

    /// 获取订阅数量
    fn subscription_count(&self, event_type: EventType) -> usize;
}

/// 动态事件总线类型（用于 trait object）
pub type DynEventHub = dyn EventHub;

/// 内存事件总线实现
pub struct MemoryEventHub {
    handlers: DashMap<EventType, Vec<(SubscriptionId, EventHandler)>>,
    next_subscription_id: AtomicU64,
}

impl MemoryEventHub {
    /// 创建新的事件总线
    pub fn new() -> Self {
        Self {
            handlers: DashMap::new(),
            next_subscription_id: AtomicU64::new(0),
        }
    }

    /// 获取下一个订阅 ID
    fn next_id(&self) -> SubscriptionId {
        SubscriptionId(self.next_subscription_id.fetch_add(1, Ordering::SeqCst))
    }
}

impl Default for MemoryEventHub {
    fn default() -> Self {
        Self::new()
    }
}

impl EventHub for MemoryEventHub {
    fn publish(&self, event: StorageEvent) -> Result<(), EventError> {
        let event_type = EventType::from(&event);

        if let Some(handlers) = self.handlers.get(&event_type) {
            for (id, handler) in handlers.iter() {
                if let Err(e) = handler(&event) {
                    eprintln!("[EventHub] Handler {} failed: {}", id, e);
                    // 继续执行其他 handler，不中断
                }
            }
        }

        Ok(())
    }

    fn subscribe<F>(
        &self,
        event_type: EventType,
        handler: F,
    ) -> Result<SubscriptionId, EventError>
    where
        F: Fn(&StorageEvent) -> Result<(), EventError> + Send + Sync + 'static,
    {
        let id = self.next_id();
        self.handlers
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push((id, Arc::new(handler)));
        Ok(id)
    }

    fn unsubscribe(&self, subscription_id: SubscriptionId) -> Result<(), EventError> {
        let mut removed = false;

        for mut handlers in self.handlers.iter_mut() {
            let before_len = handlers.value().len();
            handlers.value_mut().retain(|(id, _)| *id != subscription_id);
            if handlers.value().len() < before_len {
                removed = true;
            }
        }

        if removed {
            Ok(())
        } else {
            Err(EventError::SubscriptionNotFound(subscription_id.0))
        }
    }

    fn subscription_count(&self, event_type: EventType) -> usize {
        self.handlers
            .get(&event_type)
            .map(|h| h.len())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_publish_subscribe() {
        let hub = Arc::new(MemoryEventHub::new());
        let counter = Arc::new(AtomicUsize::new(0));

        let counter_clone = counter.clone();
        hub.subscribe(EventType::VertexEvent, move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .unwrap();

        let event = StorageEvent::VertexInserted {
            space_id: 1,
            vertex: create_test_vertex(),
            timestamp: 0,
        };

        hub.publish(event).unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_unsubscribe() {
        let hub = Arc::new(MemoryEventHub::new());

        let sub_id = hub
            .subscribe(EventType::VertexEvent, |_| Ok(()))
            .unwrap();

        assert_eq!(hub.subscription_count(EventType::VertexEvent), 1);

        hub.unsubscribe(sub_id).unwrap();
        assert_eq!(hub.subscription_count(EventType::VertexEvent), 0);
    }

    #[test]
    fn test_multiple_handlers() {
        let hub = Arc::new(MemoryEventHub::new());
        let counter1 = Arc::new(AtomicUsize::new(0));
        let counter2 = Arc::new(AtomicUsize::new(0));

        let c1 = counter1.clone();
        hub.subscribe(EventType::VertexEvent, move |_| {
            c1.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .unwrap();

        let c2 = counter2.clone();
        hub.subscribe(EventType::VertexEvent, move |_| {
            c2.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .unwrap();

        let event = StorageEvent::VertexInserted {
            space_id: 1,
            vertex: create_test_vertex(),
            timestamp: 0,
        };

        hub.publish(event).unwrap();
        assert_eq!(counter1.load(Ordering::SeqCst), 1);
        assert_eq!(counter2.load(Ordering::SeqCst), 1);
    }

    fn create_test_vertex() -> crate::core::Vertex {
        use crate::core::types::Tag;
        crate::core::Vertex {
            vid: crate::core::Value::Int64(1),
            tags: vec![],
        }
    }
}
