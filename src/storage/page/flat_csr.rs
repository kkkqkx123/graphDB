//! Flat CSR Implementation
//!
//! A flat CSR (Compressed Sparse Row) structure using contiguous arrays
//! for improved cache locality and memory efficiency.

use std::sync::atomic::{AtomicU64, Ordering};

use super::{EdgeRecord, DELETED_TIMESTAMP, EDGE_RECORD_SIZE, INVALID_TIMESTAMP};

pub type Timestamp = u32;

pub type VertexId = u64;
pub type EdgeId = u64;

const DEFAULT_VERTEX_CAPACITY: usize = 1024;
const DEFAULT_EDGE_CAPACITY: usize = 4096;

#[derive(Debug)]
pub struct FlatCsr {
    offsets: Vec<usize>,
    degrees: Vec<usize>,
    edges: Vec<EdgeRecord>,
    edge_count: AtomicU64,
    vertex_capacity: usize,
}

impl FlatCsr {
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_VERTEX_CAPACITY, DEFAULT_EDGE_CAPACITY)
    }

    pub fn with_capacity(vertex_capacity: usize, edge_capacity: usize) -> Self {
        let capacity = vertex_capacity.max(1);
        Self {
            offsets: vec![0; capacity + 1],
            degrees: vec![0; capacity],
            edges: Vec::with_capacity(edge_capacity),
            edge_count: AtomicU64::new(0),
            vertex_capacity: capacity,
        }
    }

    pub fn resize(&mut self, new_vertex_capacity: usize) {
        if new_vertex_capacity <= self.vertex_capacity {
            return;
        }

        let last_offset = *self.offsets.last().unwrap_or(&0);
        self.offsets.resize(new_vertex_capacity + 1, last_offset);
        self.degrees.resize(new_vertex_capacity, 0);
        self.vertex_capacity = new_vertex_capacity;
    }

    pub fn insert(&mut self, src: VertexId, edge: EdgeRecord) -> bool {
        let src_idx = src as usize;
        if src_idx >= self.vertex_capacity {
            let new_capacity = (src_idx + 1).next_power_of_two();
            self.resize(new_capacity);
        }

        let offset = self.offsets[src_idx];
        let degree = self.degrees[src_idx];

        for i in offset..offset + degree {
            if i < self.edges.len() {
                let existing = &self.edges[i];
                if existing.dst_id == edge.dst_id && !existing.is_deleted() {
                    return false;
                }
            }
        }

        let insert_pos = offset + degree;

        if insert_pos >= self.edges.len() {
            self.edges.push(edge);
        } else {
            self.edges.insert(insert_pos, edge);
        }

        self.degrees[src_idx] += 1;

        for i in (src_idx + 1)..=self.vertex_capacity {
            self.offsets[i] += 1;
        }

        self.edge_count.fetch_add(1, Ordering::Relaxed);
        true
    }

    pub fn delete(&mut self, src: VertexId, dst: VertexId, ts: Timestamp) -> bool {
        let src_idx = src as usize;
        if src_idx >= self.vertex_capacity {
            return false;
        }

        let offset = self.offsets[src_idx];
        let degree = self.degrees[src_idx];

        for i in offset..offset + degree {
            if i < self.edges.len() {
                let edge = &mut self.edges[i];
                if edge.dst_id == dst && !edge.is_deleted() && edge.timestamp <= ts {
                    edge.mark_deleted();
                    self.edge_count.fetch_sub(1, Ordering::Relaxed);
                    return true;
                }
            }
        }

        false
    }

    pub fn delete_by_edge_id(&mut self, edge_id: EdgeId, ts: Timestamp) -> bool {
        for edge in &mut self.edges {
            if edge.edge_id == edge_id && !edge.is_deleted() && edge.timestamp <= ts {
                edge.mark_deleted();
                self.edge_count.fetch_sub(1, Ordering::Relaxed);
                return true;
            }
        }
        false
    }

    pub fn revert_delete(&mut self, src: VertexId, dst: VertexId, ts: Timestamp) -> bool {
        let src_idx = src as usize;
        if src_idx >= self.vertex_capacity {
            return false;
        }

        let offset = self.offsets[src_idx];
        let degree = self.degrees[src_idx];

        for i in offset..offset + degree {
            if i < self.edges.len() {
                let edge = &mut self.edges[i];
                if edge.dst_id == dst && edge.is_deleted() {
                    edge.restore(ts);
                    self.edge_count.fetch_add(1, Ordering::Relaxed);
                    return true;
                }
            }
        }

        false
    }

    pub fn get_edge(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> Option<&EdgeRecord> {
        let src_idx = src as usize;
        if src_idx >= self.vertex_capacity {
            return None;
        }

        let offset = self.offsets[src_idx];
        let degree = self.degrees[src_idx];

        for i in offset..offset + degree {
            if i < self.edges.len() {
                let edge = &self.edges[i];
                if edge.dst_id == dst && edge.is_valid(ts) {
                    return Some(edge);
                }
            }
        }

        None
    }

    pub fn iter_edges(&self, src: VertexId, ts: Timestamp) -> FlatCsrEdgeIterator {
        let src_idx = src as usize;
        let (start, end) = if src_idx < self.vertex_capacity {
            let offset = self.offsets[src_idx];
            let degree = self.degrees[src_idx];
            (offset, offset + degree)
        } else {
            (0, 0)
        };

        FlatCsrEdgeIterator {
            edges: &self.edges,
            start,
            end,
            current: start,
            ts,
        }
    }

    pub fn degree(&self, src: VertexId, ts: Timestamp) -> usize {
        let src_idx = src as usize;
        if src_idx >= self.vertex_capacity {
            return 0;
        }

        let offset = self.offsets[src_idx];
        let degree = self.degrees[src_idx];

        let mut count = 0;
        for i in offset..offset + degree {
            if i < self.edges.len() && self.edges[i].is_valid(ts) {
                count += 1;
            }
        }
        count
    }

    pub fn has_edge(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> bool {
        self.get_edge(src, dst, ts).is_some()
    }

    pub fn edge_count(&self) -> u64 {
        self.edge_count.load(Ordering::Relaxed)
    }

    pub fn vertex_capacity(&self) -> usize {
        self.vertex_capacity
    }

    pub fn is_empty(&self) -> bool {
        self.edge_count.load(Ordering::Relaxed) == 0
    }

    pub fn clear(&mut self) {
        self.offsets.fill(0);
        self.degrees.fill(0);
        self.edges.clear();
        self.edge_count.store(0, Ordering::Relaxed);
    }

    pub fn compact(&mut self) {
        let mut new_edges = Vec::with_capacity(self.edges.len());
        let mut new_offsets = vec![0usize; self.vertex_capacity + 1];
        let mut new_degrees = vec![0usize; self.vertex_capacity];

        let mut current_offset = 0usize;

        for src in 0..self.vertex_capacity {
            new_offsets[src] = current_offset;

            let offset = self.offsets[src];
            let degree = self.degrees[src];

            for i in offset..offset + degree {
                if i < self.edges.len() {
                    let edge = &self.edges[i];
                    if !edge.is_deleted() {
                        new_edges.push(edge.clone());
                        new_degrees[src] += 1;
                        current_offset += 1;
                    }
                }
            }
        }

        new_offsets[self.vertex_capacity] = current_offset;

        self.edges = new_edges;
        self.offsets = new_offsets;
        self.degrees = new_degrees;
        self.edge_count.store(current_offset as u64, Ordering::Relaxed);
    }

    pub fn dump(&self) -> Vec<u8> {
        let mut result = Vec::new();

        result.extend_from_slice(&(self.vertex_capacity as u64).to_le_bytes());
        result.extend_from_slice(&self.edge_count.load(Ordering::Relaxed).to_le_bytes());

        result.extend_from_slice(&(self.offsets.len() as u64).to_le_bytes());
        for &offset in &self.offsets {
            result.extend_from_slice(&(offset as u64).to_le_bytes());
        }

        result.extend_from_slice(&(self.degrees.len() as u64).to_le_bytes());
        for &degree in &self.degrees {
            result.extend_from_slice(&(degree as u64).to_le_bytes());
        }

        result.extend_from_slice(&(self.edges.len() as u64).to_le_bytes());
        for edge in &self.edges {
            result.extend_from_slice(&edge.to_bytes());
        }

        result
    }

    pub fn load(&mut self, data: &[u8]) {
        if data.len() < 24 {
            return;
        }

        let mut offset = 0;

        let vertex_capacity = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8])) as usize;
        offset += 8;

        let edge_count = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8]));
        offset += 8;

        self.vertex_capacity = vertex_capacity;

        let offsets_len = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8])) as usize;
        offset += 8;

        self.offsets.clear();
        for _ in 0..offsets_len {
            if offset + 8 > data.len() {
                break;
            }
            let val = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8])) as usize;
            self.offsets.push(val);
            offset += 8;
        }

        if offset + 8 > data.len() {
            return;
        }
        let degrees_len = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8])) as usize;
        offset += 8;

        self.degrees.clear();
        for _ in 0..degrees_len {
            if offset + 8 > data.len() {
                break;
            }
            let val = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8])) as usize;
            self.degrees.push(val);
            offset += 8;
        }

        if offset + 8 > data.len() {
            return;
        }
        let edges_len = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8])) as usize;
        offset += 8;

        self.edges.clear();
        for _ in 0..edges_len {
            if offset + EDGE_RECORD_SIZE > data.len() {
                break;
            }
            if let Some(edge) = EdgeRecord::from_bytes(&data[offset..offset + EDGE_RECORD_SIZE]) {
                self.edges.push(edge);
            }
            offset += EDGE_RECORD_SIZE;
        }

        self.edge_count.store(edge_count, Ordering::Relaxed);
    }

    pub fn iter(&self, ts: Timestamp) -> FlatCsrIterator {
        FlatCsrIterator::new(self, ts)
    }
}

impl Default for FlatCsr {
    fn default() -> Self {
        Self::new()
    }
}

pub struct FlatCsrEdgeIterator<'a> {
    edges: &'a [EdgeRecord],
    start: usize,
    end: usize,
    current: usize,
    ts: Timestamp,
}

impl<'a> Iterator for FlatCsrEdgeIterator<'a> {
    type Item = &'a EdgeRecord;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current < self.end {
            let edge = &self.edges[self.current];
            self.current += 1;

            if edge.is_valid(self.ts) {
                return Some(edge);
            }
        }
        None
    }
}

pub struct FlatCsrIterator<'a> {
    csr: &'a FlatCsr,
    ts: Timestamp,
    current_vertex: usize,
    current_edge: usize,
}

impl<'a> FlatCsrIterator<'a> {
    pub fn new(csr: &'a FlatCsr, ts: Timestamp) -> Self {
        Self {
            csr,
            ts,
            current_vertex: 0,
            current_edge: 0,
        }
    }
}

impl<'a> Iterator for FlatCsrIterator<'a> {
    type Item = (VertexId, &'a EdgeRecord);

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_vertex < self.csr.vertex_capacity {
            let offset = self.csr.offsets[self.current_vertex];
            let degree = self.csr.degrees[self.current_vertex];

            if self.current_edge < offset {
                self.current_edge = offset;
            }

            while self.current_edge < offset + degree {
                let edge = &self.csr.edges[self.current_edge];
                self.current_edge += 1;

                if edge.is_valid(self.ts) {
                    return Some((self.current_vertex as VertexId, edge));
                }
            }

            self.current_vertex += 1;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_query() {
        let mut csr = FlatCsr::with_capacity(10, 100);

        let edge1 = EdgeRecord::new(0, 1, 0, 0, 100);
        let edge2 = EdgeRecord::new(0, 2, 1, 1, 100);
        let edge3 = EdgeRecord::new(1, 3, 2, 2, 100);

        assert!(csr.insert(0, edge1));
        assert!(csr.insert(0, edge2));
        assert!(csr.insert(1, edge3));

        assert_eq!(csr.degree(0, 100), 2);
        assert_eq!(csr.degree(1, 100), 1);
        assert!(csr.has_edge(0, 1, 100));
        assert!(csr.has_edge(0, 2, 100));
    }

    #[test]
    fn test_delete() {
        let mut csr = FlatCsr::with_capacity(10, 100);

        csr.insert(0, EdgeRecord::new(0, 1, 0, 0, 100));
        csr.insert(0, EdgeRecord::new(0, 2, 1, 1, 100));

        assert!(csr.delete(0, 1, 200));
        assert!(!csr.has_edge(0, 1, 300));
        assert!(csr.has_edge(0, 2, 300));

        assert_eq!(csr.degree(0, 300), 1);
    }

    #[test]
    fn test_revert_delete() {
        let mut csr = FlatCsr::with_capacity(10, 100);

        csr.insert(0, EdgeRecord::new(0, 1, 0, 0, 100));
        csr.delete(0, 1, 200);

        assert!(csr.revert_delete(0, 1, 100));
        assert!(csr.has_edge(0, 1, 300));
    }

    #[test]
    fn test_mvcc_visibility() {
        let mut csr = FlatCsr::with_capacity(10, 100);

        let result1 = csr.insert(0, EdgeRecord::new(0, 1, 0, 0, 100));
        assert!(result1);
        let result2 = csr.insert(0, EdgeRecord::new(0, 2, 1, 1, 150));
        assert!(result2);

        assert_eq!(csr.degree(0, 50), 0, "At ts=50, no edges should be visible");
        assert_eq!(csr.degree(0, 120), 1, "At ts=120, only edge 0 should be visible");
        assert_eq!(csr.degree(0, 180), 2, "At ts=180, both edges should be visible");

        csr.delete(0, 1, 200);
        assert_eq!(csr.degree(0, 250), 1, "At ts=250, only edge 1 should be visible (edge 0 deleted)");
    }

    #[test]
    fn test_iterator() {
        let mut csr = FlatCsr::with_capacity(10, 100);

        csr.insert(0, EdgeRecord::new(0, 1, 0, 0, 100));
        csr.insert(0, EdgeRecord::new(0, 2, 1, 1, 100));
        csr.insert(1, EdgeRecord::new(1, 3, 2, 2, 100));

        let edges: Vec<_> = csr.iter_edges(0, 100).collect();
        assert_eq!(edges.len(), 2);

        let all_edges: Vec<_> = csr.iter(100).collect();
        assert_eq!(all_edges.len(), 3);
    }

    #[test]
    fn test_compact() {
        let mut csr = FlatCsr::with_capacity(10, 100);

        csr.insert(0, EdgeRecord::new(0, 1, 0, 0, 100));
        csr.insert(0, EdgeRecord::new(0, 2, 1, 1, 100));
        csr.delete(0, 1, 200);

        csr.compact();

        assert_eq!(csr.edges.len(), 1);
        assert_eq!(csr.degree(0, 300), 1);
    }

    #[test]
    fn test_dump_and_load() {
        let mut csr = FlatCsr::with_capacity(10, 100);

        csr.insert(0, EdgeRecord::new(0, 1, 0, 0, 100));
        csr.insert(0, EdgeRecord::new(0, 2, 1, 1, 100));
        csr.insert(1, EdgeRecord::new(1, 3, 2, 2, 100));

        let data = csr.dump();

        let mut loaded = FlatCsr::new();
        loaded.load(&data);

        assert_eq!(loaded.vertex_capacity(), csr.vertex_capacity());
        assert_eq!(loaded.edge_count(), csr.edge_count());
        assert!(loaded.has_edge(0, 1, 100));
        assert!(loaded.has_edge(0, 2, 100));
        assert!(loaded.has_edge(1, 3, 100));
    }
}
