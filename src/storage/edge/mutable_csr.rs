//! Mutable CSR Implementation
//!
//! Mutable CSR supporting dynamic edge operations with MVCC timestamps.

use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use super::{Nbr, VertexId, EdgeId, Timestamp, INVALID_TIMESTAMP, MAX_TIMESTAMP};

const DEFAULT_CAPACITY: usize = 8;
const GROWTH_FACTOR: f64 = 1.5;

#[derive(Debug)]
pub struct MutableCsr {
    adj_lists: Vec<Vec<Nbr>>,
    edge_count: AtomicU64,
    deleted_edges: Mutex<HashSet<(VertexId, EdgeId)>>,
    vertex_capacity: usize,
}

impl MutableCsr {
    pub fn new() -> Self {
        Self::with_capacity(1024)
    }

    pub fn with_capacity(vertex_capacity: usize) -> Self {
        let capacity = vertex_capacity.max(1);
        Self {
            adj_lists: vec![Vec::new(); capacity],
            edge_count: AtomicU64::new(0),
            deleted_edges: Mutex::new(HashSet::new()),
            vertex_capacity: capacity,
        }
    }

    pub fn resize(&mut self, new_vertex_capacity: usize) {
        if new_vertex_capacity > self.vertex_capacity {
            self.adj_lists.resize(new_vertex_capacity, Vec::new());
            self.vertex_capacity = new_vertex_capacity;
        }
    }

    pub fn insert_edge(
        &mut self,
        src: VertexId,
        dst: VertexId,
        edge_id: EdgeId,
        prop_offset: u32,
        ts: Timestamp,
    ) -> bool {
        let src_idx = src as usize;
        if src_idx >= self.vertex_capacity {
            let new_capacity = (src_idx + 1).next_power_of_two();
            self.resize(new_capacity);
        }

        let adj_list = &mut self.adj_lists[src_idx];

        for nbr in adj_list.iter() {
            if nbr.neighbor == dst && nbr.timestamp != INVALID_TIMESTAMP {
                return false;
            }
        }

        adj_list.push(Nbr::new(dst, edge_id, prop_offset, ts));
        self.edge_count.fetch_add(1, Ordering::Relaxed);
        true
    }

    pub fn delete_edge(&mut self, src: VertexId, edge_id: EdgeId, ts: Timestamp) -> bool {
        let src_idx = src as usize;
        if src_idx >= self.vertex_capacity {
            return false;
        }

        let adj_list = &mut self.adj_lists[src_idx];
        for nbr in adj_list.iter_mut() {
            if nbr.edge_id == edge_id && nbr.timestamp != INVALID_TIMESTAMP {
                if nbr.timestamp <= ts {
                    nbr.timestamp = INVALID_TIMESTAMP;
                    self.edge_count.fetch_sub(1, Ordering::Relaxed);
                    return true;
                }
            }
        }
        false
    }

    pub fn delete_edge_by_dst(&mut self, src: VertexId, dst: VertexId, ts: Timestamp) -> bool {
        let src_idx = src as usize;
        if src_idx >= self.vertex_capacity {
            return false;
        }

        let mut deleted = false;
        let adj_list = &mut self.adj_lists[src_idx];
        for nbr in adj_list.iter_mut() {
            if nbr.neighbor == dst && nbr.timestamp != INVALID_TIMESTAMP {
                if nbr.timestamp <= ts {
                    nbr.timestamp = INVALID_TIMESTAMP;
                    self.edge_count.fetch_sub(1, Ordering::Relaxed);
                    deleted = true;
                }
            }
        }
        deleted
    }

    pub fn revert_delete(&mut self, src: VertexId, edge_id: EdgeId, ts: Timestamp) -> bool {
        let src_idx = src as usize;
        if src_idx >= self.vertex_capacity {
            return false;
        }

        let adj_list = &mut self.adj_lists[src_idx];
        for nbr in adj_list.iter_mut() {
            if nbr.edge_id == edge_id && nbr.timestamp == INVALID_TIMESTAMP {
                nbr.timestamp = ts;
                self.edge_count.fetch_add(1, Ordering::Relaxed);
                return true;
            }
        }
        false
    }

    pub fn edges_of(&self, src: VertexId, ts: Timestamp) -> Vec<&Nbr> {
        let src_idx = src as usize;
        if src_idx >= self.vertex_capacity {
            return Vec::new();
        }

        self.adj_lists[src_idx]
            .iter()
            .filter(|nbr| nbr.timestamp <= ts && nbr.timestamp != INVALID_TIMESTAMP)
            .collect()
    }

    pub fn degree(&self, src: VertexId, ts: Timestamp) -> usize {
        let src_idx = src as usize;
        if src_idx >= self.vertex_capacity {
            return 0;
        }

        self.adj_lists[src_idx]
            .iter()
            .filter(|nbr| nbr.timestamp <= ts && nbr.timestamp != INVALID_TIMESTAMP)
            .count()
    }

    pub fn has_edge(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> bool {
        let src_idx = src as usize;
        if src_idx >= self.vertex_capacity {
            return false;
        }

        self.adj_lists[src_idx]
            .iter()
            .any(|nbr| nbr.neighbor == dst && nbr.timestamp <= ts && nbr.timestamp != INVALID_TIMESTAMP)
    }

    pub fn get_edge(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> Option<&Nbr> {
        let src_idx = src as usize;
        if src_idx >= self.vertex_capacity {
            return None;
        }

        self.adj_lists[src_idx]
            .iter()
            .find(|nbr| nbr.neighbor == dst && nbr.timestamp <= ts && nbr.timestamp != INVALID_TIMESTAMP)
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
        for adj_list in &mut self.adj_lists {
            adj_list.clear();
        }
        self.edge_count.store(0, Ordering::Relaxed);
        self.deleted_edges.lock().unwrap().clear();
    }

    pub fn compact(&mut self) {
        for adj_list in &mut self.adj_lists {
            let original_len = adj_list.len();
            adj_list.retain(|nbr| nbr.timestamp != INVALID_TIMESTAMP);
            let removed = original_len - adj_list.len();
            if removed > 0 {
                self.edge_count.fetch_sub(removed as u64, Ordering::Relaxed);
            }
        }
    }

    pub fn batch_put_edges(
        &mut self,
        src_list: &[VertexId],
        dst_list: &[VertexId],
        edge_ids: &[EdgeId],
        prop_offsets: &[u32],
        ts: Timestamp,
    ) {
        let max_vertex = src_list.iter().max().copied().unwrap_or(0) as usize;
        if max_vertex >= self.vertex_capacity {
            let new_capacity = (max_vertex + 1).next_power_of_two();
            self.resize(new_capacity);
        }

        for i in 0..src_list.len() {
            let src = src_list[i];
            let dst = dst_list[i];
            let edge_id = edge_ids[i];
            let prop_offset = prop_offsets[i];

            let src_idx = src as usize;
            self.adj_lists[src_idx].push(Nbr::new(dst, edge_id, prop_offset, ts));
        }

        self.edge_count.fetch_add(src_list.len() as u64, Ordering::Relaxed);
    }

    pub fn batch_delete_edges(&mut self, edges: &[(VertexId, EdgeId)], ts: Timestamp) {
        for &(src, edge_id) in edges {
            self.delete_edge(src, edge_id, ts);
        }
    }

    pub fn iter(&self, ts: Timestamp) -> MutableCsrIterator {
        MutableCsrIterator::new(self, ts)
    }

    pub fn iter_edges(&self, src: VertexId, ts: Timestamp) -> MutableCsrEdgeIterator {
        MutableCsrEdgeIterator::new(self, src, ts)
    }
}

impl Default for MutableCsr {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MutableCsrIterator<'a> {
    csr: &'a MutableCsr,
    ts: Timestamp,
    current_vertex: usize,
    current_edge: usize,
}

impl<'a> MutableCsrIterator<'a> {
    pub fn new(csr: &'a MutableCsr, ts: Timestamp) -> Self {
        Self {
            csr,
            ts,
            current_vertex: 0,
            current_edge: 0,
        }
    }
}

impl<'a> Iterator for MutableCsrIterator<'a> {
    type Item = (VertexId, &'a Nbr);

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_vertex < self.csr.vertex_capacity {
            let adj_list = &self.csr.adj_lists[self.current_vertex];

            while self.current_edge < adj_list.len() {
                let nbr = &adj_list[self.current_edge];
                self.current_edge += 1;

                if nbr.timestamp <= self.ts && nbr.timestamp != INVALID_TIMESTAMP {
                    return Some((self.current_vertex as VertexId, nbr));
                }
            }

            self.current_vertex += 1;
            self.current_edge = 0;
        }
        None
    }
}

pub struct MutableCsrEdgeIterator<'a> {
    edges: &'a [Nbr],
    ts: Timestamp,
    current: usize,
}

impl<'a> MutableCsrEdgeIterator<'a> {
    pub fn new(csr: &'a MutableCsr, src: VertexId, ts: Timestamp) -> Self {
        let src_idx = src as usize;
        let edges: &'a [Nbr] = if src_idx < csr.vertex_capacity {
            &csr.adj_lists[src_idx]
        } else {
            &[]
        };

        Self { edges, ts, current: 0 }
    }
}

impl<'a> Iterator for MutableCsrEdgeIterator<'a> {
    type Item = &'a Nbr;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current < self.edges.len() {
            let nbr = &self.edges[self.current];
            self.current += 1;

            if nbr.timestamp <= self.ts && nbr.timestamp != INVALID_TIMESTAMP {
                return Some(nbr);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_query() {
        let mut csr = MutableCsr::with_capacity(10);

        assert!(csr.insert_edge(0, 1, 0, 0, 100));
        assert!(csr.insert_edge(0, 2, 1, 1, 100));
        assert!(csr.insert_edge(1, 2, 2, 2, 100));

        assert_eq!(csr.degree(0, 100), 2);
        assert_eq!(csr.degree(1, 100), 1);
        assert!(csr.has_edge(0, 1, 100));
        assert!(csr.has_edge(0, 2, 100));
    }

    #[test]
    fn test_delete() {
        let mut csr = MutableCsr::with_capacity(10);

        csr.insert_edge(0, 1, 0, 0, 100);
        csr.insert_edge(0, 2, 1, 1, 100);

        assert!(csr.delete_edge(0, 0, 200));
        assert!(!csr.has_edge(0, 1, 300));
        assert!(csr.has_edge(0, 2, 300));

        assert_eq!(csr.degree(0, 300), 1);
    }

    #[test]
    fn test_revert_delete() {
        let mut csr = MutableCsr::with_capacity(10);

        csr.insert_edge(0, 1, 0, 0, 100);
        csr.delete_edge(0, 0, 200);

        assert!(csr.revert_delete(0, 0, 100));
        assert!(csr.has_edge(0, 1, 300));
    }

    #[test]
    fn test_mvcc_visibility() {
        let mut csr = MutableCsr::with_capacity(10);

        csr.insert_edge(0, 1, 0, 0, 100);
        csr.insert_edge(0, 2, 1, 1, 150);
        csr.delete_edge(0, 0, 200);

        assert_eq!(csr.degree(0, 50), 0);
        assert_eq!(csr.degree(0, 120), 2);
        assert_eq!(csr.degree(0, 180), 2);
        assert_eq!(csr.degree(0, 250), 1);
    }

    #[test]
    fn test_batch_operations() {
        let mut csr = MutableCsr::with_capacity(10);

        csr.batch_put_edges(
            &[0, 0, 1, 2],
            &[1, 2, 3, 0],
            &[0, 1, 2, 3],
            &[0, 1, 2, 3],
            100,
        );

        assert_eq!(csr.edge_count(), 4);

        csr.batch_delete_edges(&[(0, 0), (1, 2)], 200);
        assert_eq!(csr.degree(0, 300), 1);
        assert_eq!(csr.degree(1, 300), 0);
    }

    #[test]
    fn test_compact() {
        let mut csr = MutableCsr::with_capacity(10);

        csr.insert_edge(0, 1, 0, 0, 100);
        csr.insert_edge(0, 2, 1, 1, 100);
        csr.delete_edge(0, 0, 200);

        csr.compact();

        assert_eq!(csr.adj_lists[0].len(), 1);
    }
}
