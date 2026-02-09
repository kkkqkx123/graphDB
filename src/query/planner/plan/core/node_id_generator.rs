//! 节点ID生成器
//!
//! 提供全局唯一的计划节点ID分配机制

use std::sync::atomic::{AtomicI64, Ordering};

/// 节点ID生成器
///
/// 使用单例模式提供全局唯一的节点ID分配
pub struct NodeIdGenerator {
    counter: AtomicI64,
}

impl NodeIdGenerator {
    /// 获取全局单例实例
    pub fn instance() -> &'static Self {
        static INSTANCE: NodeIdGenerator = NodeIdGenerator {
            counter: AtomicI64::new(1), // 从1开始，0保留为无效ID
        };
        &INSTANCE
    }

    /// 获取下一个唯一ID
    pub fn next_id(&self) -> i64 {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }

    /// 重置计数器（仅用于测试）
    #[cfg(test)]
    pub fn reset(&self) {
        self.counter.store(1, Ordering::SeqCst);
    }
}

/// 为节点分配新ID的便捷函数
pub fn next_node_id() -> i64 {
    NodeIdGenerator::instance().next_id()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_generation() {
        NodeIdGenerator::instance().reset();
        
        let id1 = next_node_id();
        let id2 = next_node_id();
        let id3 = next_node_id();
        
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_singleton() {
        NodeIdGenerator::instance().reset();
        
        let id1 = NodeIdGenerator::instance().next_id();
        let id2 = NodeIdGenerator::instance().next_id();
        
        assert_eq!(id2, id1 + 1);
    }
}
