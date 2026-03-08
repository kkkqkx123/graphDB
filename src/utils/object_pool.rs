//! 对象池模块
//!
//! 提供轻量级的对象池实现，用于对象的复用和缓存

use std::collections::VecDeque;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct ObjectPool<T> {
    pool: VecDeque<T>,
    max_size: usize,
}

impl<T> ObjectPool<T> {
    pub fn new(max_size: usize) -> Self
    where
        T: Default,
    {
        Self {
            pool: VecDeque::new(),
            max_size,
        }
    }

    pub fn acquire(&mut self) -> T
    where
        T: Default,
    {
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

    pub fn clear(&mut self) {
        self.pool.clear();
    }
}

impl<T: Default> Default for ObjectPool<T> {
    fn default() -> Self {
        Self::new(1000)
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
        pool.release("3".to_string());

        assert_eq!(pool.size(), 2);
    }

    #[test]
    fn test_object_pool_clear() {
        let mut pool: ObjectPool<String> = ObjectPool::new(10);

        pool.release("test1".to_string());
        pool.release("test2".to_string());

        assert_eq!(pool.size(), 2);

        pool.clear();
        assert_eq!(pool.size(), 0);
    }
}
