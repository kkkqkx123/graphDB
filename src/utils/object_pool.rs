//! 对象池模块
//!
//! 提供统一的对象池实现，支持以下使用场景：
//! 1. 需要 Default 的轻量级场景（优化器节点）
//! 2. 需要工厂函数的通用场景（查询上下文）
//! 3. 需要线程安全的并发场景
//!
//! 参考 nebula-graph 的 ObjectPool 设计，采用工厂模式 + 线程安全设计

use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::fmt::Debug;

/// 统一的轻量级对象池
///
/// 适用于单线程场景，要求对象实现 Default trait
/// 使用 VecDeque 实现高效的 push/pop
#[derive(Debug, Clone)]
pub struct ObjectPool<T: Default> {
    pool: VecDeque<T>,
    max_size: usize,
}

impl<T: Default> ObjectPool<T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            pool: VecDeque::new(),
            max_size,
        }
    }

    pub fn with_capacity(capacity: usize, max_size: usize) -> Self {
        Self {
            pool: VecDeque::with_capacity(capacity),
            max_size,
        }
    }

    pub fn acquire(&mut self) -> T {
        self.pool.pop_front().unwrap_or_default()
    }

    pub fn release(&mut self, obj: T) {
        if self.pool.len() < self.max_size {
            self.pool.push_back(obj);
        }
    }

    pub fn size(&self) -> usize {
        self.pool.len()
    }

    pub fn capacity(&self) -> usize {
        self.max_size
    }

    pub fn is_empty(&self) -> bool {
        self.pool.is_empty()
    }

    pub fn clear(&mut self) {
        self.pool.clear();
    }
}

impl<T: Default> Default for ObjectPool<T> {
    fn default() -> Self {
        Self::new(1000)
    }
}

/// 线程安全的对象池
///
/// 使用 Arc<RwLock> 实现线程安全
/// 通过工厂函数创建对象，支持任意类型
pub struct ThreadSafeObjectPool<T: Clone + Send + 'static> {
    pool: Arc<RwLock<Vec<T>>>,
    factory: Arc<dyn Fn() -> T + Send + Sync>,
    max_size: usize,
}

impl<T: Clone + Send + 'static> ThreadSafeObjectPool<T> {
    pub fn new<F>(factory: F, max_size: usize) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            pool: Arc::new(RwLock::new(Vec::with_capacity(max_size))),
            factory: Arc::new(factory),
            max_size,
        }
    }

    pub fn acquire(&self) -> T {
        if let Ok(mut pool) = self.pool.write() {
            if let Some(obj) = pool.pop() {
                return obj;
            }
        }
        (self.factory)()
    }

    pub fn release(&self, obj: T) {
        if let Ok(mut pool) = self.pool.write() {
            if pool.len() < self.max_size {
                pool.push(obj);
            }
        }
    }

    pub fn size(&self) -> usize {
        self.pool.read().map(|p| p.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    pub fn clear(&self) {
        if let Ok(mut pool) = self.pool.write() {
            pool.clear();
        }
    }
}

impl<T: Clone + Send + 'static> Clone for ThreadSafeObjectPool<T> {
    fn clone(&self) -> Self {
        Self {
            pool: Arc::clone(&self.pool),
            factory: Arc::clone(&self.factory),
            max_size: self.max_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_pool_basic() {
        let mut pool: ObjectPool<Vec<i32>> = ObjectPool::new(10);

        let mut obj = pool.acquire();
        obj.push(42);
        assert_eq!(obj, vec![42]);

        pool.release(obj);

        let obj2 = pool.acquire();
        assert_eq!(obj2, vec![42]);
        assert_eq!(pool.size(), 0);
    }

    #[test]
    fn test_object_pool_max_size() {
        let mut pool: ObjectPool<String> = ObjectPool::new(2);

        pool.release("1".to_string());
        pool.release("2".to_string());
        pool.release("3".to_string()); // 超过 max_size，应被丢弃

        assert_eq!(pool.size(), 2);
    }

    #[test]
    fn test_thread_safe_object_pool() {
        let pool: ThreadSafeObjectPool<String> =
            ThreadSafeObjectPool::new(|| "default".to_string(), 10);

        let obj1 = pool.acquire();
        assert_eq!(obj1, "default");

        pool.release(obj1);
        assert_eq!(pool.size(), 1);

        let obj2 = pool.acquire();
        assert_eq!(obj2, "default");
        assert_eq!(pool.size(), 0);
    }

    #[test]
    fn test_thread_safe_clone() {
        let pool: ThreadSafeObjectPool<i32> =
            ThreadSafeObjectPool::new(|| 42, 10);

        let obj = pool.acquire();
        assert_eq!(obj, 42);

        pool.release(obj);

        let cloned_pool = pool.clone();
        let obj2 = cloned_pool.acquire();
        assert_eq!(obj2, 42);
    }
}
