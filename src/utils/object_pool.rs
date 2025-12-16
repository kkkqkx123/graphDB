/// A simple object pool for reusing objects to reduce allocation overhead
#[derive(Debug, Clone)]
pub struct ObjectPool<T> {
    pool: Vec<T>,
    max_size: usize,
}

impl<T: Default + Clone> ObjectPool<T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            pool: Vec::new(),
            max_size,
        }
    }

    pub fn get(&mut self) -> T {
        self.pool.pop().unwrap_or_default()
    }

    pub fn put(&mut self, obj: T) {
        if self.pool.len() < self.max_size {
            self.pool.push(obj);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_pool() {
        let mut pool: ObjectPool<Vec<i32>> = ObjectPool::new(10);

        let mut obj = pool.get();
        obj.push(42);

        assert_eq!(obj, vec![42]);

        pool.put(obj);

        let obj2 = pool.get();
        assert_eq!(obj2, vec![42]); // Should reuse the same object
    }
}
