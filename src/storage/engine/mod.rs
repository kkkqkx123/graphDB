use crate::core::StorageError;

pub trait Engine: Send + Sync {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError>;
    fn put(&mut self, key: &[u8], value: &[u8]) -> Result<(), StorageError>;
    fn delete(&mut self, key: &[u8]) -> Result<(), StorageError>;
    fn scan(&self, prefix: &[u8]) -> Result<Box<dyn StorageIterator>, StorageError>;
    fn batch(&mut self, ops: Vec<Operation>) -> Result<(), StorageError>;
}

pub enum Operation {
    Put { key: Vec<u8>, value: Vec<u8> },
    Delete { key: Vec<u8> },
}

pub trait StorageIterator: Send + {
    fn key(&self) -> Option<&[u8]>;
    fn value(&self) -> Option<&[u8]>;
    fn next(&mut self) -> bool;
    fn estimate_remaining(&self) -> Option<usize>;
}

pub struct EmptyIterator;

impl StorageIterator for EmptyIterator {
    fn key(&self) -> Option<&[u8]> {
        None
    }

    fn value(&self) -> Option<&[u8]> {
        None
    }

    fn next(&mut self) -> bool {
        false
    }

    fn estimate_remaining(&self) -> Option<usize> {
        Some(0)
    }
}

pub struct VecIterator<T> {
    data: Vec<T>,
    index: usize,
}

impl<T> VecIterator<T> {
    pub fn new(data: Vec<T>) -> Self {
        Self { data, index: 0 }
    }
}

impl<T> StorageIterator for VecIterator<T>
where
    T: AsRef<[u8]> + Send,
{
    fn key(&self) -> Option<&[u8]> {
        self.data.get(self.index).map(|v| v.as_ref())
    }

    fn value(&self) -> Option<&[u8]> {
        self.data.get(self.index).map(|v| v.as_ref())
    }

    fn next(&mut self) -> bool {
        if self.index < self.data.len() {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn estimate_remaining(&self) -> Option<usize> {
        Some(self.data.len().saturating_sub(self.index))
    }
}
