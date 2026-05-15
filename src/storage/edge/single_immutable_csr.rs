//! Single Immutable CSR Implementation
//!
//! Optimized immutable CSR for scenarios where each vertex has at most one outgoing edge.
//! Uses a simple array for maximum memory efficiency and O(1) access.
//!
//! Use cases:
//! - Snapshot storage for single-edge relationships
//! - Read-only views of one-to-one relationships

use std::sync::atomic::{AtomicU64, Ordering};

use super::{CsrBase, CsrType, EdgeId, ImmutableCsrTrait, ImmutableNbr, VertexId};

fn invalid_vertex_id() -> VertexId {
    VertexId::from_u64(u64::MAX)
}

pub struct SingleImmutableCsr {
    nbr_list: Vec<ImmutableNbr>,
    edge_count: AtomicU64,
    vertex_capacity: usize,
}

impl Clone for SingleImmutableCsr {
    fn clone(&self) -> Self {
        Self {
            nbr_list: self.nbr_list.clone(),
            edge_count: AtomicU64::new(self.edge_count.load(Ordering::Relaxed)),
            vertex_capacity: self.vertex_capacity,
        }
    }
}

impl std::fmt::Debug for SingleImmutableCsr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SingleImmutableCsr")
            .field("vertex_capacity", &self.vertex_capacity)
            .field("edge_count", &self.edge_count.load(Ordering::Relaxed))
            .finish_non_exhaustive()
    }
}

impl SingleImmutableCsr {
    pub fn new() -> Self {
        Self::with_capacity(1024)
    }

    pub fn with_capacity(vertex_capacity: usize) -> Self {
        let vertex_cap = vertex_capacity.max(1);
        let nbr_list = vec![ImmutableNbr::new(invalid_vertex_id(), 0, 0); vertex_cap];

        Self {
            nbr_list,
            edge_count: AtomicU64::new(0),
            vertex_capacity: vertex_cap,
        }
    }

    pub fn vertex_capacity(&self) -> usize {
        self.vertex_capacity
    }

    pub fn edge_count(&self) -> u64 {
        self.edge_count.load(Ordering::Relaxed)
    }

    pub fn is_empty(&self) -> bool {
        self.edge_count.load(Ordering::Relaxed) == 0
    }

    pub fn resize(&mut self, new_vertex_capacity: usize) {
        if new_vertex_capacity > self.vertex_capacity {
            let additional = new_vertex_capacity - self.vertex_capacity;
            self.nbr_list
                .extend(std::iter::repeat_n(
                    ImmutableNbr::new(invalid_vertex_id(), 0, 0),
                    additional,
                ));
            self.vertex_capacity = new_vertex_capacity;
        }
    }

    pub fn batch_put_edges(
        &mut self,
        src_list: &[VertexId],
        dst_list: &[VertexId],
        edge_ids: &[EdgeId],
        prop_offsets: &[u32],
    ) {
        if src_list.is_empty() {
            return;
        }

        let max_vertex = src_list
            .iter()
            .max()
            .cloned()
            .unwrap_or(VertexId::zero())
            .as_int64()
            .unwrap_or(0) as usize;
        if max_vertex >= self.vertex_capacity {
            self.resize(max_vertex + 1);
        }

        for i in 0..src_list.len() {
            let src = src_list[i].as_int64().unwrap_or(0) as usize;
            if src < self.vertex_capacity {
                self.nbr_list[src] =
                    ImmutableNbr::new(dst_list[i], edge_ids[i], prop_offsets[i]);
            }
        }

        self.edge_count
            .store(src_list.len() as u64, Ordering::Relaxed);
    }

    pub fn get_edge(&self, src: VertexId) -> Option<&ImmutableNbr> {
        let src_idx = src.as_int64().unwrap_or(0) as usize;

        if src_idx >= self.vertex_capacity {
            return None;
        }

        let nbr = &self.nbr_list[src_idx];
        if nbr.neighbor == invalid_vertex_id() {
            return None;
        }

        Some(nbr)
    }

    pub fn get_edge_by_dst(&self, src: VertexId, dst: VertexId) -> Option<&ImmutableNbr> {
        let edge = self.get_edge(src)?;
        if edge.neighbor == dst {
            Some(edge)
        } else {
            None
        }
    }

    pub fn edges_of(&self, src: VertexId) -> Option<&ImmutableNbr> {
        self.get_edge(src)
    }

    pub fn degree(&self, src: VertexId) -> usize {
        if self.get_edge(src).is_some() {
            1
        } else {
            0
        }
    }

    pub fn has_edge(&self, src: VertexId, dst: VertexId) -> bool {
        self.get_edge_by_dst(src, dst).is_some()
    }

    pub fn clear(&mut self) {
        for nbr in &mut self.nbr_list {
            *nbr = ImmutableNbr::new(invalid_vertex_id(), 0, 0);
        }
        self.edge_count.store(0, Ordering::Relaxed);
    }

    pub fn dump(&self) -> Vec<u8> {
        let mut result = Vec::new();

        result.extend_from_slice(&(self.vertex_capacity as u64).to_le_bytes());
        result.extend_from_slice(&self.edge_count.load(Ordering::Relaxed).to_le_bytes());

        for nbr in &self.nbr_list {
            result.extend_from_slice(&nbr.neighbor.as_int64().unwrap_or(0).to_le_bytes());
            result.extend_from_slice(&nbr.edge_id.to_le_bytes());
            result.extend_from_slice(&nbr.prop_offset.to_le_bytes());
        }

        result
    }

    pub fn load(&mut self, data: &[u8]) {
        if data.len() < 16 {
            return;
        }

        let mut offset = 0;

        let vertex_capacity =
            u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8])) as usize;
        offset += 8;

        let edge_count = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8]));
        offset += 8;

        let expected_len = 16 + vertex_capacity * 20;
        if data.len() < expected_len {
            return;
        }

        let mut nbr_list = Vec::with_capacity(vertex_capacity);
        for _ in 0..vertex_capacity {
            let neighbor =
                u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8]));
            offset += 8;
            let edge_id = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8]));
            offset += 8;
            let prop_offset =
                u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap_or([0; 4]));
            offset += 4;

            nbr_list.push(ImmutableNbr::new(
                VertexId::from_u64(neighbor),
                edge_id,
                prop_offset,
            ));
        }

        self.vertex_capacity = vertex_capacity;
        self.nbr_list = nbr_list;
        self.edge_count.store(edge_count, Ordering::Relaxed);
    }

    pub fn iter(&self) -> SingleImmutableCsrIterator<'_> {
        SingleImmutableCsrIterator::new(self)
    }
}

impl Default for SingleImmutableCsr {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SingleImmutableCsrIterator<'a> {
    csr: &'a SingleImmutableCsr,
    current_vertex: usize,
}

impl<'a> SingleImmutableCsrIterator<'a> {
    pub fn new(csr: &'a SingleImmutableCsr) -> Self {
        Self {
            csr,
            current_vertex: 0,
        }
    }
}

impl<'a> Iterator for SingleImmutableCsrIterator<'a> {
    type Item = (VertexId, &'a ImmutableNbr);

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_vertex < self.csr.vertex_capacity {
            let src = VertexId::from_int64(self.current_vertex as i64);
            self.current_vertex += 1;

            if let Some(nbr) = self.csr.get_edge(src) {
                return Some((src, nbr));
            }
        }
        None
    }
}

impl CsrBase for SingleImmutableCsr {
    fn vertex_capacity(&self) -> usize {
        self.vertex_capacity
    }

    fn edge_count(&self) -> u64 {
        self.edge_count.load(Ordering::Relaxed)
    }

    fn csr_type(&self) -> CsrType {
        CsrType::SingleImmutable
    }

    fn resize(&mut self, new_vertex_capacity: usize) {
        SingleImmutableCsr::resize(self, new_vertex_capacity);
    }

    fn clear(&mut self) {
        SingleImmutableCsr::clear(self);
    }

    fn dump(&self) -> Vec<u8> {
        SingleImmutableCsr::dump(self)
    }

    fn load(&mut self, data: &[u8]) {
        SingleImmutableCsr::load(self, data);
    }
}

impl ImmutableCsrTrait for SingleImmutableCsr {
    fn get_edge(&self, src: VertexId, dst: VertexId) -> Option<&ImmutableNbr> {
        SingleImmutableCsr::get_edge_by_dst(self, src, dst)
    }

    fn edges_of(&self, src: VertexId) -> &[ImmutableNbr] {
        static EMPTY: &[ImmutableNbr] = &[];
        if let Some(nbr) = SingleImmutableCsr::get_edge(self, src) {
            std::slice::from_ref(nbr)
        } else {
            EMPTY
        }
    }

    fn degree(&self, src: VertexId) -> usize {
        SingleImmutableCsr::degree(self, src)
    }

    fn has_edge(&self, src: VertexId, dst: VertexId) -> bool {
        SingleImmutableCsr::has_edge(self, src, dst)
    }

    fn batch_put_edges(
        &mut self,
        src_list: &[VertexId],
        dst_list: &[VertexId],
        edge_ids: &[EdgeId],
        prop_offsets: &[u32],
    ) {
        SingleImmutableCsr::batch_put_edges(self, src_list, dst_list, edge_ids, prop_offsets);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut csr = SingleImmutableCsr::with_capacity(10);

        csr.batch_put_edges(
            &[0, 1, 2].map(VertexId::from_int64),
            &[10, 20, 30].map(VertexId::from_int64),
            &[100, 101, 102],
            &[0, 1, 2],
        );

        assert_eq!(csr.edge_count(), 3);
        assert!(csr.has_edge(VertexId::from_int64(0), VertexId::from_int64(10)));
        assert!(csr.has_edge(VertexId::from_int64(1), VertexId::from_int64(20)));
        assert!(csr.has_edge(VertexId::from_int64(2), VertexId::from_int64(30)));
        assert!(!csr.has_edge(VertexId::from_int64(0), VertexId::from_int64(20)));
    }

    #[test]
    fn test_degree() {
        let mut csr = SingleImmutableCsr::with_capacity(10);

        csr.batch_put_edges(
            &[0, 1].map(VertexId::from_int64),
            &[10, 20].map(VertexId::from_int64),
            &[100, 101],
            &[0, 1],
        );

        assert_eq!(csr.degree(VertexId::from_int64(0)), 1);
        assert_eq!(csr.degree(VertexId::from_int64(1)), 1);
        assert_eq!(csr.degree(VertexId::from_int64(2)), 0);
    }

    #[test]
    fn test_dump_and_load() {
        let mut csr1 = SingleImmutableCsr::with_capacity(10);

        csr1.batch_put_edges(
            &[0, 1, 2].map(VertexId::from_int64),
            &[10, 20, 30].map(VertexId::from_int64),
            &[100, 101, 102],
            &[0, 1, 2],
        );

        let data = csr1.dump();

        let mut csr2 = SingleImmutableCsr::new();
        csr2.load(&data);

        assert_eq!(csr2.vertex_capacity(), csr1.vertex_capacity());
        assert_eq!(csr2.edge_count(), csr1.edge_count());
        assert!(csr2.has_edge(VertexId::from_int64(0), VertexId::from_int64(10)));
        assert!(csr2.has_edge(VertexId::from_int64(1), VertexId::from_int64(20)));
    }

    #[test]
    fn test_iterator() {
        let mut csr = SingleImmutableCsr::with_capacity(10);

        csr.batch_put_edges(
            &[0, 2, 4].map(VertexId::from_int64),
            &[10, 30, 50].map(VertexId::from_int64),
            &[100, 102, 104],
            &[0, 2, 4],
        );

        let edges: Vec<_> = csr.iter().collect();
        assert_eq!(edges.len(), 3);
    }
}
