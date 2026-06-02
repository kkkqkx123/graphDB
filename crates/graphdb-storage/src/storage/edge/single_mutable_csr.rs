//! Single Mutable CSR Implementation
//!
//! Optimized CSR for scenarios where each vertex has at most one outgoing edge.
//! Uses a simple array instead of offset/degree arrays, providing O(1) access.
//!
//! Use cases:
//! - "Spouse" relationship (one-to-one)
//! - "Current employer" relationship
//! - Any single-edge semantic relationship

use std::sync::atomic::{AtomicU64, Ordering};

use crate::core::{StorageError, StorageResult};
use crate::storage::utils::{read_u32_le, read_u64_le};

use super::{
    CsrBase, CsrType, EdgeId, MutableCsrTrait, Nbr, Timestamp, VertexId, INVALID_EDGE_ID,
    INVALID_TIMESTAMP,
};

fn write_vertex_id(out: &mut Vec<u8>, id: VertexId) {
    let bytes = id.as_bytes();
    out.push(bytes.len() as u8);
    out.extend_from_slice(bytes);
}

fn read_vertex_id(data: &[u8], offset: &mut usize) -> StorageResult<VertexId> {
    if *offset >= data.len() {
        return Err(StorageError::deserialize_error(
            "Single CSR data too short for vertex id length",
        ));
    }

    let len = data[*offset] as usize;
    *offset += 1;
    if data.len().saturating_sub(*offset) < len {
        return Err(StorageError::deserialize_error(
            "Single CSR data too short for vertex id bytes",
        ));
    }

    let id = VertexId::from_bytes(data[*offset..*offset + len].to_vec());
    *offset += len;
    Ok(id)
}

const DEFAULT_VERTEX_CAPACITY: usize = 1024;

pub struct SingleMutableCsr {
    nbr_list: Vec<Nbr>,
    edge_count: AtomicU64,
    vertex_capacity: usize,
}

impl Clone for SingleMutableCsr {
    fn clone(&self) -> Self {
        Self {
            nbr_list: self.nbr_list.clone(),
            edge_count: AtomicU64::new(self.edge_count.load(Ordering::Relaxed)),
            vertex_capacity: self.vertex_capacity,
        }
    }
}

impl std::fmt::Debug for SingleMutableCsr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SingleMutableCsr")
            .field("vertex_capacity", &self.vertex_capacity)
            .field("edge_count", &self.edge_count.load(Ordering::Relaxed))
            .finish_non_exhaustive()
    }
}

impl SingleMutableCsr {
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_VERTEX_CAPACITY)
    }

    pub fn with_capacity(vertex_capacity: usize) -> Self {
        let vertex_cap = vertex_capacity.max(1);
        let nbr_list = vec![
            Nbr::new(
                VertexId::from_int64(0),
                INVALID_EDGE_ID,
                0,
                INVALID_TIMESTAMP
            );
            vertex_cap
        ];

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
        if new_vertex_capacity <= self.vertex_capacity {
            return;
        }

        let additional = new_vertex_capacity - self.vertex_capacity;
        self.nbr_list.extend(std::iter::repeat_n(
            Nbr::new(
                VertexId::from_int64(0),
                INVALID_EDGE_ID,
                0,
                INVALID_TIMESTAMP,
            ),
            additional,
        ));
        self.vertex_capacity = new_vertex_capacity;
    }

    pub fn ensure_vertex_capacity(&mut self, min_capacity: usize) {
        if min_capacity > self.vertex_capacity {
            let new_capacity = min_capacity.next_power_of_two();
            self.resize(new_capacity);
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
        let src_idx = src.as_int64().unwrap_or(0) as usize;

        if src_idx >= self.vertex_capacity {
            self.ensure_vertex_capacity(src_idx + 1);
        }

        let nbr = &mut self.nbr_list[src_idx];

        if nbr.timestamp != INVALID_TIMESTAMP && ts <= nbr.timestamp {
            return false;
        }

        let was_deleted = nbr.timestamp == INVALID_TIMESTAMP;
        nbr.neighbor = dst;
        nbr.edge_id = edge_id;
        nbr.prop_offset = prop_offset;
        nbr.timestamp = ts;

        if was_deleted {
            self.edge_count.fetch_add(1, Ordering::Relaxed);
        }

        true
    }

    pub fn update_edge(
        &mut self,
        src: VertexId,
        dst: VertexId,
        edge_id: EdgeId,
        prop_offset: u32,
        ts: Timestamp,
    ) -> bool {
        let src_idx = src.as_int64().unwrap_or(0) as usize;

        if src_idx >= self.vertex_capacity {
            return false;
        }

        let nbr = &mut self.nbr_list[src_idx];

        if nbr.timestamp == INVALID_TIMESTAMP {
            return false;
        }

        if nbr.neighbor != dst {
            return false;
        }

        if nbr.timestamp > ts {
            return false;
        }

        nbr.edge_id = edge_id;
        nbr.prop_offset = prop_offset;
        nbr.timestamp = ts;

        true
    }

    pub fn delete_edge(&mut self, src: VertexId, ts: Timestamp) -> bool {
        let src_idx = src.as_int64().unwrap_or(0) as usize;

        if src_idx >= self.vertex_capacity {
            return false;
        }

        let nbr = &mut self.nbr_list[src_idx];

        if nbr.timestamp == INVALID_TIMESTAMP || nbr.timestamp > ts {
            return false;
        }

        nbr.timestamp = INVALID_TIMESTAMP;
        self.edge_count.fetch_sub(1, Ordering::Relaxed);
        true
    }

    pub fn delete_edge_by_id(&mut self, src: VertexId, _edge_id: EdgeId, ts: Timestamp) -> bool {
        self.delete_edge(src, ts)
    }

    pub fn delete_edge_by_dst(&mut self, src: VertexId, dst: VertexId, ts: Timestamp) -> bool {
        let src_idx = src.as_int64().unwrap_or(0) as usize;

        if src_idx >= self.vertex_capacity {
            return false;
        }

        let nbr = &mut self.nbr_list[src_idx];

        if nbr.neighbor != dst || nbr.timestamp == INVALID_TIMESTAMP || nbr.timestamp > ts {
            return false;
        }

        nbr.timestamp = INVALID_TIMESTAMP;
        self.edge_count.fetch_sub(1, Ordering::Relaxed);
        true
    }

    pub fn delete_edge_by_offset(&mut self, src: VertexId, _offset: i32, ts: Timestamp) -> bool {
        if _offset != 0 {
            return false;
        }
        self.delete_edge(src, ts)
    }

    pub fn revert_delete(&mut self, src: VertexId, dst: VertexId, ts: Timestamp) -> bool {
        let src_idx = src.as_int64().unwrap_or(0) as usize;

        if src_idx >= self.vertex_capacity {
            return false;
        }

        let nbr = &mut self.nbr_list[src_idx];

        if nbr.timestamp != INVALID_TIMESTAMP {
            return false;
        }

        nbr.neighbor = dst;
        nbr.timestamp = ts;
        self.edge_count.fetch_add(1, Ordering::Relaxed);
        true
    }

    pub fn revert_delete_by_offset(&mut self, src: VertexId, _offset: i32, ts: Timestamp) -> bool {
        if _offset != 0 {
            return false;
        }

        let src_idx = src.as_int64().unwrap_or(0) as usize;

        if src_idx >= self.vertex_capacity {
            return false;
        }

        let nbr = &mut self.nbr_list[src_idx];

        if nbr.timestamp != INVALID_TIMESTAMP {
            return false;
        }

        nbr.timestamp = ts;
        self.edge_count.fetch_add(1, Ordering::Relaxed);
        true
    }

    pub fn revert_delete_by_id(&mut self, src: VertexId, edge_id: EdgeId, ts: Timestamp) -> bool {
        let src_idx = src.as_int64().unwrap_or(0) as usize;
        if src_idx >= self.vertex_capacity {
            return false;
        }

        let nbr = &mut self.nbr_list[src_idx];
        if nbr.timestamp != INVALID_TIMESTAMP || nbr.edge_id != edge_id {
            return false;
        }

        nbr.timestamp = ts;
        self.edge_count.fetch_add(1, Ordering::Relaxed);
        true
    }

    pub fn get_edge(&self, src: VertexId, ts: Timestamp) -> Option<Nbr> {
        let src_idx = src.as_int64().unwrap_or(0) as usize;

        if src_idx >= self.vertex_capacity {
            return None;
        }

        let nbr = &self.nbr_list[src_idx];

        if nbr.timestamp == INVALID_TIMESTAMP || nbr.timestamp > ts {
            return None;
        }

        Some(*nbr)
    }

    pub fn get_edge_by_dst(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> Option<Nbr> {
        let edge = self.get_edge(src, ts)?;
        if edge.neighbor == dst {
            Some(edge)
        } else {
            None
        }
    }

    pub fn edges_of(&self, src: VertexId, ts: Timestamp) -> Vec<Nbr> {
        self.get_edge(src, ts).map_or(Vec::new(), |nbr| vec![nbr])
    }

    pub fn degree(&self, src: VertexId, ts: Timestamp) -> usize {
        if self.get_edge(src, ts).is_some() {
            1
        } else {
            0
        }
    }

    pub fn has_edge(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> bool {
        self.get_edge_by_dst(src, dst, ts).is_some()
    }

    pub fn find_deleted_edge(&self, src: VertexId, dst: VertexId) -> Option<EdgeId> {
        let src_idx = src.as_int64().unwrap_or(0) as usize;

        if src_idx >= self.vertex_capacity {
            return None;
        }

        let nbr = &self.nbr_list[src_idx];

        if nbr.timestamp == INVALID_TIMESTAMP && nbr.neighbor == dst {
            Some(nbr.edge_id)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        for nbr in &mut self.nbr_list {
            *nbr = Nbr::new(
                VertexId::from_int64(0),
                INVALID_EDGE_ID,
                0,
                INVALID_TIMESTAMP,
            );
        }
        self.edge_count.store(0, Ordering::Relaxed);
    }

    pub fn compact(&mut self) {
        // No-op for single CSR - no tombstones to compact
        // Deleted edges are already marked with INVALID_TIMESTAMP
    }

    pub fn compact_with_ts(&mut self, _ts: Timestamp, _reserve_ratio: f32) -> usize {
        // No-op for single CSR - no tombstones to compact
        // Returns 0 as no edges are removed
        0
    }

    pub fn batch_put_edges(
        &mut self,
        src_list: &[VertexId],
        dst_list: &[VertexId],
        edge_ids: &[EdgeId],
        prop_offsets: &[u32],
        ts: Timestamp,
    ) {
        let max_vertex = src_list
            .iter()
            .max()
            .cloned()
            .unwrap_or(VertexId::zero())
            .as_int64()
            .unwrap_or(0) as usize;
        self.ensure_vertex_capacity(max_vertex + 1);

        for i in 0..src_list.len() {
            let src_idx = src_list[i].as_int64().unwrap_or(0) as usize;
            let nbr = &mut self.nbr_list[src_idx];

            if nbr.timestamp == INVALID_TIMESTAMP {
                self.edge_count.fetch_add(1, Ordering::Relaxed);
            }

            nbr.neighbor = dst_list[i];
            nbr.edge_id = edge_ids[i];
            nbr.prop_offset = prop_offsets[i];
            nbr.timestamp = ts;
        }
    }

    pub fn dump(&self) -> Vec<u8> {
        let mut result = Vec::new();

        result.extend_from_slice(&(self.vertex_capacity as u64).to_le_bytes());
        result.extend_from_slice(&self.edge_count.load(Ordering::Relaxed).to_le_bytes());

        for nbr in &self.nbr_list {
            write_vertex_id(&mut result, nbr.neighbor);
            result.extend_from_slice(&nbr.edge_id.to_le_bytes());
            result.extend_from_slice(&nbr.prop_offset.to_le_bytes());
            result.extend_from_slice(&nbr.timestamp.to_le_bytes());
        }

        result
    }

    pub fn used_memory_size(&self) -> usize {
        self.nbr_list.len() * std::mem::size_of::<Nbr>() + std::mem::size_of::<Self>()
    }

    pub fn load(&mut self, data: &[u8]) -> StorageResult<()> {
        if data.len() < 16 {
            return Err(StorageError::deserialize_error(
                "Single CSR data too short for header",
            ));
        }

        let mut offset = 0usize;

        let vertex_capacity = read_u64_le(data, &mut offset)? as usize;
        let edge_count = read_u64_le(data, &mut offset)?;

        let mut nbr_list = Vec::with_capacity(vertex_capacity);
        for _ in 0..vertex_capacity {
            let neighbor = read_vertex_id(data, &mut offset)?;
            let edge_id = read_u64_le(data, &mut offset)?;
            let prop_offset = read_u32_le(data, &mut offset)?;
            let timestamp = read_u32_le(data, &mut offset)?;

            nbr_list.push(Nbr::new(neighbor, edge_id, prop_offset, timestamp));
        }

        self.vertex_capacity = vertex_capacity;
        self.nbr_list = nbr_list;
        self.edge_count.store(edge_count, Ordering::Relaxed);

        Ok(())
    }

    pub fn iter(&self, ts: Timestamp) -> SingleMutableCsrIterator<'_> {
        SingleMutableCsrIterator::new(self, ts)
    }

    pub fn iter_edges(&self, src: VertexId, ts: Timestamp) -> SingleCsrEdgeIterator<'_> {
        SingleCsrEdgeIterator::new(self, src, ts)
    }
}

pub struct SingleCsrEdgeIterator<'a> {
    csr: &'a SingleMutableCsr,
    src: VertexId,
    ts: Timestamp,
    consumed: bool,
}

impl<'a> SingleCsrEdgeIterator<'a> {
    pub fn new(csr: &'a SingleMutableCsr, src: VertexId, ts: Timestamp) -> Self {
        Self {
            csr,
            src,
            ts,
            consumed: false,
        }
    }
}

impl<'a> Iterator for SingleCsrEdgeIterator<'a> {
    type Item = Nbr;

    fn next(&mut self) -> Option<Self::Item> {
        if self.consumed {
            return None;
        }

        self.consumed = true;
        self.csr.get_edge(self.src, self.ts)
    }
}

impl Default for SingleMutableCsr {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SingleMutableCsrIterator<'a> {
    csr: &'a SingleMutableCsr,
    current_vertex: usize,
    ts: Timestamp,
}

impl<'a> SingleMutableCsrIterator<'a> {
    pub fn new(csr: &'a SingleMutableCsr, ts: Timestamp) -> Self {
        Self {
            csr,
            current_vertex: 0,
            ts,
        }
    }
}

impl<'a> Iterator for SingleMutableCsrIterator<'a> {
    type Item = (VertexId, Nbr);

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_vertex < self.csr.vertex_capacity {
            let src = VertexId::from_int64(self.current_vertex as i64);
            self.current_vertex += 1;

            if let Some(nbr) = self.csr.get_edge(src, self.ts) {
                return Some((src, nbr));
            }
        }
        None
    }
}

impl CsrBase for SingleMutableCsr {
    fn vertex_capacity(&self) -> usize {
        self.vertex_capacity
    }

    fn edge_count(&self) -> u64 {
        self.edge_count.load(Ordering::Relaxed)
    }

    fn csr_type(&self) -> CsrType {
        CsrType::SingleMutable
    }

    fn dump(&self) -> Vec<u8> {
        SingleMutableCsr::dump(self)
    }

    fn load(&mut self, data: &[u8]) -> StorageResult<()> {
        SingleMutableCsr::load(self, data)
    }
}

impl MutableCsrTrait for SingleMutableCsr {
    fn insert_edge(
        &mut self,
        src: VertexId,
        dst: VertexId,
        edge_id: EdgeId,
        prop_offset: u32,
        ts: Timestamp,
    ) -> bool {
        SingleMutableCsr::insert_edge(self, src, dst, edge_id, prop_offset, ts)
    }

    fn delete_edge(&mut self, src: VertexId, _edge_id: EdgeId, ts: Timestamp) -> bool {
        SingleMutableCsr::delete_edge(self, src, ts)
    }

    fn delete_edge_by_dst(&mut self, src: VertexId, dst: VertexId, ts: Timestamp) -> bool {
        SingleMutableCsr::delete_edge_by_dst(self, src, dst, ts)
    }

    fn delete_edge_by_offset(&mut self, src: VertexId, offset: i32, ts: Timestamp) -> bool {
        SingleMutableCsr::delete_edge_by_offset(self, src, offset, ts)
    }

    fn revert_delete_by_offset(&mut self, src: VertexId, offset: i32, ts: Timestamp) -> bool {
        SingleMutableCsr::revert_delete_by_offset(self, src, offset, ts)
    }

    fn get_edge(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> Option<Nbr> {
        SingleMutableCsr::get_edge_by_dst(self, src, dst, ts)
    }

    fn edges_of(&self, src: VertexId, ts: Timestamp) -> Vec<Nbr> {
        SingleMutableCsr::edges_of(self, src, ts)
    }

    fn degree(&self, src: VertexId, ts: Timestamp) -> usize {
        SingleMutableCsr::degree(self, src, ts)
    }

    fn has_edge(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> bool {
        SingleMutableCsr::has_edge(self, src, dst, ts)
    }

    fn compact(&mut self) {
        SingleMutableCsr::compact(self);
    }

    fn compact_with_ts(&mut self, ts: Timestamp, reserve_ratio: f32) -> usize {
        SingleMutableCsr::compact_with_ts(self, ts, reserve_ratio)
    }

    fn find_deleted_edge(&self, src: VertexId, dst: VertexId) -> Option<EdgeId> {
        SingleMutableCsr::find_deleted_edge(self, src, dst)
    }

    fn used_memory_size(&self) -> usize {
        SingleMutableCsr::used_memory_size(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut csr = SingleMutableCsr::with_capacity(10);

        assert!(csr.insert_edge(
            VertexId::from_int64(0),
            VertexId::from_int64(1),
            100,
            0,
            100
        ));
        assert!(!csr.insert_edge(VertexId::from_int64(0), VertexId::from_int64(2), 101, 1, 99));
        assert!(csr.insert_edge(
            VertexId::from_int64(0),
            VertexId::from_int64(2),
            102,
            1,
            101
        ));

        assert_eq!(csr.edge_count(), 1);
        assert!(csr.has_edge(VertexId::from_int64(0), VertexId::from_int64(2), 150));
        assert!(!csr.has_edge(VertexId::from_int64(0), VertexId::from_int64(1), 150));
    }

    #[test]
    fn test_delete_and_revert() {
        let mut csr = SingleMutableCsr::with_capacity(10);

        csr.insert_edge(
            VertexId::from_int64(0),
            VertexId::from_int64(1),
            100,
            0,
            100,
        );
        assert_eq!(csr.edge_count(), 1);

        assert!(csr.delete_edge(VertexId::from_int64(0), 150));
        assert_eq!(csr.edge_count(), 0);

        assert!(csr.revert_delete(VertexId::from_int64(0), VertexId::from_int64(1), 100));
        assert_eq!(csr.edge_count(), 1);
    }

    #[test]
    fn test_batch_put() {
        let mut csr = SingleMutableCsr::new();

        csr.batch_put_edges(
            &[0, 1, 2].map(VertexId::from_int64),
            &[10, 20, 30].map(VertexId::from_int64),
            &[100, 101, 102],
            &[0, 1, 2],
            100,
        );

        assert_eq!(csr.edge_count(), 3);
        assert!(csr.has_edge(VertexId::from_int64(0), VertexId::from_int64(10), 100));
        assert!(csr.has_edge(VertexId::from_int64(1), VertexId::from_int64(20), 100));
        assert!(csr.has_edge(VertexId::from_int64(2), VertexId::from_int64(30), 100));
    }

    #[test]
    fn test_dump_and_load() {
        let mut csr1 = SingleMutableCsr::with_capacity(10);

        csr1.batch_put_edges(
            &[0, 1, 2].map(VertexId::from_int64),
            &[10, 20, 30].map(VertexId::from_int64),
            &[100, 101, 102],
            &[0, 1, 2],
            100,
        );

        let data = csr1.dump();

        let mut csr2 = SingleMutableCsr::new();
        let _ = csr2.load(&data);

        assert_eq!(csr2.vertex_capacity(), csr1.vertex_capacity());
        assert_eq!(csr2.edge_count(), csr1.edge_count());
        assert!(csr2.has_edge(VertexId::from_int64(0), VertexId::from_int64(10), 100));
        assert!(csr2.has_edge(VertexId::from_int64(1), VertexId::from_int64(20), 100));
    }
}
