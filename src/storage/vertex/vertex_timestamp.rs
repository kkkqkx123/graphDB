//! Vertex Timestamp
//!
//! MVCC timestamp tracking for vertices.
//! Tracks creation and deletion timestamps for each vertex.

use super::{Timestamp, INVALID_TIMESTAMP, MAX_TIMESTAMP, VertexStatus};

#[derive(Debug, Clone)]
pub struct VertexTimestamp {
    start_ts: Vec<Timestamp>,
    end_ts: Vec<Timestamp>,
    deleted: Vec<bool>,
    capacity: usize,
}

impl VertexTimestamp {
    pub fn new() -> Self {
        Self::with_capacity(1024)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            start_ts: Vec::with_capacity(capacity),
            end_ts: Vec::with_capacity(capacity),
            deleted: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn insert(&mut self, index: u32, ts: Timestamp) {
        let idx = index as usize;
        if idx >= self.start_ts.len() {
            self.start_ts.resize(idx + 1, INVALID_TIMESTAMP);
            self.end_ts.resize(idx + 1, INVALID_TIMESTAMP);
            self.deleted.resize(idx + 1, false);
        }
        self.start_ts[idx] = ts;
        self.end_ts[idx] = MAX_TIMESTAMP;
        self.deleted[idx] = false;
    }

    pub fn remove(&mut self, index: u32, ts: Timestamp) {
        let idx = index as usize;
        if idx < self.end_ts.len() {
            self.end_ts[idx] = ts;
            self.deleted[idx] = true;
        }
    }

    pub fn revert_remove(&mut self, index: u32, ts: Timestamp) {
        let idx = index as usize;
        if idx < self.end_ts.len() && self.deleted[idx] {
            self.end_ts[idx] = MAX_TIMESTAMP;
            self.deleted[idx] = false;
        }
    }

    pub fn is_valid(&self, index: u32, ts: Timestamp) -> bool {
        let idx = index as usize;
        if idx >= self.start_ts.len() {
            return false;
        }

        let start = self.start_ts[idx];
        let end = self.end_ts[idx];

        if start == INVALID_TIMESTAMP {
            return false;
        }

        start <= ts && end > ts
    }

    pub fn is_deleted(&self, index: u32) -> bool {
        let idx = index as usize;
        idx < self.deleted.len() && self.deleted[idx]
    }

    pub fn get_status(&self, index: u32) -> VertexStatus {
        if self.is_deleted(index) {
            VertexStatus::Deleted
        } else {
            VertexStatus::Active
        }
    }

    pub fn get_start_ts(&self, index: u32) -> Option<Timestamp> {
        let idx = index as usize;
        if idx < self.start_ts.len() {
            let ts = self.start_ts[idx];
            if ts != INVALID_TIMESTAMP {
                return Some(ts);
            }
        }
        None
    }

    pub fn get_end_ts(&self, index: u32) -> Option<Timestamp> {
        let idx = index as usize;
        if idx < self.end_ts.len() {
            let ts = self.end_ts[idx];
            if ts != MAX_TIMESTAMP {
                return Some(ts);
            }
        }
        None
    }

    pub fn valid_count(&self, ts: Timestamp) -> usize {
        self.start_ts
            .iter()
            .enumerate()
            .filter(|(i, &start)| {
                start != INVALID_TIMESTAMP
                    && start <= ts
                    && self.end_ts[*i] > ts
            })
            .count()
    }

    pub fn size(&self) -> usize {
        self.start_ts.len()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn reserve(&mut self, new_capacity: usize) {
        if new_capacity > self.capacity {
            self.capacity = new_capacity;
            self.start_ts.reserve(new_capacity);
            self.end_ts.reserve(new_capacity);
        }
    }

    pub fn resize(&mut self, new_size: usize) {
        self.start_ts.resize(new_size, INVALID_TIMESTAMP);
        self.end_ts.resize(new_size, MAX_TIMESTAMP);
        self.deleted.resize(new_size, false);
    }

    pub fn clear(&mut self) {
        self.start_ts.clear();
        self.end_ts.clear();
        self.deleted.clear();
    }

    pub fn compact(&mut self) {
        let mut write_idx = 0;
        for read_idx in 0..self.start_ts.len() {
            if !self.deleted[read_idx] {
                if write_idx != read_idx {
                    self.start_ts[write_idx] = self.start_ts[read_idx];
                    self.end_ts[write_idx] = self.end_ts[read_idx];
                    self.deleted[write_idx] = self.deleted[read_idx];
                }
                write_idx += 1;
            }
        }
        self.start_ts.truncate(write_idx);
        self.end_ts.truncate(write_idx);
        self.deleted.truncate(write_idx);
    }

    pub fn dump(&self) -> Vec<Timestamp> {
        let mut result = Vec::with_capacity(self.start_ts.len() * 2 + self.deleted.len());
        for i in 0..self.start_ts.len() {
            result.push(self.start_ts[i]);
            result.push(self.end_ts[i]);
            result.push(if self.deleted[i] { 1u32 } else { 0u32 });
        }
        result
    }

    pub fn load(&mut self, data: &[Timestamp]) {
        self.clear();
        let count = data.len() / 3;
        self.start_ts.reserve(count);
        self.end_ts.reserve(count);
        self.deleted.reserve(count);

        for i in 0..count {
            self.start_ts.push(data[i * 3]);
            self.end_ts.push(data[i * 3 + 1]);
            self.deleted.push(data[i * 3 + 2] == 1);
        }
    }
}

impl Default for VertexTimestamp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_validity() {
        let mut vts = VertexTimestamp::new();

        vts.insert(0, 100);
        vts.insert(1, 101);
        vts.insert(2, 102);

        assert!(vts.is_valid(0, 100));
        assert!(vts.is_valid(0, 200));
        assert!(!vts.is_valid(0, 50));

        assert!(vts.is_valid(1, 101));
        assert!(vts.is_valid(2, 102));
    }

    #[test]
    fn test_delete_and_revert() {
        let mut vts = VertexTimestamp::new();

        vts.insert(0, 100);
        vts.remove(0, 200);

        assert!(vts.is_deleted(0));
        assert!(vts.is_valid(0, 150));
        assert!(!vts.is_valid(0, 250));

        vts.revert_remove(0, 100);
        assert!(!vts.is_deleted(0));
        assert!(vts.is_valid(0, 250));
    }

    #[test]
    fn test_valid_count() {
        let mut vts = VertexTimestamp::new();

        vts.insert(0, 100);
        vts.insert(1, 101);
        vts.insert(2, 102);
        vts.remove(1, 200);

        assert_eq!(vts.valid_count(150), 3);
        assert_eq!(vts.valid_count(250), 2);
    }
}
