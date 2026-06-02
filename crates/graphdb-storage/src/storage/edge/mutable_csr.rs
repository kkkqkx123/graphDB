//! Mutable CSR Implementation
//!
//! Two-level CSR with append-only overflow for O(1) amortized vertex expansion.
//! Primary blocks are stored contiguously in `nbr_list` (flat CSR layout).
//! Overflow edges are stored in a per-vertex `SmallVec<[Nbr; OVERFLOW_INLINE]>` for
//! cache-friendly iteration. When a vertex's primary block is full, new edges
//! spill to its overflow buffer, avoiding O(n) splice on the main array.

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::core::{StorageError, StorageResult};
use crate::storage::utils::{read_u32_le, read_u64_le};

use super::{
    CsrBase, EdgeId, MutableCsrTrait, Nbr, Timestamp, VertexId, INVALID_TIMESTAMP,
};

fn write_vertex_id(out: &mut Vec<u8>, id: VertexId) {
    let bytes = id.as_bytes();
    out.push(bytes.len() as u8);
    out.extend_from_slice(bytes);
}

fn read_vertex_id(data: &[u8], offset: &mut usize) -> StorageResult<VertexId> {
    if *offset >= data.len() {
        return Err(StorageError::deserialize_error(
            "CSR data too short for vertex id length",
        ));
    }

    let len = data[*offset] as usize;
    *offset += 1;
    if data.len().saturating_sub(*offset) < len {
        return Err(StorageError::deserialize_error(
            "CSR data too short for vertex id bytes",
        ));
    }

    let id = VertexId::from_bytes(data[*offset..*offset + len].to_vec());
    *offset += len;
    Ok(id)
}

const DEFAULT_VERTEX_CAPACITY: usize = 1024;
const DEFAULT_EDGE_CAPACITY: usize = 4096;
const DEFAULT_VERTEX_DEGREE: usize = 4;
const NO_OVERFLOW: u32 = u32::MAX;

/// Mutable CSR graph structure with two-level storage.
///
/// # Layout
///
/// Each vertex has:
/// - **Primary block**: contiguous slot in `nbr_list` (size = `primary_capacities[src_idx]`),
///   starting at `adj_offsets[src_idx]`. Active edges: `degrees[src_idx]`.
/// - **Overflow block**: contiguous region in `nbr_list` for edges beyond primary capacity,
///   stored as append-only blocks at the end of `nbr_list`.
///
/// When primary fills (`degrees == primary_capacities`), new edges go to overflow.
/// Overflow blocks are allocated via `expand_vertex_capacity()` which appends to `nbr_list`,
/// avoiding O(n) splice on the main array.
///
/// `compact()` merges overflow back into primary, restoring flat CSR layout.
pub struct MutableCsr {
    nbr_list: Vec<Nbr>,
    adj_offsets: Vec<u32>,
    degrees: Vec<u32>,
    primary_capacities: Vec<u32>,

    overflow_starts: Vec<u32>,
    overflow_counts: Vec<u32>,
    overflow_capacities: Vec<u32>,

    edge_count: AtomicU64,
    vertex_capacity: usize,
    total_edge_capacity: usize,
}

impl Clone for MutableCsr {
    fn clone(&self) -> Self {
        Self {
            nbr_list: self.nbr_list.clone(),
            adj_offsets: self.adj_offsets.clone(),
            degrees: self.degrees.clone(),
            primary_capacities: self.primary_capacities.clone(),
            overflow_starts: self.overflow_starts.clone(),
            overflow_counts: self.overflow_counts.clone(),
            overflow_capacities: self.overflow_capacities.clone(),
            edge_count: AtomicU64::new(self.edge_count.load(Ordering::Relaxed)),
            vertex_capacity: self.vertex_capacity,
            total_edge_capacity: self.total_edge_capacity,
        }
    }
}

impl fmt::Debug for MutableCsr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MutableCsr")
            .field("vertex_capacity", &self.vertex_capacity)
            .field("total_edge_capacity", &self.total_edge_capacity)
            .field("edge_count", &self.edge_count.load(Ordering::Relaxed))
            .finish_non_exhaustive()
    }
}

impl MutableCsr {
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_VERTEX_CAPACITY, DEFAULT_EDGE_CAPACITY)
    }

    pub fn with_capacity(vertex_capacity: usize, edge_capacity: usize) -> Self {
        let vertex_cap = vertex_capacity.max(1);
        let edge_cap = edge_capacity.max(vertex_cap * DEFAULT_VERTEX_DEGREE);

        let mut nbr_list = Vec::with_capacity(edge_cap);
        let mut adj_offsets = Vec::with_capacity(vertex_cap);
        let mut primary_capacities = Vec::with_capacity(vertex_cap);

        let mut offset = 0usize;
        for _ in 0..vertex_cap {
            adj_offsets.push(offset as u32);
            primary_capacities.push(DEFAULT_VERTEX_DEGREE as u32);
            offset += DEFAULT_VERTEX_DEGREE;
        }

        nbr_list.resize(
            offset,
            Nbr::new(VertexId::from_int64(0), 0, 0, INVALID_TIMESTAMP),
        );

        Self {
            nbr_list,
            adj_offsets,
            degrees: vec![0; vertex_cap],
            primary_capacities,
            overflow_starts: vec![NO_OVERFLOW; vertex_cap],
            overflow_counts: vec![0; vertex_cap],
            overflow_capacities: vec![0; vertex_cap],
            edge_count: AtomicU64::new(0),
            vertex_capacity: vertex_cap,
            total_edge_capacity: offset,
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

    /// Resize vertex capacity (requires exclusive access)
    pub fn resize(&mut self, new_vertex_capacity: usize) {
        if new_vertex_capacity <= self.vertex_capacity {
            return;
        }

        let old_capacity = self.vertex_capacity;
        let additional = new_vertex_capacity - old_capacity;

        let mut new_total_capacity = self.total_edge_capacity;
        for _ in 0..additional {
            self.adj_offsets.push(new_total_capacity as u32);
            self.primary_capacities.push(DEFAULT_VERTEX_DEGREE as u32);
            self.degrees.push(0);
            self.overflow_starts.push(NO_OVERFLOW);
            self.overflow_counts.push(0);
            self.overflow_capacities.push(0);
            new_total_capacity += DEFAULT_VERTEX_DEGREE;
        }

        self.nbr_list.resize(
            new_total_capacity,
            Nbr::new(VertexId::from_int64(0), 0, 0, INVALID_TIMESTAMP),
        );
        self.vertex_capacity = new_vertex_capacity;
        self.total_edge_capacity = new_total_capacity;
    }

    /// Ensure vertex capacity (grows if needed)
    pub fn ensure_vertex_capacity(&mut self, min_capacity: usize) {
        if min_capacity > self.vertex_capacity {
            let new_capacity = min_capacity.next_power_of_two();
            self.resize(new_capacity);
        }
    }

    /// Expand vertex capacity by appending overflow block at end of nbr_list.
    /// Copies existing overflow data to the new block if re-expanding.
    fn expand_vertex_capacity(&mut self, src_idx: usize) {
        let old_cap = self.primary_capacities[src_idx] as usize;
        let new_cap = (old_cap * 2).max(4);
        let additional = new_cap - old_cap;

        let append_pos = self.nbr_list.len();
        self.nbr_list.resize(
            append_pos + additional,
            Nbr::new(VertexId::from_int64(0), 0, 0, INVALID_TIMESTAMP),
        );

        // Copy existing overflow data to new block if re-expanding
        if self.overflow_starts[src_idx] != NO_OVERFLOW {
            let old_start = self.overflow_starts[src_idx] as usize;
            let old_count = self.overflow_counts[src_idx] as usize;
            for i in 0..old_count {
                self.nbr_list[append_pos + i] = self.nbr_list[old_start + i];
            }
        }

        self.overflow_starts[src_idx] = append_pos as u32;
        self.overflow_capacities[src_idx] = additional as u32;
        self.primary_capacities[src_idx] = new_cap as u32;
        self.total_edge_capacity += additional;
    }

    /// Insert an edge with automatic capacity expansion
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

        // Duplicate check across both primary and overflow
        let degree = self.degrees[src_idx] as usize;
        let base = self.adj_offsets[src_idx] as usize;
        for i in 0..degree {
            let nbr = &self.nbr_list[base + i];
            if nbr.neighbor == dst && nbr.timestamp != INVALID_TIMESTAMP {
                return false;
            }
        }
        if self.overflow_starts[src_idx] != NO_OVERFLOW {
            let o_start = self.overflow_starts[src_idx] as usize;
            let o_count = self.overflow_counts[src_idx] as usize;
            for i in 0..o_count {
                let nbr = &self.nbr_list[o_start + i];
                if nbr.neighbor == dst && nbr.timestamp != INVALID_TIMESTAMP {
                    return false;
                }
            }
        }

        // Write to primary if space available and overflow not yet allocated
        if self.overflow_starts[src_idx] == NO_OVERFLOW
            && degree < self.primary_capacities[src_idx] as usize
        {
            self.nbr_list[base + degree] = Nbr::new(dst, edge_id, prop_offset, ts);
            self.degrees[src_idx] += 1;
            self.edge_count.fetch_add(1, Ordering::Relaxed);
            return true;
        }

        // Write to overflow, expanding if needed
        if self.overflow_starts[src_idx] == NO_OVERFLOW
            || self.overflow_counts[src_idx] >= self.overflow_capacities[src_idx]
        {
            self.expand_vertex_capacity(src_idx);
        }
        let o_start = self.overflow_starts[src_idx] as usize;
        let o_count = self.overflow_counts[src_idx] as usize;
        self.nbr_list[o_start + o_count] = Nbr::new(dst, edge_id, prop_offset, ts);
        self.overflow_counts[src_idx] += 1;
        self.edge_count.fetch_add(1, Ordering::Relaxed);
        true
    }

    fn scan_overflow_for_edge_id(&self, src_idx: usize, edge_id: EdgeId) -> Option<usize> {
        if self.overflow_starts[src_idx] == NO_OVERFLOW {
            return None;
        }
        let o_start = self.overflow_starts[src_idx] as usize;
        let o_count = self.overflow_counts[src_idx] as usize;
        (0..o_count).find(|&i| self.nbr_list[o_start + i].edge_id == edge_id)
    }

    fn scan_overflow_for_dst(&self, src_idx: usize, dst: VertexId) -> Vec<usize> {
        if self.overflow_starts[src_idx] == NO_OVERFLOW {
            return Vec::new();
        }
        let o_start = self.overflow_starts[src_idx] as usize;
        let o_count = self.overflow_counts[src_idx] as usize;
        let mut result = Vec::new();
        for i in 0..o_count {
            if self.nbr_list[o_start + i].neighbor == dst {
                result.push(i);
            }
        }
        result
    }

    /// Delete an edge by edge_id
    pub fn delete_edge(&mut self, src: VertexId, edge_id: EdgeId, ts: Timestamp) -> bool {
        let src_idx = src.as_int64().unwrap_or(0) as usize;
        if src_idx >= self.vertex_capacity {
            return false;
        }

        // Scan primary
        let degree = self.degrees[src_idx] as usize;
        let offset = self.adj_offsets[src_idx] as usize;
        for i in 0..degree {
            let nbr = &mut self.nbr_list[offset + i];
            if nbr.edge_id == edge_id && nbr.timestamp != INVALID_TIMESTAMP && nbr.timestamp <= ts {
                nbr.timestamp = INVALID_TIMESTAMP;
                self.edge_count.fetch_sub(1, Ordering::Relaxed);
                return true;
            }
        }

        // Scan overflow
        if let Some(idx) = self.scan_overflow_for_edge_id(src_idx, edge_id) {
            let o_start = self.overflow_starts[src_idx] as usize;
            let nbr = &mut self.nbr_list[o_start + idx];
            if nbr.timestamp != INVALID_TIMESTAMP && nbr.timestamp <= ts {
                nbr.timestamp = INVALID_TIMESTAMP;
                self.edge_count.fetch_sub(1, Ordering::Relaxed);
                return true;
            }
        }

        false
    }

    /// Delete edge by destination vertex
    pub fn delete_edge_by_dst(&mut self, src: VertexId, dst: VertexId, ts: Timestamp) -> bool {
        let src_idx = src.as_int64().unwrap_or(0) as usize;
        if src_idx >= self.vertex_capacity {
            return false;
        }

        let mut deleted = false;

        // Scan primary
        let degree = self.degrees[src_idx] as usize;
        let offset = self.adj_offsets[src_idx] as usize;
        for i in 0..degree {
            let nbr = &mut self.nbr_list[offset + i];
            if nbr.neighbor == dst && nbr.timestamp != INVALID_TIMESTAMP && nbr.timestamp <= ts {
                nbr.timestamp = INVALID_TIMESTAMP;
                self.edge_count.fetch_sub(1, Ordering::Relaxed);
                deleted = true;
            }
        }

        // Scan overflow
        let indices = self.scan_overflow_for_dst(src_idx, dst);
        if self.overflow_starts[src_idx] != NO_OVERFLOW {
            let o_start = self.overflow_starts[src_idx] as usize;
            for idx in indices {
                let nbr = &mut self.nbr_list[o_start + idx];
                if nbr.timestamp != INVALID_TIMESTAMP && nbr.timestamp <= ts {
                    nbr.timestamp = INVALID_TIMESTAMP;
                    self.edge_count.fetch_sub(1, Ordering::Relaxed);
                    deleted = true;
                }
            }
        }

        deleted
    }

    /// Delete an edge by offset position in the CSR primary block
    pub fn delete_edge_by_offset(&mut self, src: VertexId, offset: i32, ts: Timestamp) -> bool {
        let src_idx = src.as_int64().unwrap_or(0) as usize;
        if src_idx >= self.vertex_capacity {
            return false;
        }

        let base_offset = self.adj_offsets[src_idx] as usize;
        let idx = base_offset + offset as usize;

        if idx >= self.nbr_list.len() {
            return false;
        }

        let nbr = &mut self.nbr_list[idx];
        if nbr.timestamp != INVALID_TIMESTAMP && nbr.timestamp <= ts {
            nbr.timestamp = INVALID_TIMESTAMP;
            self.edge_count.fetch_sub(1, Ordering::Relaxed);
            return true;
        }
        false
    }

    /// Revert a deleted edge by offset position in the primary block
    pub fn revert_delete_by_offset(&mut self, src: VertexId, offset: i32, ts: Timestamp) -> bool {
        let src_idx = src.as_int64().unwrap_or(0) as usize;
        if src_idx >= self.vertex_capacity {
            return false;
        }

        let base_offset = self.adj_offsets[src_idx] as usize;
        let idx = base_offset + offset as usize;

        if idx >= self.nbr_list.len() {
            return false;
        }

        let nbr = &mut self.nbr_list[idx];
        if nbr.timestamp == INVALID_TIMESTAMP {
            nbr.timestamp = ts;
            self.edge_count.fetch_add(1, Ordering::Relaxed);
            return true;
        }
        false
    }

    /// Find a deleted edge by destination
    pub fn find_deleted_edge(&self, src: VertexId, dst: VertexId) -> Option<EdgeId> {
        let src_idx = src.as_int64().unwrap_or(0) as usize;
        if src_idx >= self.vertex_capacity {
            return None;
        }

        // Scan primary
        let degree = self.degrees[src_idx] as usize;
        let offset = self.adj_offsets[src_idx] as usize;
        for i in 0..degree {
            let nbr = &self.nbr_list[offset + i];
            if nbr.neighbor == dst && nbr.timestamp == INVALID_TIMESTAMP {
                return Some(nbr.edge_id);
            }
        }

        // Scan overflow
        if self.overflow_starts[src_idx] != NO_OVERFLOW {
            let o_start = self.overflow_starts[src_idx] as usize;
            let o_count = self.overflow_counts[src_idx] as usize;
            for i in 0..o_count {
                let nbr = &self.nbr_list[o_start + i];
                if nbr.neighbor == dst && nbr.timestamp == INVALID_TIMESTAMP {
                    return Some(nbr.edge_id);
                }
            }
        }

        None
    }

    /// Get edges of a vertex at a given timestamp
    pub fn edges_of(&self, src: VertexId, ts: Timestamp) -> Vec<Nbr> {
        let src_idx = src.as_int64().unwrap_or(0) as usize;
        if src_idx >= self.vertex_capacity {
            return Vec::new();
        }

        let degree = self.degrees[src_idx] as usize;
        let offset = self.adj_offsets[src_idx] as usize;

        let total_valid_primary = self.count_valid_primary(src_idx, ts);
        let total_valid_overflow = self.count_valid_overflow(src_idx, ts);
        let mut result = Vec::with_capacity(total_valid_primary + total_valid_overflow);

        for i in 0..degree {
            let nbr = &self.nbr_list[offset + i];
            if nbr.timestamp <= ts && nbr.timestamp != INVALID_TIMESTAMP {
                result.push(*nbr);
            }
        }

        if self.overflow_starts[src_idx] != NO_OVERFLOW {
            let o_start = self.overflow_starts[src_idx] as usize;
            let o_count = self.overflow_counts[src_idx] as usize;
            for i in 0..o_count {
                let nbr = &self.nbr_list[o_start + i];
                if nbr.timestamp <= ts && nbr.timestamp != INVALID_TIMESTAMP {
                    result.push(*nbr);
                }
            }
        }

        result
    }

    /// Get edges of a vertex with prefetch optimization
    #[cfg(target_arch = "x86_64")]
    pub fn edges_of_with_prefetch(&self, src: VertexId, ts: Timestamp) -> Vec<Nbr> {
        use std::arch::x86_64::_mm_prefetch;
        use std::arch::x86_64::_MM_HINT_T0;

        let src_idx = src.as_int64().unwrap_or(0) as usize;
        if src_idx >= self.vertex_capacity {
            return Vec::new();
        }

        let degree = self.degrees[src_idx] as usize;
        let offset = self.adj_offsets[src_idx] as usize;

        let total_valid_primary = self.count_valid_primary(src_idx, ts);
        let total_valid_overflow = self.count_valid_overflow(src_idx, ts);
        let mut result = Vec::with_capacity(total_valid_primary + total_valid_overflow);

        const PREFETCH_DISTANCE: usize = 8;

        for i in 0..degree {
            if i + PREFETCH_DISTANCE < degree {
                let prefetch_idx = offset + i + PREFETCH_DISTANCE;
                if prefetch_idx < self.nbr_list.len() {
                    unsafe {
                        _mm_prefetch(
                            &self.nbr_list[prefetch_idx] as *const Nbr as *const i8,
                            _MM_HINT_T0,
                        );
                    }
                }
            }

            let nbr = &self.nbr_list[offset + i];
            if nbr.timestamp <= ts && nbr.timestamp != INVALID_TIMESTAMP {
                result.push(*nbr);
            }
        }

        if self.overflow_starts[src_idx] != NO_OVERFLOW {
            let o_start = self.overflow_starts[src_idx] as usize;
            let o_count = self.overflow_counts[src_idx] as usize;
            for i in 0..o_count {
                if i + PREFETCH_DISTANCE < o_count {
                    unsafe {
                        _mm_prefetch(
                            &self.nbr_list[o_start + i + PREFETCH_DISTANCE] as *const Nbr
                                as *const i8,
                            _MM_HINT_T0,
                        );
                    }
                }
                let nbr = &self.nbr_list[o_start + i];
                if nbr.timestamp <= ts && nbr.timestamp != INVALID_TIMESTAMP {
                    result.push(*nbr);
                }
            }
        }

        result
    }

    #[cfg(not(target_arch = "x86_64"))]
    pub fn edges_of_with_prefetch(&self, src: VertexId, ts: Timestamp) -> Vec<Nbr> {
        self.edges_of(src, ts)
    }

    fn count_valid_primary(&self, src_idx: usize, ts: Timestamp) -> usize {
        let degree = self.degrees[src_idx] as usize;
        let offset = self.adj_offsets[src_idx] as usize;
        let mut count = 0;
        for i in 0..degree {
            let nbr = &self.nbr_list[offset + i];
            if nbr.timestamp <= ts && nbr.timestamp != INVALID_TIMESTAMP {
                count += 1;
            }
        }
        count
    }

    fn count_valid_overflow(&self, src_idx: usize, ts: Timestamp) -> usize {
        if self.overflow_starts[src_idx] == NO_OVERFLOW {
            return 0;
        }
        let o_start = self.overflow_starts[src_idx] as usize;
        let o_count = self.overflow_counts[src_idx] as usize;
        let mut count = 0;
        for i in 0..o_count {
            let nbr = &self.nbr_list[o_start + i];
            if nbr.timestamp <= ts && nbr.timestamp != INVALID_TIMESTAMP {
                count += 1;
            }
        }
        count
    }

    pub fn get_vertex_capacity(&self) -> usize {
        self.vertex_capacity
    }

    /// Get a specific edge
    pub fn get_edge(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> Option<Nbr> {
        let src_idx = src.as_int64().unwrap_or(0) as usize;
        if src_idx >= self.vertex_capacity {
            return None;
        }

        // Scan primary
        let degree = self.degrees[src_idx] as usize;
        let offset = self.adj_offsets[src_idx] as usize;
        for i in 0..degree {
            let nbr = &self.nbr_list[offset + i];
            if nbr.neighbor == dst && nbr.timestamp <= ts && nbr.timestamp != INVALID_TIMESTAMP {
                return Some(*nbr);
            }
        }

        // Scan overflow
        if self.overflow_starts[src_idx] != NO_OVERFLOW {
            let o_start = self.overflow_starts[src_idx] as usize;
            let o_count = self.overflow_counts[src_idx] as usize;
            for i in 0..o_count {
                let nbr = &self.nbr_list[o_start + i];
                if nbr.neighbor == dst && nbr.timestamp <= ts && nbr.timestamp != INVALID_TIMESTAMP
                {
                    return Some(*nbr);
                }
            }
        }

        None
    }

    /// Clear all edges
    pub fn clear(&mut self) {
        for nbr in &mut self.nbr_list {
            *nbr = Nbr::new(VertexId::from_int64(0), 0, 0, INVALID_TIMESTAMP);
        }
        for degree in &mut self.degrees {
            *degree = 0;
        }
        for o_count in &mut self.overflow_counts {
            *o_count = 0;
        }
        self.edge_count.store(0, Ordering::Relaxed);
    }

    /// Batch insert edges with parallel optimization
    ///
    /// Uses a two-phase approach:
    /// - Phase 1 (sequential): Group edges by source, calculate primary/overflow split
    /// - Phase 2 (parallel): Write primary edges via unsafe non-overlapping writes
    /// - Phase 3 (sequential): Write overflow edges to per-vertex Vecs
    ///
    /// # Safety
    ///
    /// Parallel writes are safe because each vertex's primary region is non-overlapping.
    pub fn batch_insert_parallel(
        &mut self,
        src_list: &[VertexId],
        dst_list: &[VertexId],
        edge_ids: &[EdgeId],
        prop_offsets: &[u32],
        ts: Timestamp,
    ) {
        use rayon::prelude::*;
        use std::collections::HashMap;

        assert_eq!(
            src_list.len(),
            dst_list.len(),
            "Source and destination lists must have equal length"
        );
        assert_eq!(
            src_list.len(),
            edge_ids.len(),
            "Source and edge ID lists must have equal length"
        );
        assert_eq!(
            src_list.len(),
            prop_offsets.len(),
            "Source and property offset lists must have equal length"
        );

        if src_list.is_empty() {
            return;
        }

        // Phase 1: Pre-allocation and grouping (sequential)
        let max_vertex = src_list
            .iter()
            .max()
            .cloned()
            .unwrap_or(VertexId::zero())
            .as_int64()
            .unwrap_or(0) as usize;
        self.ensure_vertex_capacity(max_vertex + 1);

        // Group edges by source vertex and split into primary/overflow
        let mut groups: HashMap<VertexId, Vec<(VertexId, EdgeId, u32)>> = HashMap::new();
        for i in 0..src_list.len() {
            groups.entry(src_list[i]).or_default().push((
                dst_list[i],
                edge_ids[i],
                prop_offsets[i],
            ));
        }

        struct VertexBatch {
            primary_start: usize,
            primary_count: usize,
        }

        let mut batch_info: HashMap<VertexId, VertexBatch> = HashMap::new();
        let mut total_new_edges = 0usize;

        for (&src, edges) in &groups {
            let src_idx = src.as_int64().unwrap_or(0) as usize;
            let current_degree = self.degrees[src_idx] as usize;
            let capacity = self.primary_capacities[src_idx] as usize;
            let new_edges = edges.len();
            let primary_space = capacity.saturating_sub(current_degree);
            let primary_count = primary_space.min(new_edges);

            batch_info.insert(
                src,
                VertexBatch {
                    primary_start: (self.adj_offsets[src_idx] as usize) + current_degree,
                    primary_count,
                },
            );
            total_new_edges += new_edges;
        }

        // Phase 2: Parallel primary writes (unsafe non-overlapping regions)
        // Clone batch_info so the closure doesn't consume the original
        let nbr_list_ptr = self.nbr_list.as_mut_ptr() as usize;
        let degrees_ptr = self.degrees.as_mut_ptr() as usize;

        groups.par_iter().for_each(|(src, edges)| {
            let src_idx = src.as_int64().unwrap_or(0) as usize;
            let info = &batch_info[src];
            let mut pos = info.primary_start;
            let mut written = 0usize;

            unsafe {
                let nbr_list_ptr = nbr_list_ptr as *mut Nbr;
                let degrees_ptr = degrees_ptr as *mut u32;
                for (dst, edge_id, prop_offset) in edges {
                    if written >= info.primary_count {
                        break;
                    }
                    std::ptr::write(
                        nbr_list_ptr.add(pos),
                        Nbr::new(*dst, *edge_id, *prop_offset, ts),
                    );
                    pos += 1;
                    written += 1;
                }
                let old_degree = std::ptr::read(degrees_ptr.add(src_idx));
                std::ptr::write(degrees_ptr.add(src_idx), old_degree + written as u32);
            }
        });

        // Phase 3: Sequential overflow writes (expand overflow blocks in nbr_list)
        for (src, edges) in &groups {
            let info = &batch_info[src];
            if info.primary_count < edges.len() {
                let src_idx = src.as_int64().unwrap_or(0) as usize;
                for (dst, edge_id, prop_offset) in &edges[info.primary_count..] {
                    if self.overflow_starts[src_idx] == NO_OVERFLOW
                        || self.overflow_counts[src_idx] >= self.overflow_capacities[src_idx]
                    {
                        self.expand_vertex_capacity(src_idx);
                    }
                    let o_start = self.overflow_starts[src_idx] as usize;
                    let o_count = self.overflow_counts[src_idx] as usize;
                    self.nbr_list[o_start + o_count] = Nbr::new(*dst, *edge_id, *prop_offset, ts);
                    self.overflow_counts[src_idx] += 1;
                }
            }
        }

        // Update global edge count
        self.edge_count
            .fetch_add(total_new_edges as u64, Ordering::Relaxed);
    }

    /// Batch delete edges
    pub fn batch_delete_edges(&mut self, edges: &[(VertexId, EdgeId)], ts: Timestamp) {
        for (src, edge_id) in edges {
            self.delete_edge(*src, *edge_id, ts);
        }
    }

    /// Create iterator over all edges
    pub fn iter(&self, ts: Timestamp) -> MutableCsrIterator<'_> {
        MutableCsrIterator::new(self, ts)
    }

    /// Dump to bytes
    ///
    /// Format:
    /// - vertex_capacity (u64)
    /// - edge_count (u64)
    /// - total_edge_capacity (u64)
    /// - adj_offsets (u32 * vertex_capacity)
    /// - degrees (u32 * vertex_capacity)
    /// - primary_capacities (u32 * vertex_capacity)
    /// - overflow_starts (u32 * vertex_capacity)
    /// - overflow_counts (u32 * vertex_capacity)
    /// - overflow_capacities (u32 * vertex_capacity)
    /// - nbr_list (Nbr * total_edge_capacity)
    pub fn dump(&self) -> Vec<u8> {
        let mut result = Vec::new();

        result.extend_from_slice(&(self.vertex_capacity as u64).to_le_bytes());
        result.extend_from_slice(&self.edge_count.load(Ordering::Relaxed).to_le_bytes());
        result.extend_from_slice(&(self.total_edge_capacity as u64).to_le_bytes());

        for &offset in &self.adj_offsets {
            result.extend_from_slice(&offset.to_le_bytes());
        }

        for &degree in &self.degrees {
            result.extend_from_slice(&degree.to_le_bytes());
        }

        for &cap in &self.primary_capacities {
            result.extend_from_slice(&cap.to_le_bytes());
        }

        for &start in &self.overflow_starts {
            result.extend_from_slice(&start.to_le_bytes());
        }

        for &count in &self.overflow_counts {
            result.extend_from_slice(&count.to_le_bytes());
        }

        for &cap in &self.overflow_capacities {
            result.extend_from_slice(&cap.to_le_bytes());
        }

        for nbr in &self.nbr_list {
            write_vertex_id(&mut result, nbr.neighbor);
            result.extend_from_slice(&nbr.edge_id.to_le_bytes());
            result.extend_from_slice(&nbr.prop_offset.to_le_bytes());
            result.extend_from_slice(&nbr.timestamp.to_le_bytes());
        }

        result
    }

    /// Load from bytes
    pub fn load(&mut self, data: &[u8]) -> StorageResult<()> {
        if data.len() < 24 {
            return Err(StorageError::deserialize_error(
                "CSR data too short for header",
            ));
        }

        let mut offset = 0usize;

        let vertex_capacity = read_u64_le(data, &mut offset)? as usize;
        let edge_count = read_u64_le(data, &mut offset)?;
        let total_edge_capacity = read_u64_le(data, &mut offset)? as usize;

        let mut adj_offsets = Vec::with_capacity(vertex_capacity);
        for _ in 0..vertex_capacity {
            adj_offsets.push(read_u32_le(data, &mut offset)?);
        }

        let mut degrees = Vec::with_capacity(vertex_capacity);
        for _ in 0..vertex_capacity {
            degrees.push(read_u32_le(data, &mut offset)?);
        }

        let mut primary_capacities = Vec::with_capacity(vertex_capacity);
        for _ in 0..vertex_capacity {
            primary_capacities.push(read_u32_le(data, &mut offset)?);
        }

        let mut overflow_starts = Vec::with_capacity(vertex_capacity);
        for _ in 0..vertex_capacity {
            overflow_starts.push(read_u32_le(data, &mut offset)?);
        }

        let mut overflow_counts = Vec::with_capacity(vertex_capacity);
        for _ in 0..vertex_capacity {
            overflow_counts.push(read_u32_le(data, &mut offset)?);
        }

        let mut overflow_capacities = Vec::with_capacity(vertex_capacity);
        for _ in 0..vertex_capacity {
            overflow_capacities.push(read_u32_le(data, &mut offset)?);
        }

        let nbr_count = total_edge_capacity;
        let mut nbr_list = Vec::with_capacity(nbr_count);
        for _ in 0..nbr_count {
            let neighbor = read_vertex_id(data, &mut offset)?;
            let edge_id = read_u64_le(data, &mut offset)?;
            let prop_offset = read_u32_le(data, &mut offset)?;
            let timestamp = read_u32_le(data, &mut offset)?;

            nbr_list.push(Nbr::new(neighbor, edge_id, prop_offset, timestamp));
        }

        self.vertex_capacity = vertex_capacity;
        self.total_edge_capacity = total_edge_capacity;
        self.adj_offsets = adj_offsets;
        self.degrees = degrees;
        self.primary_capacities = primary_capacities;
        self.overflow_starts = overflow_starts;
        self.overflow_counts = overflow_counts;
        self.overflow_capacities = overflow_capacities;
        self.nbr_list = nbr_list;
        self.edge_count.store(edge_count, Ordering::Relaxed);

        Ok(())
    }

    /// Compact CSR by removing deleted edges and reclaiming space.
    /// Merges overflow back into primary, restoring flat CSR layout.
    pub fn compact_with_ts(&mut self, ts: u32, reserve_ratio: f32) -> usize {
        // Phase 1: compact individual vertex data (primary + overflow)
        // and compute new layout.
        let mut new_offsets = Vec::with_capacity(self.vertex_capacity);
        let mut new_degrees = Vec::with_capacity(self.vertex_capacity);
        let mut new_capacities = Vec::with_capacity(self.vertex_capacity);
        let mut new_edges = Vec::<Nbr>::new();
        let mut removed_count = 0usize;

        for vid in 0..self.vertex_capacity {
            let start = self.adj_offsets[vid] as usize;
            let degree = self.degrees[vid] as usize;

            new_offsets.push(new_edges.len());

            // Collect valid edges from primary
            for i in 0..degree {
                let nbr = &self.nbr_list[start + i];
                if nbr.timestamp <= ts {
                    new_edges.push(*nbr);
                } else {
                    removed_count += 1;
                }
            }

            // Collect valid edges from overflow
            if self.overflow_starts[vid] != NO_OVERFLOW {
                let o_start = self.overflow_starts[vid] as usize;
                let o_count = self.overflow_counts[vid] as usize;
                for i in 0..o_count {
                    let nbr = &self.nbr_list[o_start + i];
                    if nbr.timestamp <= ts {
                        new_edges.push(*nbr);
                    } else {
                        removed_count += 1;
                    }
                }
            }

            let valid = new_edges.len() - new_offsets[vid];
            new_degrees.push(valid as u32);
            let new_cap = ((valid as f32 / (1.0 - reserve_ratio)).ceil() as u32).max(1);
            new_capacities.push(new_cap);
        }

        // Phase 2: rebuild nbr_list as flat CSR (no overflow)
        let new_total_edge_capacity: usize = new_capacities.iter().map(|&c| c as usize).sum();
        let mut new_nbr_list = Vec::with_capacity(new_total_edge_capacity);
        let mut final_offsets = Vec::with_capacity(self.vertex_capacity);

        for vid in 0..self.vertex_capacity {
            final_offsets.push(new_nbr_list.len() as u32);
            let off = new_offsets[vid];
            let deg = new_degrees[vid] as usize;
            let cap = new_capacities[vid] as usize;

            new_nbr_list.extend_from_slice(&new_edges[off..off + deg]);
            // Fill remaining capacity with empty Nbr
            let remaining = cap - deg;
            if remaining > 0 {
                new_nbr_list.resize(
                    new_nbr_list.len() + remaining,
                    Nbr::new(VertexId::from_int64(0), 0, 0, INVALID_TIMESTAMP),
                );
            }
        }

        self.nbr_list = new_nbr_list;
        self.adj_offsets = final_offsets;
        self.degrees = new_degrees;
        self.primary_capacities = new_capacities;
        self.total_edge_capacity = new_total_edge_capacity;

        // Clear all overflow
        for start in &mut self.overflow_starts {
            *start = NO_OVERFLOW;
        }
        for count in &mut self.overflow_counts {
            *count = 0;
        }
        for cap in &mut self.overflow_capacities {
            *cap = 0;
        }

        removed_count
    }

    /// Get memory size
    pub fn memory_size(&self) -> usize {
        let mut total = 0;

        total += self.nbr_list.len() * std::mem::size_of::<Nbr>();
        total += self.adj_offsets.len() * std::mem::size_of::<u32>();
        total += self.degrees.len() * std::mem::size_of::<u32>();
        total += self.primary_capacities.len() * std::mem::size_of::<u32>();
        total += self.overflow_starts.len() * std::mem::size_of::<u32>();
        total += self.overflow_counts.len() * std::mem::size_of::<u32>();
        total += self.overflow_capacities.len() * std::mem::size_of::<u32>();
        total += std::mem::size_of::<Self>();

        total
    }

    /// Get used memory size (active edges only)
    pub fn used_memory_size(&self) -> usize {
        let active_edges = self.edge_count.load(Ordering::Relaxed) as usize;
        active_edges * std::mem::size_of::<Nbr>() + std::mem::size_of::<Self>()
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
    in_overflow: bool,
    overflow_idx: usize,
}

impl<'a> MutableCsrIterator<'a> {
    pub fn new(csr: &'a MutableCsr, ts: Timestamp) -> Self {
        Self {
            csr,
            ts,
            current_vertex: 0,
            current_edge: 0,
            in_overflow: false,
            overflow_idx: 0,
        }
    }
}

impl<'a> Iterator for MutableCsrIterator<'a> {
    type Item = (VertexId, Nbr);

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_vertex < self.csr.vertex_capacity {
            let degree = self.csr.degrees[self.current_vertex] as usize;
            let offset = self.csr.adj_offsets[self.current_vertex] as usize;

            if !self.in_overflow {
                // Scan primary
                while self.current_edge < degree {
                    let nbr = self.csr.nbr_list[offset + self.current_edge];
                    self.current_edge += 1;
                    if nbr.timestamp <= self.ts && nbr.timestamp != INVALID_TIMESTAMP {
                        return Some((VertexId::from_int64(self.current_vertex as i64), nbr));
                    }
                }
                // Move to overflow phase
                self.in_overflow = true;
                self.overflow_idx = 0;
            }

            // Scan overflow
            if self.csr.overflow_starts[self.current_vertex] != NO_OVERFLOW {
                let o_start = self.csr.overflow_starts[self.current_vertex] as usize;
                let o_count = self.csr.overflow_counts[self.current_vertex] as usize;
                while self.overflow_idx < o_count {
                    let nbr = self.csr.nbr_list[o_start + self.overflow_idx];
                    self.overflow_idx += 1;
                    if nbr.timestamp <= self.ts && nbr.timestamp != INVALID_TIMESTAMP {
                        return Some((VertexId::from_int64(self.current_vertex as i64), nbr));
                    }
                }
            }

            // Move to next vertex
            self.current_vertex += 1;
            self.current_edge = 0;
            self.in_overflow = false;
            self.overflow_idx = 0;
        }
        None
    }
}

impl CsrBase for MutableCsr {
    fn vertex_capacity(&self) -> usize {
        self.vertex_capacity
    }

    fn edge_count(&self) -> u64 {
        self.edge_count.load(Ordering::Relaxed)
    }

    fn dump(&self) -> Vec<u8> {
        MutableCsr::dump(self)
    }

    fn load(&mut self, data: &[u8]) -> StorageResult<()> {
        MutableCsr::load(self, data)
    }
}

impl MutableCsrTrait for MutableCsr {
    fn insert_edge(
        &mut self,
        src: VertexId,
        dst: VertexId,
        edge_id: EdgeId,
        prop_offset: u32,
        ts: Timestamp,
    ) -> bool {
        MutableCsr::insert_edge(self, src, dst, edge_id, prop_offset, ts)
    }

    fn delete_edge(&mut self, src: VertexId, edge_id: EdgeId, ts: Timestamp) -> bool {
        MutableCsr::delete_edge(self, src, edge_id, ts)
    }

    fn delete_edge_by_dst(&mut self, src: VertexId, dst: VertexId, ts: Timestamp) -> bool {
        MutableCsr::delete_edge_by_dst(self, src, dst, ts)
    }

    fn delete_edge_by_offset(&mut self, src: VertexId, offset: i32, ts: Timestamp) -> bool {
        MutableCsr::delete_edge_by_offset(self, src, offset, ts)
    }

    fn revert_delete_by_offset(&mut self, src: VertexId, offset: i32, ts: Timestamp) -> bool {
        MutableCsr::revert_delete_by_offset(self, src, offset, ts)
    }

    fn get_edge(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> Option<Nbr> {
        MutableCsr::get_edge(self, src, dst, ts)
    }

    fn edges_of(&self, src: VertexId, ts: Timestamp) -> Vec<Nbr> {
        MutableCsr::edges_of(self, src, ts)
    }

    fn compact_with_ts(&mut self, ts: Timestamp, reserve_ratio: f32) -> usize {
        MutableCsr::compact_with_ts(self, ts, reserve_ratio)
    }

    fn find_deleted_edge(&self, src: VertexId, dst: VertexId) -> Option<EdgeId> {
        MutableCsr::find_deleted_edge(self, src, dst)
    }

    fn used_memory_size(&self) -> usize {
        MutableCsr::used_memory_size(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_insert_and_query() {
        let mut csr = MutableCsr::with_capacity(10, 100);

        assert!(csr.insert_edge(VertexId::from_int64(0), VertexId::from_int64(1), 100, 0, 1));
        assert!(csr.insert_edge(VertexId::from_int64(0), VertexId::from_int64(2), 101, 0, 1));
        assert!(csr.insert_edge(VertexId::from_int64(1), VertexId::from_int64(3), 102, 0, 1));

        assert!(!csr.insert_edge(VertexId::from_int64(0), VertexId::from_int64(1), 103, 0, 1));

        assert_eq!(csr.edge_count(), 3);
    }

    #[test]
    fn test_delete_edge() {
        let mut csr = MutableCsr::with_capacity(10, 100);

        csr.insert_edge(VertexId::from_int64(0), VertexId::from_int64(1), 100, 0, 1);
        csr.insert_edge(VertexId::from_int64(0), VertexId::from_int64(2), 101, 0, 1);

        assert!(csr.delete_edge(VertexId::from_int64(0), 100, 2));

        assert_eq!(csr.edge_count(), 1);
    }

    #[test]
    fn test_dump_and_load() {
        let mut csr1 = MutableCsr::with_capacity(10, 100);

        csr1.insert_edge(VertexId::from_int64(0), VertexId::from_int64(1), 100, 0, 1);
        csr1.insert_edge(VertexId::from_int64(0), VertexId::from_int64(2), 101, 0, 1);
        csr1.insert_edge(VertexId::from_int64(1), VertexId::from_int64(3), 102, 0, 1);

        let data = csr1.dump();

        let mut csr2 = MutableCsr::new();
        let _ = csr2.load(&data);

        assert_eq!(csr2.vertex_capacity(), csr1.vertex_capacity());
        assert_eq!(csr2.edge_count(), csr1.edge_count());
    }

    #[test]
    fn test_resize() {
        let mut csr = MutableCsr::with_capacity(2, 10);

        csr.insert_edge(VertexId::from_int64(0), VertexId::from_int64(1), 100, 0, 1);
        csr.insert_edge(
            VertexId::from_int64(100),
            VertexId::from_int64(1),
            101,
            0,
            1,
        );

        assert!(csr.vertex_capacity() >= 101);
    }

    #[test]
    fn test_iterator() {
        let mut csr = MutableCsr::with_capacity(10, 100);

        csr.insert_edge(VertexId::from_int64(0), VertexId::from_int64(1), 100, 0, 1);
        csr.insert_edge(VertexId::from_int64(0), VertexId::from_int64(2), 101, 0, 1);
        csr.insert_edge(VertexId::from_int64(1), VertexId::from_int64(3), 102, 0, 1);

        let edges: Vec<_> = csr.iter(1).collect();
        assert_eq!(edges.len(), 3);
    }

    #[test]
    fn test_overflow_insert() {
        let mut csr = MutableCsr::with_capacity(10, 100);

        assert!(csr.insert_edge(VertexId::from_int64(0), VertexId::from_int64(1), 100, 0, 1));
        assert!(csr.insert_edge(VertexId::from_int64(0), VertexId::from_int64(2), 101, 0, 1));
        assert!(csr.insert_edge(VertexId::from_int64(0), VertexId::from_int64(3), 102, 0, 1));
        assert!(csr.insert_edge(VertexId::from_int64(0), VertexId::from_int64(4), 103, 0, 1));
        assert!(csr.insert_edge(VertexId::from_int64(0), VertexId::from_int64(5), 104, 0, 1));

        assert_eq!(csr.edge_count(), 5);

        let edges = csr.edges_of(VertexId::from_int64(0), 1);
        assert_eq!(edges.len(), 5);

        assert!(!csr.insert_edge(VertexId::from_int64(0), VertexId::from_int64(5), 105, 0, 1));

        assert!(csr.delete_edge(VertexId::from_int64(0), 104, 2));
    }

    #[test]
    fn test_overflow_dump_and_load() {
        let mut csr1 = MutableCsr::with_capacity(10, 100);

        for i in 1..=6 {
            let dst = VertexId::from_int64(i as i64);
            csr1.insert_edge(VertexId::from_int64(0), dst, i as u64, 0, 1);
        }

        let data = csr1.dump();

        let mut csr2 = MutableCsr::new();
        let _ = csr2.load(&data);

        assert_eq!(csr2.vertex_capacity(), csr1.vertex_capacity());
        assert_eq!(csr2.edge_count(), csr1.edge_count());
        assert_eq!(csr2.overflow_counts[0], 2);
    }

    #[test]
    fn test_compact_with_ts_merges_overflow() {
        let mut csr = MutableCsr::with_capacity(10, 100);

        for i in 1..=6 {
            let dst = VertexId::from_int64(i as i64);
            csr.insert_edge(VertexId::from_int64(0), dst, i as u64, 0, 1);
        }

        csr.delete_edge(VertexId::from_int64(0), 3, 5);
        csr.delete_edge(VertexId::from_int64(0), 5, 5);
        csr.delete_edge(VertexId::from_int64(0), 6, 5);

        let removed = csr.compact_with_ts(3, 0.25);
        assert_eq!(removed, 3);

        assert!(csr.overflow_starts[0] == NO_OVERFLOW);

        let edges = csr.edges_of(VertexId::from_int64(0), 3);
        assert_eq!(edges.len(), 3);
    }

    #[test]
    fn test_overflow_iterator() {
        let mut csr = MutableCsr::with_capacity(10, 100);

        for i in 1..=6 {
            let dst = VertexId::from_int64(i as i64);
            csr.insert_edge(VertexId::from_int64(0), dst, i as u64, 0, 1);
        }

        let all_edges: Vec<_> = csr.iter(1).collect();
        assert_eq!(all_edges.len(), 6);
    }

    #[test]
    fn test_multiple_vertices_overflow() {
        let mut csr = MutableCsr::with_capacity(10, 100);

        for v in 0..3 {
            for i in 0..6 {
                let src = VertexId::from_int64(v);
                let dst = VertexId::from_int64(v * 100 + i + 1);
                csr.insert_edge(src, dst, (v * 10 + i) as u64, 0, 1);
            }
        }

        let all: Vec<_> = csr.iter(1).collect();
        assert_eq!(all.len(), 18);
    }
}
