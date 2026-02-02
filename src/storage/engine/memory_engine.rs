use super::{Engine, Operation, StorageIterator, TransactionId, SnapshotId};
use crate::storage::iterator::VecPairIterator;
use crate::core::StorageError;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

const MAX_SNAPSHOTS: usize = 10;
const MAX_SNAPSHOT_SIZE: usize = 1000;

#[derive(Clone)]
pub struct MemoryEngine {
    data: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
    snapshots: Arc<RwLock<HashMap<SnapshotId, Arc<HashMap<Vec<u8>, Vec<u8>>>>>>,
    active_transactions: Arc<RwLock<HashMap<TransactionId, TransactionData>>>,
    next_tx_id: Arc<RwLock<TransactionId>>,
    next_snapshot_id: Arc<RwLock<SnapshotId>>,
}

#[derive(Debug, Clone)]
struct TransactionData {
    writes: HashMap<Vec<u8>, Vec<u8>>,
    deletes: Vec<Vec<u8>>,
    snapshot: Arc<HashMap<Vec<u8>, Vec<u8>>>,
}

impl MemoryEngine {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            snapshots: Arc::new(RwLock::new(HashMap::new())),
            active_transactions: Arc::new(RwLock::new(HashMap::new())),
            next_tx_id: Arc::new(RwLock::new(TransactionId::new(1))),
            next_snapshot_id: Arc::new(RwLock::new(SnapshotId::new(1))),
        }
    }

    fn cleanup_old_snapshots(snapshots: &mut HashMap<SnapshotId, Arc<HashMap<Vec<u8>, Vec<u8>>>>) {
        if snapshots.len() <= MAX_SNAPSHOTS {
            return;
        }
        let to_remove: Vec<_> = snapshots.keys()
            .take(snapshots.len() - MAX_SNAPSHOTS)
            .cloned()
            .collect();
        for key in to_remove {
            snapshots.remove(&key);
        }
    }
}

impl Default for MemoryEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine for MemoryEngine {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        let data = self.data.read().map_err(|e| StorageError::DbError(e.to_string()))?;
        Ok(data.get(key).cloned())
    }

    fn put(&mut self, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        let mut data = self.data.write().map_err(|e| StorageError::DbError(e.to_string()))?;
        data.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn delete(&mut self, key: &[u8]) -> Result<(), StorageError> {
        let mut data = self.data.write().map_err(|e| StorageError::DbError(e.to_string()))?;
        data.remove(key);
        Ok(())
    }

    fn scan(&self, prefix: &[u8]) -> Result<Box<dyn StorageIterator>, StorageError> {
        let data = self.data.read().map_err(|e| StorageError::DbError(e.to_string()))?;
        let mut results: Vec<(Vec<u8>, Vec<u8>)> = data
            .iter()
            .filter(|(k, _)| k.starts_with(prefix))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        drop(data);

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
        let mut data = self.data.write().map_err(|e| StorageError::DbError(e.to_string()))?;
        for op in ops {
            match op {
                Operation::Put { key, value } => {
                    data.insert(key, value);
                }
                Operation::Delete { key } => {
                    data.remove(&key);
                }
            }
        }
        Ok(())
    }

    fn begin_transaction(&mut self) -> Result<TransactionId, StorageError> {
        let mut next_tx_id = self.next_tx_id.write().map_err(|e| StorageError::DbError(e.to_string()))?;
        let tx_id = *next_tx_id;
        *next_tx_id += 1;

        let data = self.data.read().map_err(|e| StorageError::DbError(e.to_string()))?;
        let snapshot_data: HashMap<Vec<u8>, Vec<u8>> = data.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        drop(data);
        let snapshot: Arc<HashMap<Vec<u8>, Vec<u8>>> = Arc::new(snapshot_data);

        let mut active_transactions = self.active_transactions.write().map_err(|e| StorageError::DbError(e.to_string()))?;
        active_transactions.insert(tx_id, TransactionData {
            writes: HashMap::new(),
            deletes: Vec::new(),
            snapshot,
        });

        Ok(tx_id)
    }

    fn commit_transaction(&mut self, tx_id: TransactionId) -> Result<(), StorageError> {
        let mut active_transactions = self.active_transactions.write().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(tx_data) = active_transactions.remove(&tx_id) {
            let mut data = self.data.write().map_err(|e| StorageError::DbError(e.to_string()))?;

            for key in tx_data.deletes {
                data.remove(&key);
            }

            for (key, value) in tx_data.writes {
                data.insert(key, value);
            }
        }
        Ok(())
    }

    fn rollback_transaction(&mut self, tx_id: TransactionId) -> Result<(), StorageError> {
        let mut active_transactions = self.active_transactions.write().map_err(|e| StorageError::DbError(e.to_string()))?;
        active_transactions.remove(&tx_id);
        Ok(())
    }

    fn create_snapshot(&self) -> Result<SnapshotId, StorageError> {
        let mut next_snapshot_id = self.next_snapshot_id.write().map_err(|e| StorageError::DbError(e.to_string()))?;
        let snap_id = *next_snapshot_id;
        *next_snapshot_id += 1;

        let data = self.data.read().map_err(|e| StorageError::DbError(e.to_string()))?;
        if data.len() > MAX_SNAPSHOT_SIZE {
            return Err(StorageError::DbError("Snapshot exceeds maximum size".to_string()));
        }
        let snapshot_data: HashMap<Vec<u8>, Vec<u8>> = data.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        drop(data);
        let snapshot: Arc<HashMap<Vec<u8>, Vec<u8>>> = Arc::new(snapshot_data);

        let mut snapshots = self.snapshots.write().map_err(|e| StorageError::DbError(e.to_string()))?;
        Self::cleanup_old_snapshots(&mut snapshots);
        snapshots.insert(snap_id, snapshot);

        Ok(snap_id)
    }

    fn get_snapshot(&self, snap_id: SnapshotId) -> Result<Option<Box<dyn StorageIterator>>, StorageError> {
        let snapshots = self.snapshots.read().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(snapshot_data) = snapshots.get(&snap_id) {
            let snapshot_ref: &HashMap<Vec<u8>, Vec<u8>> = &**snapshot_data;
            let mut results: Vec<(Vec<u8>, Vec<u8>)> = snapshot_ref
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
        let mut snapshots = self.snapshots.write().map_err(|e| StorageError::DbError(e.to_string()))?;
        snapshots.remove(&snap_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut engine = MemoryEngine::new();

        assert_eq!(engine.get(b"key1").unwrap(), None);

        engine.put(b"key1", b"value1").unwrap();
        assert_eq!(engine.get(b"key1").unwrap(), Some(b"value1".to_vec()));

        engine.delete(b"key1").unwrap();
        assert_eq!(engine.get(b"key1").unwrap(), None);
    }

    #[test]
    fn test_scan() {
        let mut engine = MemoryEngine::new();

        engine.put(b"a1", b"v1").unwrap();
        engine.put(b"a2", b"v2").unwrap();
        engine.put(b"b1", b"v3").unwrap();

        let iter = engine.scan(b"a").unwrap();
        let items: Vec<_> = std::iter::from_fn(|| {
            let next = iter.key().is_some();
            next.then(|| (iter.key().unwrap().to_vec(), iter.value().unwrap().to_vec()))
        }).collect();

        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_batch() {
        let mut engine = MemoryEngine::new();

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
    fn test_transaction() {
        let mut engine = MemoryEngine::new();
        engine.put(b"k1", b"v1").unwrap();

        let tx_id = engine.begin_transaction().unwrap();

        engine.put(b"tx_k1", b"tx_v1").unwrap();
        engine.delete(b"k1").unwrap();

        assert_eq!(engine.get(b"tx_k1").unwrap(), None);
        assert_eq!(engine.get(b"k1").unwrap(), Some(b"v1".to_vec()));

        engine.commit_transaction(tx_id).unwrap();

        assert_eq!(engine.get(b"tx_k1").unwrap(), Some(b"tx_v1".to_vec()));
        assert_eq!(engine.get(b"k1").unwrap(), None);
    }

    #[test]
    fn test_transaction_rollback() {
        let mut engine = MemoryEngine::new();
        engine.put(b"k1", b"v1").unwrap();

        let tx_id = engine.begin_transaction().unwrap();

        engine.put(b"tx_k1", b"tx_v1").unwrap();
        engine.delete(b"k1").unwrap();

        engine.rollback_transaction(tx_id).unwrap();

        assert_eq!(engine.get(b"tx_k1").unwrap(), None);
        assert_eq!(engine.get(b"k1").unwrap(), Some(b"v1".to_vec()));
    }

    #[test]
    fn test_snapshot() {
        let mut engine = MemoryEngine::new();
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
