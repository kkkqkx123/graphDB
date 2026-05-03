//! CSR (Compressed Sparse Row) Implementation
//!
//! Immutable CSR for read-optimized edge storage.

use std::sync::atomic::{AtomicU64, Ordering};

use super::{ImmutableNbr, Nbr, VertexId, EdgeId, Timestamp, INVALID_TIMESTAMP, MAX_TIMESTAMP};

#[derive(Debug, Clone)]
pub struct Csr {
    offsets: Vec<u32>,
    edges: Vec<ImmutableNbr>,
    edge_count: AtomicU64,
    vertex_capacity: usize,
}

impl Csr {
    pub fn new() -> Self {
        Self {
            offsets: vec![0],
            edges: Vec::new(),
            edge_count: AtomicU64::new(0),
            vertex_capacity: 1,
        }
    }

    pub fn with_capacity(vertex_capacity: usize, edge_capacity: usize) -> Self {
        Self {
            offsets: vec![0; vertex_capacity + 1],
            edges: Vec::with_capacity(edge_capacity),
            edge_count: AtomicU64::new(0),
            vertex_capacity,
        }
    }

    pub fn resize(&mut self, new_vertex_capacity: usize) {
        if new_vertex_capacity > self.vertex_capacity {
            self.offsets.resize(new_vertex_capacity + 1, self.offsets[self.vertex_capacity]);
            self.vertex_capacity = new_vertex_capacity;
        }
    }

    pub fn batch_put_edges(
        &mut self,
        src_list: &[VertexId],
        dst_list: &[VertexId],
        edge_ids: &[EdgeId],
        prop_offsets: &[u32],
        _ts: Timestamp,
    ) {
        let max_vertex = src_list.iter().max().copied().unwrap_or(0) as usize;
        if max_vertex >= self.vertex_capacity {
            self.resize(max_vertex + 1);
        }

        let mut degrees = vec![0u32; self.vertex_capacity];
        for &src in src_list {
            let src_idx = src as usize;
            if src_idx < degrees.len() {
                degrees[src_idx] += 1;
            }
        }

        let mut new_offsets = vec![0u32; self.vertex_capacity + 1];
        let mut cumsum = 0u32;
        for (i, &deg) in degrees.iter().enumerate() {
            new_offsets[i] = cumsum;
            cumsum += deg;
        }
        new_offsets[self.vertex_capacity] = cumsum;

        let total_edges = self.edges.len() + src_list.len();
        let mut new_edges = vec![
            ImmutableNbr::new(0, 0, 0);
            total_edges
        ];

        for i in 0..self.vertex_capacity {
            let old_start = self.offsets[i] as usize;
            let old_end = self.offsets[i + 1] as usize;
            let new_start = new_offsets[i] as usize;
            if old_start < old_end && old_end <= self.edges.len() {
                new_edges[new_start..new_start + (old_end - old_start)]
                    .copy_from_slice(&self.edges[old_start..old_end]);
            }
        }

        let mut current_pos = new_offsets.clone();
        for i in 0..src_list.len() {
            let src = src_list[i] as usize;
            let pos = current_pos[src] as usize;
            new_edges[pos] = ImmutableNbr::new(dst_list[i], edge_ids[i], prop_offsets[i]);
            current_pos[src] += 1;
        }

        self.offsets = new_offsets;
        self.edges = new_edges;
        self.edge_count.fetch_add(src_list.len() as u64, Ordering::Relaxed);
    }

    pub fn edges_of(&self, vid: VertexId) -> &[ImmutableNbr] {
        let vid_idx = vid as usize;
        if vid_idx >= self.vertex_capacity {
            return &[];
        }

        let start = self.offsets[vid_idx] as usize;
        let end = self.offsets[vid_idx + 1] as usize;

        if start >= self.edges.len() || end > self.edges.len() {
            return &[];
        }

        &self.edges[start..end]
    }

    pub fn degree(&self, vid: VertexId) -> usize {
        let vid_idx = vid as usize;
        if vid_idx >= self.vertex_capacity {
            return 0;
        }

        (self.offsets[vid_idx + 1] - self.offsets[vid_idx]) as usize
    }

    pub fn edge_count(&self) -> u64 {
        self.edge_count.load(Ordering::Relaxed)
    }

    pub fn vertex_capacity(&self) -> usize {
        self.vertex_capacity
    }

    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    pub fn clear(&mut self) {
        self.offsets = vec![0];
        self.edges.clear();
        self.edge_count.store(0, Ordering::Relaxed);
        self.vertex_capacity = 1;
    }

    pub fn iter(&self) -> CsrIterator {
        CsrIterator::new(self)
    }

    pub fn iter_edges(&self, vid: VertexId) -> CsrEdgeIterator {
        CsrEdgeIterator::new(self, vid)
    }
}

impl Default for Csr {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CsrIterator<'a> {
    csr: &'a Csr,
    current_vertex: usize,
    current_edge: usize,
}

impl<'a> CsrIterator<'a> {
    pub fn new(csr: &'a Csr) -> Self {
        Self {
            csr,
            current_vertex: 0,
            current_edge: 0,
        }
    }
}

impl<'a> Iterator for CsrIterator<'a> {
    type Item = (VertexId, &'a ImmutableNbr);

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_vertex < self.csr.vertex_capacity {
            let start = self.csr.offsets[self.current_vertex] as usize;
            let end = self.csr.offsets[self.current_vertex + 1] as usize;

            if self.current_edge < start {
                self.current_edge = start;
            }

            if self.current_edge < end {
                let edge = &self.csr.edges[self.current_edge];
                self.current_edge += 1;
                return Some((self.current_vertex as VertexId, edge));
            }

            self.current_vertex += 1;
        }
        None
    }
}

pub struct CsrEdgeIterator<'a> {
    edges: &'a [ImmutableNbr],
    current: usize,
}

impl<'a> CsrEdgeIterator<'a> {
    pub fn new(csr: &'a Csr, vid: VertexId) -> Self {
        let vid_idx = vid as usize;
        let edges = if vid_idx < csr.vertex_capacity {
            let start = csr.offsets[vid_idx] as usize;
            let end = csr.offsets[vid_idx + 1] as usize;
            if start < csr.edges.len() && end <= csr.edges.len() {
                &csr.edges[start..end]
            } else {
                &[]
            }
        } else {
            &[]
        };

        Self { edges, current: 0 }
    }
}

impl<'a> Iterator for CsrEdgeIterator<'a> {
    type Item = &'a ImmutableNbr;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.edges.len() {
            let edge = &self.edges[self.current];
            self.current += 1;
            Some(edge)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut csr = Csr::with_capacity(10, 100);

        csr.batch_put_edges(
            &[0, 0, 1, 2],
            &[1, 2, 3, 0],
            &[0, 1, 2, 3],
            &[0, 1, 2, 3],
            100,
        );

        assert_eq!(csr.degree(0), 2);
        assert_eq!(csr.degree(1), 1);
        assert_eq!(csr.degree(2), 1);
        assert_eq!(csr.edge_count(), 4);

        let edges: Vec<_> = csr.iter_edges(0).collect();
        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn test_iterator() {
        let mut csr = Csr::with_capacity(5, 20);

        csr.batch_put_edges(
            &[0, 1, 2],
            &[1, 2, 3],
            &[0, 1, 2],
            &[0, 0, 0],
            100,
        );

        let count = csr.iter().count();
        assert_eq!(count, 3);
    }
}
