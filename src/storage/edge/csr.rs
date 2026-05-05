//! CSR (Compressed Sparse Row) Implementation
//!
//! Immutable CSR for read-optimized edge storage.
//! Uses contiguous storage for memory efficiency and cache locality.

use std::sync::atomic::{AtomicU64, Ordering};

use super::{EdgeId, ImmutableNbr, Timestamp, VertexId};

/// Immutable CSR with contiguous storage
///
/// Standard CSR format:
/// - `offsets`: Offset array where offsets[v] is the start index in edges for vertex v
/// - `edges`: Contiguous array of all edges
/// - offsets[vertex_capacity] stores the total edge count
#[derive(Debug)]
pub struct Csr {
    offsets: Vec<u32>,
    edges: Vec<ImmutableNbr>,
    edge_count: AtomicU64,
    vertex_capacity: usize,
}

impl Clone for Csr {
    fn clone(&self) -> Self {
        Self {
            offsets: self.offsets.clone(),
            edges: self.edges.clone(),
            edge_count: AtomicU64::new(self.edge_count.load(Ordering::Relaxed)),
            vertex_capacity: self.vertex_capacity,
        }
    }
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

    #[inline]
    pub fn vertex_capacity(&self) -> usize {
        self.vertex_capacity
    }

    #[inline]
    pub fn edge_count(&self) -> u64 {
        self.edge_count.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    /// Resize vertex capacity
    pub fn resize(&mut self, new_vertex_capacity: usize) {
        if new_vertex_capacity > self.vertex_capacity {
            let last_offset = *self.offsets.last().unwrap_or(&0);
            self.offsets.resize(new_vertex_capacity + 1, last_offset);
            self.vertex_capacity = new_vertex_capacity;
        }
    }

    /// Batch insert edges (optimized for bulk loading)
    pub fn batch_put_edges(
        &mut self,
        src_list: &[VertexId],
        dst_list: &[VertexId],
        edge_ids: &[EdgeId],
        prop_offsets: &[u32],
        _ts: Timestamp,
    ) {
        if src_list.is_empty() {
            return;
        }

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
        let mut new_edges = vec![ImmutableNbr::new(0, 0, 0); total_edges];

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
            if src < current_pos.len() - 1 {
                let pos = current_pos[src] as usize;
                if pos < new_edges.len() {
                    new_edges[pos] = ImmutableNbr::new(dst_list[i], edge_ids[i], prop_offsets[i]);
                    current_pos[src] += 1;
                }
            }
        }

        self.offsets = new_offsets;
        self.edges = new_edges;
        self.edge_count
            .fetch_add(src_list.len() as u64, Ordering::Relaxed);
    }

    /// Get edges of a vertex
    #[inline]
    pub fn edges_of(&self, vid: VertexId) -> &[ImmutableNbr] {
        let vid_idx = vid as usize;
        if vid_idx >= self.vertex_capacity {
            return &[];
        }

        let start = self.offsets[vid_idx] as usize;
        let end = self.offsets[vid_idx + 1] as usize;

        if start >= self.edges.len() || end > self.edges.len() || start > end {
            return &[];
        }

        &self.edges[start..end]
    }

    /// Get degree of a vertex
    #[inline]
    pub fn degree(&self, vid: VertexId) -> usize {
        let vid_idx = vid as usize;
        if vid_idx >= self.vertex_capacity {
            return 0;
        }

        (self.offsets[vid_idx + 1] - self.offsets[vid_idx]) as usize
    }

    /// Check if an edge exists
    pub fn has_edge(&self, src: VertexId, dst: VertexId) -> bool {
        let edges = self.edges_of(src);
        edges.iter().any(|e| e.neighbor == dst)
    }

    /// Get a specific edge
    pub fn get_edge(&self, src: VertexId, dst: VertexId) -> Option<&ImmutableNbr> {
        let edges = self.edges_of(src);
        edges.iter().find(|e| e.neighbor == dst)
    }

    /// Get edge by edge_id
    pub fn get_edge_by_id(&self, src: VertexId, edge_id: EdgeId) -> Option<&ImmutableNbr> {
        let edges = self.edges_of(src);
        edges.iter().find(|e| e.edge_id == edge_id)
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.offsets = vec![0];
        self.edges.clear();
        self.edge_count.store(0, Ordering::Relaxed);
        self.vertex_capacity = 1;
    }

    /// Create iterator over all edges
    pub fn iter(&self) -> CsrIterator {
        CsrIterator::new(self)
    }

    /// Create iterator over edges of a specific vertex
    pub fn iter_edges(&self, vid: VertexId) -> CsrEdgeIterator {
        CsrEdgeIterator::new(self, vid)
    }

    /// Dump to bytes
    pub fn dump(&self) -> Vec<u8> {
        let mut result = Vec::new();

        result.extend_from_slice(&(self.vertex_capacity as u64).to_le_bytes());
        result.extend_from_slice(&self.edge_count.load(Ordering::Relaxed).to_le_bytes());

        result.extend_from_slice(&(self.offsets.len() as u64).to_le_bytes());
        for &offset in &self.offsets {
            result.extend_from_slice(&offset.to_le_bytes());
        }

        result.extend_from_slice(&(self.edges.len() as u64).to_le_bytes());
        for edge in &self.edges {
            result.extend_from_slice(&edge.neighbor.to_le_bytes());
            result.extend_from_slice(&edge.edge_id.to_le_bytes());
            result.extend_from_slice(&edge.prop_offset.to_le_bytes());
        }

        result
    }

    /// Load from bytes
    pub fn load(&mut self, data: &[u8]) {
        if data.len() < 24 {
            return;
        }

        let mut offset = 0;

        let vertex_capacity =
            u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8])) as usize;
        offset += 8;

        let edge_count = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8]));
        offset += 8;

        let offsets_len =
            u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8])) as usize;
        offset += 8;

        if offset + offsets_len * 4 > data.len() {
            return;
        }
        let mut offsets = Vec::with_capacity(offsets_len);
        for _ in 0..offsets_len {
            let off = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap_or([0; 4]));
            offsets.push(off);
            offset += 4;
        }

        if offset + 8 > data.len() {
            return;
        }
        let edges_len =
            u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8])) as usize;
        offset += 8;

        if offset + edges_len * 20 > data.len() {
            return;
        }
        let mut edges = Vec::with_capacity(edges_len);
        for _ in 0..edges_len {
            let neighbor =
                u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8]));
            offset += 8;
            let edge_id = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8]));
            offset += 8;
            let prop_offset =
                u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap_or([0; 4]));
            offset += 4;

            edges.push(ImmutableNbr {
                neighbor,
                edge_id,
                prop_offset,
            });
        }

        self.vertex_capacity = vertex_capacity;
        self.offsets = offsets;
        self.edges = edges;
        self.edge_count.store(edge_count, Ordering::Relaxed);
    }

    /// Get raw offsets slice
    pub fn offsets(&self) -> &[u32] {
        &self.offsets
    }

    /// Get raw edges slice
    pub fn edges(&self) -> &[ImmutableNbr] {
        &self.edges
    }

    /// Create from raw components (for advanced use)
    pub fn from_raw(offsets: Vec<u32>, edges: Vec<ImmutableNbr>, vertex_capacity: usize) -> Self {
        let edge_count = edges.len() as u64;
        Self {
            offsets,
            edges,
            edge_count: AtomicU64::new(edge_count),
            vertex_capacity,
        }
    }
}

impl Default for Csr {
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator over all edges in the CSR
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

            if self.current_edge < end && self.current_edge < self.csr.edges.len() {
                let edge = &self.csr.edges[self.current_edge];
                self.current_edge += 1;
                return Some((self.current_vertex as VertexId, edge));
            }

            self.current_vertex += 1;
            self.current_edge = 0;
        }
        None
    }
}

/// Iterator over edges of a specific vertex
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

        assert!(csr.has_edge(0, 1));
        assert!(csr.has_edge(0, 2));
        assert!(!csr.has_edge(0, 3));
    }

    #[test]
    fn test_iterator() {
        let mut csr = Csr::with_capacity(5, 20);

        csr.batch_put_edges(&[0, 1, 2], &[1, 2, 3], &[0, 1, 2], &[0, 0, 0], 100);

        let count = csr.iter().count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_dump_and_load() {
        let mut csr1 = Csr::with_capacity(10, 100);

        csr1.batch_put_edges(
            &[0, 0, 1, 2],
            &[1, 2, 3, 0],
            &[0, 1, 2, 3],
            &[0, 1, 2, 3],
            100,
        );

        let data = csr1.dump();

        let mut csr2 = Csr::new();
        csr2.load(&data);

        assert_eq!(csr2.vertex_capacity(), csr1.vertex_capacity());
        assert_eq!(csr2.edge_count(), csr1.edge_count());
        assert!(csr2.has_edge(0, 1));
        assert!(csr2.has_edge(0, 2));
        assert!(csr2.has_edge(1, 3));
    }

    #[test]
    fn test_get_edge() {
        let mut csr = Csr::with_capacity(10, 100);

        csr.batch_put_edges(&[0, 0], &[1, 2], &[100, 101], &[0, 1], 100);

        let edge = csr.get_edge(0, 1);
        assert!(edge.is_some());
        assert_eq!(edge.unwrap().edge_id, 100);

        let edge = csr.get_edge(0, 3);
        assert!(edge.is_none());
    }

    #[test]
    fn test_get_edge_by_id() {
        let mut csr = Csr::with_capacity(10, 100);

        csr.batch_put_edges(&[0, 0], &[1, 2], &[100, 101], &[0, 1], 100);

        let edge = csr.get_edge_by_id(0, 100);
        assert!(edge.is_some());
        assert_eq!(edge.unwrap().neighbor, 1);
    }
}
