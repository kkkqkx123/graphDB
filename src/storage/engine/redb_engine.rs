use super::{Engine, Operation, StorageIterator, TransactionId, SnapshotId};
use crate::storage::iterator::VecPairIterator;
use crate::core::StorageError;
use redb::{Database, ReadableTable, TableDefinition, TypeName};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

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
const SNAPSHOTS_TABLE: TableDefinition<ByteKey, ByteKey> = TableDefinition::new("snapshots");

pub struct RedbEngine {
    db: Database,
    db_path: String,
    snapshots: Arc<Mutex<HashMap<SnapshotId, HashMap<Vec<u8>, Vec<u8>>>>>,
    next_snapshot_id: Arc<Mutex<SnapshotId>>,
}

impl RedbEngine {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, StorageError> {
        let db_path = path.as_ref().to_string_lossy().to_string();

        let db = Database::create(path.as_ref())
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(Self {
            db,
            db_path,
            snapshots: Arc::new(Mutex::new(HashMap::new())),
            next_snapshot_id: Arc::new(Mutex::new(SnapshotId::new(1))),
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

    fn begin_transaction(&mut self) -> Result<TransactionId, StorageError> {
        let tx_id = TransactionId::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_nanos() as u64,
        );
        Ok(tx_id)
    }

    fn commit_transaction(&mut self, _tx_id: TransactionId) -> Result<(), StorageError> {
        Ok(())
    }

    fn rollback_transaction(&mut self, _tx_id: TransactionId) -> Result<(), StorageError> {
        Ok(())
    }

    fn create_snapshot(&self) -> Result<SnapshotId, StorageError> {
        let mut next_snapshot_id = self.next_snapshot_id.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let snap_id = *next_snapshot_id;
        *next_snapshot_id += 1;

        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn
            .open_table(DATA_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let mut snapshot_data: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
        let iter = table
            .iter()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        for item in iter {
            let (key, value) = item.map_err(|e| StorageError::DbError(e.to_string()))?;
            snapshot_data.insert(key.value().0, value.value().0);
        }

        let mut snapshots = self.snapshots.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        snapshots.insert(snap_id, snapshot_data);

        Ok(snap_id)
    }

    fn get_snapshot(&self, snap_id: SnapshotId) -> Result<Option<Box<dyn StorageIterator>>, StorageError> {
        let snapshots = self.snapshots.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(snapshot_data) = snapshots.get(&snap_id) {
            let mut results: Vec<(Vec<u8>, Vec<u8>)> = snapshot_data
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            drop(snapshots);

            results.sort_by(|a, b| a.0.cmp(&b.0));

            let mut keys = Vec::with_capacity(results.len());
            let mut values = Vec::with_capacity(results.len());
            for (k, v) in results {
                keys.push(k);
                values.push(v);
            }

            Ok(Some(Box::new(VecPairIterator::new(keys, values))))
        } else {
            Ok(None)
        }
    }

    fn delete_snapshot(&self, snap_id: SnapshotId) -> Result<(), StorageError> {
        let mut snapshots = self.snapshots.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        snapshots.remove(&snap_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let mut engine = RedbEngine::new(temp_dir.path().join("test.db")).unwrap();

        assert_eq!(engine.get(b"key1").unwrap(), None);

        engine.put(b"key1", b"value1").unwrap();
        assert_eq!(engine.get(b"key1").unwrap(), Some(b"value1".to_vec()));

        engine.delete(b"key1").unwrap();
        assert_eq!(engine.get(b"key1").unwrap(), None);
    }

    #[test]
    fn test_scan() {
        let temp_dir = TempDir::new().unwrap();
        let mut engine = RedbEngine::new(temp_dir.path().join("test.db")).unwrap();

        engine.put(b"a1", b"v1").unwrap();
        engine.put(b"a2", b"v2").unwrap();
        engine.put(b"b1", b"v3").unwrap();

        let iter = engine.scan(b"a").unwrap();
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
        let temp_dir = TempDir::new().unwrap();
        let mut engine = RedbEngine::new(temp_dir.path().join("test.db")).unwrap();

        let ops = vec![
            Operation::Put { key: b"k1".to_vec(), value: b"v1".to_vec() },
            Operation::Put { key: b"k2".to_vec(), value: b"v2".to_vec() },
            Operation::Delete { key: b"k3".to_vec() },
        ];

        engine.batch(ops).unwrap();

        assert_eq!(engine.get(b"k1").unwrap(), Some(b"v1".to_vec()));
        assert_eq!(engine.get(b"k2").unwrap(), Some(b"v2".to_vec()));
        assert_eq!(engine.get(b"k3").unwrap(), None);
    }

    #[test]
    fn test_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        let mut engine = RedbEngine::new(temp_dir.path().join("test.db")).unwrap();
        engine.put(b"k1", b"v1").unwrap();

        let snap_id = engine.create_snapshot().unwrap();

        engine.put(b"k2", b"v2").unwrap();
        engine.delete(b"k1").unwrap();

        assert_eq!(engine.get(b"k1").unwrap(), None);
        assert_eq!(engine.get(b"k2").unwrap(), Some(b"v2".to_vec()));

        let snap_iter = engine.get_snapshot(snap_id).unwrap().unwrap();
        let mut snapshot_items = Vec::new();
        let mut iter = snap_iter;
        while iter.next() {
            if let (Some(k), Some(v)) = (iter.key(), iter.value()) {
                snapshot_items.push((k.to_vec(), v.to_vec()));
            }
        }

        assert_eq!(snapshot_items.len(), 1);
        assert_eq!(snapshot_items[0], (b"k1".to_vec(), b"v1".to_vec()));

        engine.delete_snapshot(snap_id).unwrap();
        assert!(engine.get_snapshot(snap_id).unwrap().is_none());
    }
}
