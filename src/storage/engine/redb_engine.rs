use super::{Engine, Operation, StorageIterator};
use crate::storage::iterator::VecPairIterator;
use crate::core::StorageError;
use redb::{Database, ReadableTable, TableDefinition, TypeName};
use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ByteKey(pub Vec<u8>);

impl redb::Key for ByteKey {
    fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
        data1.cmp(data2)
    }
}

impl redb::Value for ByteKey {
    type SelfType<'a> = ByteKey where Self: 'a;
    type AsBytes<'a> = Vec<u8> where Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> ByteKey where Self: 'a {
        ByteKey(data.to_vec())
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Vec<u8> where Self: 'b {
        value.0.clone()
    }

    fn type_name() -> TypeName {
        TypeName::new("graphdb::ByteKey")
    }
}

const DATA_TABLE: TableDefinition<ByteKey, ByteKey> = TableDefinition::new("data");

pub struct RedbEngine {
    db: Database,
    db_path: String,
}

impl RedbEngine {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, StorageError> {
        let db_path = path.as_ref().to_string_lossy().to_string();

        let db = Database::create(path.as_ref())
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let write_txn = db
            .begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            write_txn
                .open_table(DATA_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(Self {
            db,
            db_path,
        })
    }
}

impl Clone for RedbEngine {
    fn clone(&self) -> Self {
        Self::new(&self.db_path).expect("Failed to clone RedbEngine")
    }
}

impl Engine for RedbEngine {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn
            .open_table(DATA_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        match table
            .get(ByteKey(key.to_vec()))
            .map_err(|e| StorageError::DbError(e.to_string()))?
        {
            Some(value) => Ok(Some(value.value().0.clone())),
            None => Ok(None),
        }
    }

    fn put(&mut self, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(DATA_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            table
                .insert(ByteKey(key.to_vec()), ByteKey(value.to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        Ok(())
    }

    fn delete(&mut self, key: &[u8]) -> Result<(), StorageError> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(DATA_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            table
                .remove(ByteKey(key.to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        Ok(())
    }

    fn scan(&self, prefix: &[u8]) -> Result<Box<dyn StorageIterator>, StorageError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn
            .open_table(DATA_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let mut results: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
        let iter = table
            .iter()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        for item in iter {
            let (key, value) = item.map_err(|e| StorageError::DbError(e.to_string()))?;
            let key_bytes = key.value().0;
            if key_bytes.starts_with(prefix) {
                results.push((key_bytes, value.value().0));
            }
        }

        results.sort_by(|a, b| a.0.cmp(&b.0));

        let mut keys = Vec::with_capacity(results.len());
        let mut values = Vec::with_capacity(results.len());
        for (k, v) in results {
            keys.push(k);
            values.push(v);
        }

        Ok(Box::new(VecPairIterator::new(keys, values)))
    }

    fn batch(&mut self, ops: Vec<Operation>) -> Result<(), StorageError> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(DATA_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            for op in ops {
                match op {
                    Operation::Put { key, value } => {
                        table
                            .insert(ByteKey(key), ByteKey(value))
                            .map_err(|e| StorageError::DbError(e.to_string()))?;
                    }
                    Operation::Delete { key } => {
                        table
                            .remove(ByteKey(key))
                            .map_err(|e| StorageError::DbError(e.to_string()))?;
                    }
                }
            }
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        Ok(())
    }

    fn count_keys(&self, prefix: &[u8]) -> Result<usize, StorageError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn
            .open_table(DATA_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let mut count = 0;
        let iter = table
            .iter()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        for item in iter {
            let (key, _) = item.map_err(|e| StorageError::DbError(e.to_string()))?;
            let key_bytes = key.value().0;
            if key_bytes.starts_with(prefix) {
                count += 1;
            }
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_basic_operations() {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let mut engine = RedbEngine::new(temp_dir.path().join("test.db")).expect("Failed to create RedbEngine");

        assert_eq!(engine.get(b"key1").expect("Failed to get key"), None);

        engine.put(b"key1", b"value1").expect("Failed to put key-value pair");
        assert_eq!(engine.get(b"key1").expect("Failed to get key"), Some(b"value1".to_vec()));

        engine.delete(b"key1").expect("Failed to delete key");
        assert_eq!(engine.get(b"key1").expect("Failed to get key"), None);
    }

    #[test]
    fn test_scan() {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let mut engine = RedbEngine::new(temp_dir.path().join("test.db")).expect("Failed to create RedbEngine");

        engine.put(b"a1", b"v1").expect("Failed to put key-value pair");
        engine.put(b"a2", b"v2").expect("Failed to put key-value pair");
        engine.put(b"b1", b"v3").expect("Failed to put key-value pair");

        let iter = engine.scan(b"a").expect("Failed to create scan iterator");
        let mut items = Vec::new();
        let mut iter = iter;
        while iter.next() {
            if let (Some(k), Some(v)) = (iter.key(), iter.value()) {
                items.push((k.to_vec(), v.to_vec()));
            }
        }

        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_batch() {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let mut engine = RedbEngine::new(temp_dir.path().join("test.db")).expect("Failed to create RedbEngine");

        let ops = vec![
            Operation::Put { key: b"k1".to_vec(), value: b"v1".to_vec() },
            Operation::Put { key: b"k2".to_vec(), value: b"v2".to_vec() },
            Operation::Delete { key: b"k3".to_vec() },
        ];

        engine.batch(ops).expect("Failed to execute batch operations");

        assert_eq!(engine.get(b"k1").expect("Failed to get key"), Some(b"v1".to_vec()));
        assert_eq!(engine.get(b"k2").expect("Failed to get key"), Some(b"v2".to_vec()));
        assert_eq!(engine.get(b"k3").expect("Failed to get key"), None);
    }
}
