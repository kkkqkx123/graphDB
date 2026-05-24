use dashmap::DashMap;
use std::time::Instant;

use crate::sync::external_index::IndexOperation;

type IndexKey = (u64, String, String);

#[derive(Debug, Default)]
pub struct BufferEntry {
    pub inserts: Vec<IndexOperation>,
    pub deletes: Vec<String>,
}

#[derive(Debug)]
pub struct BatchBuffer {
    buffers: DashMap<IndexKey, BufferEntry>,
    last_commit: DashMap<IndexKey, Instant>,
}

impl Default for BatchBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl BatchBuffer {
    pub fn new() -> Self {
        Self {
            buffers: DashMap::new(),
            last_commit: DashMap::new(),
        }
    }

    pub fn add_insert(&self, key: &IndexKey, operation: IndexOperation) {
        let mut entry = self.buffers.entry(key.clone()).or_default();
        entry.inserts.push(operation);
        self.last_commit
            .entry(key.clone())
            .or_insert_with(Instant::now);
    }

    pub fn add_delete(&self, key: &IndexKey, id: String) {
        let mut entry = self.buffers.entry(key.clone()).or_default();
        entry.deletes.push(id);
        self.last_commit
            .entry(key.clone())
            .or_insert_with(Instant::now);
    }

    pub fn drain_inserts(&self, key: &IndexKey) -> Vec<IndexOperation> {
        if let Some(mut entry) = self.buffers.get_mut(key) {
            std::mem::take(&mut entry.inserts)
        } else {
            Vec::new()
        }
    }

    pub fn drain_deletes(&self, key: &IndexKey) -> Vec<String> {
        if let Some(mut entry) = self.buffers.get_mut(key) {
            std::mem::take(&mut entry.deletes)
        } else {
            Vec::new()
        }
    }

    pub fn drain_all(&self, key: &IndexKey) -> BufferEntry {
        self.buffers.remove(key).map(|(_, v)| v).unwrap_or_default()
    }

    pub fn count(&self, key: &IndexKey) -> usize {
        self.buffers
            .get(key)
            .map(|e| e.inserts.len() + e.deletes.len())
            .unwrap_or(0)
    }

    pub fn insert_count(&self, key: &IndexKey) -> usize {
        self.buffers.get(key).map(|e| e.inserts.len()).unwrap_or(0)
    }

    pub fn delete_count(&self, key: &IndexKey) -> usize {
        self.buffers.get(key).map(|e| e.deletes.len()).unwrap_or(0)
    }

    pub fn is_timeout(&self, key: &IndexKey, timeout: std::time::Duration) -> bool {
        self.last_commit
            .get(key)
            .map(|last| last.elapsed() >= timeout)
            .unwrap_or(false)
    }

    pub fn update_commit_time(&self, key: &IndexKey) {
        self.last_commit.insert(key.clone(), Instant::now());
    }

    pub fn keys(&self) -> Vec<IndexKey> {
        self.buffers.iter().map(|e| e.key().clone()).collect()
    }

    pub fn total_count(&self) -> usize {
        self.buffers
            .iter()
            .map(|e| e.inserts.len() + e.deletes.len())
            .sum()
    }

    pub fn clear(&self) {
        self.buffers.clear();
        self.last_commit.clear();
    }

    pub fn remove(&self, key: &IndexKey) -> Option<BufferEntry> {
        self.buffers.remove(key).map(|(_, v)| v)
    }
}
