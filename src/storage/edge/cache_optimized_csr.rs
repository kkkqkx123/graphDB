//! Cache-Optimized CSR Implementation
//!
//! This module provides a cache-optimized CSR implementation using Structure of Arrays (SoA)
//! layout instead of Array of Structures (AoS). This layout provides better cache locality
//! and enables SIMD optimizations.
//!
//! # Benefits of SoA Layout
//!
//! - **Better cache utilization**: When iterating over a single field (e.g., timestamps),
//!   the CPU cache is filled with only that field's data, not the entire struct.
//! - **SIMD-friendly**: Data is naturally aligned for SIMD operations.
//! - **Reduced cache line waste**: No padding between different fields.

use std::sync::atomic::{AtomicU64, Ordering};

use super::{CsrBase, CsrType, EdgeId, MutableCsrTrait, Nbr, Timestamp, VertexId, INVALID_TIMESTAMP};
use super::mutable_csr::{SpinLock, SpinLockGuard};

const DEFAULT_VERTEX_CAPACITY: usize = 1024;
const DEFAULT_EDGE_CAPACITY: usize = 4096;
const DEFAULT_VERTEX_DEGREE: usize = 4;

/// Cache-optimized CSR with Structure of Arrays (SoA) layout
///
/// Instead of storing `Vec<Nbr>` where each Nbr contains all fields,
/// we store each field in a separate array. This improves cache locality
/// when accessing individual fields and enables SIMD optimizations.
pub struct CacheOptimizedCsr {
    // SoA layout: each field in its own contiguous array
    neighbors: Vec<VertexId>,
    edge_ids: Vec<EdgeId>,
    prop_offsets: Vec<u32>,
    timestamps: Vec<Timestamp>,

    // CSR structure
    adj_offsets: Vec<usize>,
    degrees: Vec<u32>,
    capacities: Vec<u32>,
    locks: Vec<SpinLock>,

    edge_count: AtomicU64,
    vertex_capacity: usize,
    total_edge_capacity: usize,
}

impl Clone for CacheOptimizedCsr {
    fn clone(&self) -> Self {
        Self {
            neighbors: self.neighbors.clone(),
            edge_ids: self.edge_ids.clone(),
            prop_offsets: self.prop_offsets.clone(),
            timestamps: self.timestamps.clone(),
            adj_offsets: self.adj_offsets.clone(),
            degrees: self.degrees.clone(),
            capacities: self.capacities.clone(),
            locks: (0..self.vertex_capacity).map(|_| SpinLock::new()).collect(),
            edge_count: AtomicU64::new(self.edge_count.load(Ordering::Relaxed)),
            vertex_capacity: self.vertex_capacity,
            total_edge_capacity: self.total_edge_capacity,
        }
    }
}

impl std::fmt::Debug for CacheOptimizedCsr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CacheOptimizedCsr")
            .field("vertex_capacity", &self.vertex_capacity)
            .field("total_edge_capacity", &self.total_edge_capacity)
            .field("edge_count", &self.edge_count.load(Ordering::Relaxed))
            .finish_non_exhaustive()
    }
}

impl CacheOptimizedCsr {
    /// Create a new empty CSR
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_VERTEX_CAPACITY, DEFAULT_EDGE_CAPACITY)
    }

    /// Create a new CSR with specified capacities
    pub fn with_capacity(vertex_capacity: usize, edge_capacity: usize) -> Self {
        let vertex_cap = vertex_capacity.max(1);
        let edge_cap = edge_capacity.max(vertex_cap * DEFAULT_VERTEX_DEGREE);

        let mut neighbors = Vec::with_capacity(edge_cap);
        let mut edge_ids = Vec::with_capacity(edge_cap);
        let mut prop_offsets = Vec::with_capacity(edge_cap);
        let mut timestamps = Vec::with_capacity(edge_cap);

        let mut adj_offsets = Vec::with_capacity(vertex_cap);
        let mut capacities = Vec::with_capacity(vertex_cap);

        let mut offset = 0usize;
        for _ in 0..vertex_cap {
            adj_offsets.push(offset);
            capacities.push(DEFAULT_VERTEX_DEGREE as u32);
            offset += DEFAULT_VERTEX_DEGREE;
        }

        neighbors.resize(offset, VertexId::zero());
        edge_ids.resize(offset, 0);
        prop_offsets.resize(offset, 0);
        timestamps.resize(offset, INVALID_TIMESTAMP);

        Self {
            neighbors,
            edge_ids,
            prop_offsets,
            timestamps,
            adj_offsets,
            degrees: vec![0; vertex_cap],
            capacities,
            locks: (0..vertex_cap).map(|_| SpinLock::new()).collect(),
            edge_count: AtomicU64::new(0),
            vertex_capacity: vertex_cap,
            total_edge_capacity: offset,
        }
    }

    /// Ensure vertex capacity is sufficient
    fn ensure_vertex_capacity(&mut self, new_capacity: usize) {
        if new_capacity <= self.vertex_capacity {
            return;
        }

        let old_capacity = self.vertex_capacity;
        let additional = new_capacity - old_capacity;

        // Extend offsets and capacities
        let mut offset = if self.adj_offsets.is_empty() {
            0
        } else {
            self.adj_offsets.last().copied().unwrap_or(0)
                + self.capacities.last().copied().unwrap_or(0) as usize
        };

        for _ in 0..additional {
            self.adj_offsets.push(offset);
            self.capacities.push(DEFAULT_VERTEX_DEGREE as u32);
            offset += DEFAULT_VERTEX_DEGREE;
        }

        // Extend data arrays
        let additional_edges = additional * DEFAULT_VERTEX_DEGREE;
        self.neighbors.resize(self.neighbors.len() + additional_edges, VertexId::zero());
        self.edge_ids.resize(self.edge_ids.len() + additional_edges, 0);
        self.prop_offsets.resize(self.prop_offsets.len() + additional_edges, 0);
        self.timestamps.resize(self.timestamps.len() + additional_edges, INVALID_TIMESTAMP);

        // Extend degrees and locks
        self.degrees.extend(std::iter::repeat_n(0, additional));
        self.locks.extend((0..additional).map(|_| SpinLock::new()));

        self.vertex_capacity = new_capacity;
        self.total_edge_capacity = offset;
    }

    /// Ensure edge capacity for a vertex
    fn ensure_edge_capacity(&mut self, src_idx: usize, required: usize) {
        let current_capacity = self.capacities[src_idx] as usize;
        if required <= current_capacity {
            return;
        }

        let new_capacity = required.max(current_capacity * 2);
        let old_capacity = current_capacity;
        let additional = new_capacity - old_capacity;

        // Find position to insert
        let insert_pos = self.adj_offsets[src_idx] + old_capacity;

        // Insert space in all arrays
        self.neighbors.splice(insert_pos..insert_pos, std::iter::repeat_n(VertexId::zero(), additional));
        self.edge_ids.splice(insert_pos..insert_pos, std::iter::repeat_n(0, additional));
        self.prop_offsets.splice(insert_pos..insert_pos, std::iter::repeat_n(0, additional));
        self.timestamps.splice(insert_pos..insert_pos, std::iter::repeat_n(INVALID_TIMESTAMP, additional));

        // Update capacities and offsets for subsequent vertices
        self.capacities[src_idx] = new_capacity as u32;
        for offset in self.adj_offsets.iter_mut().skip(src_idx + 1) {
            *offset += additional;
        }

        self.total_edge_capacity += additional;
    }

    /// Insert an edge (internal implementation)
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

        let _guard = SpinLockGuard::new(&self.locks[src_idx]);

        let degree = self.degrees[src_idx] as usize;
        let capacity = self.capacities[src_idx] as usize;
        let offset = self.adj_offsets[src_idx];

        // Check if edge already exists
        for i in 0..degree {
            if self.neighbors[offset + i] == dst && self.timestamps[offset + i] != INVALID_TIMESTAMP {
                return false;
            }
        }

        // Check for deleted slot
        for i in 0..degree {
            if self.timestamps[offset + i] == INVALID_TIMESTAMP {
                self.neighbors[offset + i] = dst;
                self.edge_ids[offset + i] = edge_id;
                self.prop_offsets[offset + i] = prop_offset;
                self.timestamps[offset + i] = ts;
                self.edge_count.fetch_add(1, Ordering::Relaxed);
                return true;
            }
        }

        // Need new slot
        if degree >= capacity {
            drop(_guard);
            self.ensure_edge_capacity(src_idx, degree + 1);
            let _guard = SpinLockGuard::new(&self.locks[src_idx]);
            let offset = self.adj_offsets[src_idx];
            self.neighbors[offset + degree] = dst;
            self.edge_ids[offset + degree] = edge_id;
            self.prop_offsets[offset + degree] = prop_offset;
            self.timestamps[offset + degree] = ts;
            self.degrees[src_idx] += 1;
            self.edge_count.fetch_add(1, Ordering::Relaxed);
        } else {
            self.neighbors[offset + degree] = dst;
            self.edge_ids[offset + degree] = edge_id;
            self.prop_offsets[offset + degree] = prop_offset;
            self.timestamps[offset + degree] = ts;
            self.degrees[src_idx] += 1;
            self.edge_count.fetch_add(1, Ordering::Relaxed);
        }

        true
    }

    /// Get edges of a vertex at a given timestamp
    pub fn edges_of(&self, src: VertexId, ts: Timestamp) -> Vec<Nbr> {
        let src_idx = src.as_int64().unwrap_or(0) as usize;
        if src_idx >= self.vertex_capacity {
            return Vec::new();
        }

        let degree = self.degrees[src_idx] as usize;
        let offset = self.adj_offsets[src_idx];

        let mut result = Vec::with_capacity(degree);
        for i in 0..degree {
            let timestamp = self.timestamps[offset + i];
            if timestamp <= ts && timestamp != INVALID_TIMESTAMP {
                result.push(Nbr {
                    neighbor: self.neighbors[offset + i],
                    edge_id: self.edge_ids[offset + i],
                    prop_offset: self.prop_offsets[offset + i],
                    timestamp,
                });
            }
        }
        result
    }

    /// Get edges with SIMD optimization (x86_64 only)
    /// 
    /// Uses AVX2 instructions to filter timestamps in parallel.
    /// Falls back to scalar version if AVX2 is not available.
    #[cfg(target_arch = "x86_64")]
    pub fn edges_of_simd(&self, src: VertexId, ts: Timestamp) -> Vec<Nbr> {
        if is_x86_feature_detected!("avx2") {
            unsafe { self.edges_of_avx2(src, ts) }
        } else {
            self.edges_of(src, ts)
        }
    }

    /// AVX2-optimized edge filtering
    /// 
    /// Filters edges based on timestamp using AVX2 SIMD instructions.
    /// Processes 8 timestamps in parallel.
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn edges_of_avx2(&self, src: VertexId, ts: Timestamp) -> Vec<Nbr> {
        use std::arch::x86_64::*;

        let src_idx = src.as_int64().unwrap_or(0) as usize;
        if src_idx >= self.vertex_capacity {
            return Vec::new();
        }

        let degree = self.degrees[src_idx] as usize;
        let offset = self.adj_offsets[src_idx];

        let mut result = Vec::with_capacity(degree);

        // Process 8 timestamps at a time
        let ts_vec = _mm256_set1_epi32(ts as i32);
        let invalid_vec = _mm256_set1_epi32(INVALID_TIMESTAMP as i32);

        let chunks = degree / 8;
        let _remainder = degree % 8;

        // Process chunks of 8
        for chunk_idx in 0..chunks {
            let i = chunk_idx * 8;
            let ptr = self.timestamps.as_ptr().add(offset + i);
            let ts_chunk = _mm256_loadu_si256(ptr as *const __m256i);

            // Condition 1: timestamp <= ts
            // _mm256_cmpgt_epi32(a, b) returns 0xFFFFFFFF if a > b, else 0
            // We want timestamp <= ts, i.e., NOT (timestamp > ts)
            let gt_ts = _mm256_cmpgt_epi32(ts_chunk, ts_vec);
            let all_ones = _mm256_set1_epi32(-1);
            let le_ts = _mm256_xor_si256(gt_ts, all_ones);

            // Condition 2: timestamp != INVALID_TIMESTAMP
            // We want timestamp != INVALID, i.e., NOT (timestamp == INVALID)
            let eq_invalid = _mm256_cmpeq_epi32(ts_chunk, invalid_vec);
            let ne_invalid = _mm256_xor_si256(eq_invalid, all_ones);

            // Combine: timestamp <= ts AND timestamp != INVALID
            let valid = _mm256_and_si256(le_ts, ne_invalid);

            // Extract mask - each int32 element maps to 4 bytes in the mask
            let mask = _mm256_movemask_epi8(valid);

            // Check each element (every 4 bytes in mask corresponds to one i32)
            for j in 0..8 {
                // Each i32 occupies 4 bytes, so we check bits j*4 to j*4+3
                // If any of these bits are set, the comparison was true
                if (mask & (0xF << (j * 4))) != 0 {
                    let idx = offset + i + j;
                    result.push(Nbr {
                        neighbor: self.neighbors[idx],
                        edge_id: self.edge_ids[idx],
                        prop_offset: self.prop_offsets[idx],
                        timestamp: self.timestamps[idx],
                    });
                }
            }
        }

        // Handle remainder with scalar code
        for i in (chunks * 8)..degree {
            let idx = offset + i;
            let timestamp = self.timestamps[idx];
            if timestamp <= ts && timestamp != INVALID_TIMESTAMP {
                result.push(Nbr {
                    neighbor: self.neighbors[idx],
                    edge_id: self.edge_ids[idx],
                    prop_offset: self.prop_offsets[idx],
                    timestamp,
                });
            }
        }

        result
    }

    /// Get edges with SIMD optimization (fallback for non-x86_64)
    #[cfg(not(target_arch = "x86_64"))]
    pub fn edges_of_simd(&self, src: VertexId, ts: Timestamp) -> Vec<Nbr> {
        self.edges_of(src, ts)
    }

    /// Batch insert edges (optimized for bulk operations)
    /// 
    /// This method is optimized for bulk insertions and provides better
    /// performance than individual insert_edge calls.
    /// 
    /// # Note
    /// 
    /// Parallel insertion is currently not implemented due to the complexity
    /// of safe concurrent mutation. This method uses sequential insertion
    /// which is still efficient for batch operations.
    pub fn batch_insert_parallel(
        &mut self,
        src_list: &[VertexId],
        dst_list: &[VertexId],
        edge_ids: &[EdgeId],
        prop_offsets: &[u32],
        ts: Timestamp,
    ) {
        // Use sequential insertion for now
        // TODO: Implement parallel insertion with proper concurrency design
        self.batch_put_edges(src_list, dst_list, edge_ids, prop_offsets, ts);
    }
}

impl Default for CacheOptimizedCsr {
    fn default() -> Self {
        Self::new()
    }
}

impl CsrBase for CacheOptimizedCsr {
    fn vertex_capacity(&self) -> usize {
        self.vertex_capacity
    }

    fn edge_count(&self) -> u64 {
        self.edge_count.load(Ordering::Relaxed)
    }

    fn csr_type(&self) -> CsrType {
        CsrType::Mutable
    }

    fn resize(&mut self, new_vertex_capacity: usize) {
        self.ensure_vertex_capacity(new_vertex_capacity);
    }

    fn clear(&mut self) {
        self.neighbors.fill(VertexId::zero());
        self.edge_ids.fill(0);
        self.prop_offsets.fill(0);
        self.timestamps.fill(INVALID_TIMESTAMP);
        self.degrees.fill(0);
        self.edge_count.store(0, Ordering::Relaxed);
    }

    fn dump(&self) -> Vec<u8> {
        let mut result = Vec::new();

        result.extend_from_slice(&(self.vertex_capacity as u64).to_le_bytes());
        result.extend_from_slice(&self.edge_count.load(Ordering::Relaxed).to_le_bytes());
        result.extend_from_slice(&(self.total_edge_capacity as u64).to_le_bytes());

        for &offset in &self.adj_offsets {
            result.extend_from_slice(&(offset as u64).to_le_bytes());
        }

        for &degree in &self.degrees {
            result.extend_from_slice(&degree.to_le_bytes());
        }

        for &capacity in &self.capacities {
            result.extend_from_slice(&capacity.to_le_bytes());
        }

        for chunk in self.neighbors.chunks(1) {
            result.extend_from_slice(&chunk[0].as_int64().unwrap_or(0).to_le_bytes());
        }

        for chunk in self.edge_ids.chunks(1) {
            result.extend_from_slice(&chunk[0].to_le_bytes());
        }

        for chunk in self.prop_offsets.chunks(1) {
            result.extend_from_slice(&chunk[0].to_le_bytes());
        }

        for chunk in self.timestamps.chunks(1) {
            result.extend_from_slice(&chunk[0].to_le_bytes());
        }

        result
    }

    fn load(&mut self, data: &[u8]) {
        if data.len() < 24 {
            return;
        }

        let mut offset = 0;
        let vertex_capacity = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8])) as usize;
        offset += 8;
        let _edge_count = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8]));
        offset += 8;
        let total_edge_capacity = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8])) as usize;
        offset += 8;

        if vertex_capacity == 0 {
            return;
        }

        if offset + vertex_capacity * 8 > data.len() {
            return;
        }
        let mut adj_offsets = Vec::with_capacity(vertex_capacity);
        for _ in 0..vertex_capacity {
            let off = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8])) as usize;
            adj_offsets.push(off);
            offset += 8;
        }

        if offset + vertex_capacity * 4 > data.len() {
            return;
        }
        let mut degrees = Vec::with_capacity(vertex_capacity);
        for _ in 0..vertex_capacity {
            let deg = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap_or([0; 4]));
            degrees.push(deg);
            offset += 4;
        }

        if offset + vertex_capacity * 4 > data.len() {
            return;
        }
        let mut capacities = Vec::with_capacity(vertex_capacity);
        for _ in 0..vertex_capacity {
            let cap = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap_or([0; 4]));
            capacities.push(cap);
            offset += 4;
        }

        let nbr_count = total_edge_capacity;
        if offset + nbr_count * 8 > data.len() {
            return;
        }
        let mut neighbors = Vec::with_capacity(nbr_count);
        for _ in 0..nbr_count {
            let neighbor = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8]));
            neighbors.push(VertexId::from_u64(neighbor));
            offset += 8;
        }

        if offset + nbr_count * 8 > data.len() {
            return;
        }
        let mut edge_ids = Vec::with_capacity(nbr_count);
        for _ in 0..nbr_count {
            let edge_id = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8]));
            edge_ids.push(edge_id);
            offset += 8;
        }

        if offset + nbr_count * 4 > data.len() {
            return;
        }
        let mut prop_offsets = Vec::with_capacity(nbr_count);
        for _ in 0..nbr_count {
            let prop_offset = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap_or([0; 4]));
            prop_offsets.push(prop_offset);
            offset += 4;
        }

        if offset + nbr_count * 4 > data.len() {
            return;
        }
        let mut timestamps = Vec::with_capacity(nbr_count);
        for _ in 0..nbr_count {
            let timestamp = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap_or([0; 4]));
            timestamps.push(timestamp);
            offset += 4;
        }

        self.vertex_capacity = vertex_capacity;
        self.total_edge_capacity = total_edge_capacity;
        self.adj_offsets = adj_offsets;
        self.degrees = degrees;
        self.capacities = capacities;
        self.neighbors = neighbors;
        self.edge_ids = edge_ids;
        self.prop_offsets = prop_offsets;
        self.timestamps = timestamps;
        self.locks = (0..vertex_capacity).map(|_| SpinLock::new()).collect();
        self.edge_count = AtomicU64::new(0);
    }
}

impl MutableCsrTrait for CacheOptimizedCsr {
    fn insert_edge(
        &mut self,
        src: VertexId,
        dst: VertexId,
        edge_id: EdgeId,
        prop_offset: u32,
        ts: Timestamp,
    ) -> bool {
        self.insert_edge(src, dst, edge_id, prop_offset, ts)
    }

    fn delete_edge(&mut self, src: VertexId, edge_id: EdgeId, _ts: Timestamp) -> bool {
        let src_idx = src.as_int64().unwrap_or(0) as usize;
        if src_idx >= self.vertex_capacity {
            return false;
        }

        let _guard = SpinLockGuard::new(&self.locks[src_idx]);

        let degree = self.degrees[src_idx] as usize;
        let offset = self.adj_offsets[src_idx];

        for i in 0..degree {
            if self.edge_ids[offset + i] == edge_id && self.timestamps[offset + i] != INVALID_TIMESTAMP {
                self.timestamps[offset + i] = INVALID_TIMESTAMP;
                self.edge_count.fetch_sub(1, Ordering::Relaxed);
                return true;
            }
        }
        false
    }

    fn delete_edge_by_dst(&mut self, src: VertexId, dst: VertexId, _ts: Timestamp) -> bool {
        let src_idx = src.as_int64().unwrap_or(0) as usize;
        if src_idx >= self.vertex_capacity {
            return false;
        }

        let _guard = SpinLockGuard::new(&self.locks[src_idx]);

        let degree = self.degrees[src_idx] as usize;
        let offset = self.adj_offsets[src_idx];

        for i in 0..degree {
            if self.neighbors[offset + i] == dst && self.timestamps[offset + i] != INVALID_TIMESTAMP {
                self.timestamps[offset + i] = INVALID_TIMESTAMP;
                self.edge_count.fetch_sub(1, Ordering::Relaxed);
                return true;
            }
        }
        false
    }

    fn delete_edge_by_offset(&mut self, src: VertexId, offset_pos: i32, _ts: Timestamp) -> bool {
        let src_idx = src.as_int64().unwrap_or(0) as usize;
        if src_idx >= self.vertex_capacity {
            return false;
        }

        let _guard = SpinLockGuard::new(&self.locks[src_idx]);

        let base_offset = self.adj_offsets[src_idx];
        let idx = base_offset + offset_pos as usize;

        if idx >= self.timestamps.len() {
            return false;
        }

        if self.timestamps[idx] != INVALID_TIMESTAMP {
            self.timestamps[idx] = INVALID_TIMESTAMP;
            self.edge_count.fetch_sub(1, Ordering::Relaxed);
            return true;
        }
        false
    }

    fn revert_delete(&mut self, src: VertexId, edge_id: EdgeId, ts: Timestamp) -> bool {
        let src_idx = src.as_int64().unwrap_or(0) as usize;
        if src_idx >= self.vertex_capacity {
            return false;
        }

        let _guard = SpinLockGuard::new(&self.locks[src_idx]);

        let degree = self.degrees[src_idx] as usize;
        let offset = self.adj_offsets[src_idx];

        for i in 0..degree {
            if self.edge_ids[offset + i] == edge_id && self.timestamps[offset + i] == INVALID_TIMESTAMP {
                self.timestamps[offset + i] = ts;
                self.edge_count.fetch_add(1, Ordering::Relaxed);
                return true;
            }
        }
        false
    }

    fn revert_delete_by_offset(&mut self, src: VertexId, offset_pos: i32, ts: Timestamp) -> bool {
        let src_idx = src.as_int64().unwrap_or(0) as usize;
        if src_idx >= self.vertex_capacity {
            return false;
        }

        let _guard = SpinLockGuard::new(&self.locks[src_idx]);

        let base_offset = self.adj_offsets[src_idx];
        let idx = base_offset + offset_pos as usize;

        if idx >= self.timestamps.len() {
            return false;
        }

        if self.timestamps[idx] == INVALID_TIMESTAMP {
            self.timestamps[idx] = ts;
            self.edge_count.fetch_add(1, Ordering::Relaxed);
            return true;
        }
        false
    }

    fn get_edge(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> Option<Nbr> {
        let src_idx = src.as_int64().unwrap_or(0) as usize;
        if src_idx >= self.vertex_capacity {
            return None;
        }

        let degree = self.degrees[src_idx] as usize;
        let offset = self.adj_offsets[src_idx];

        for i in 0..degree {
            if self.neighbors[offset + i] == dst
                && self.timestamps[offset + i] <= ts
                && self.timestamps[offset + i] != INVALID_TIMESTAMP
            {
                return Some(Nbr {
                    neighbor: self.neighbors[offset + i],
                    edge_id: self.edge_ids[offset + i],
                    prop_offset: self.prop_offsets[offset + i],
                    timestamp: self.timestamps[offset + i],
                });
            }
        }
        None
    }

    fn edges_of(&self, src: VertexId, ts: Timestamp) -> Vec<Nbr> {
        self.edges_of(src, ts)
    }

    fn degree(&self, src: VertexId, ts: Timestamp) -> usize {
        let src_idx = src.as_int64().unwrap_or(0) as usize;
        if src_idx >= self.vertex_capacity {
            return 0;
        }

        let degree = self.degrees[src_idx] as usize;
        let offset = self.adj_offsets[src_idx];

        let mut count = 0;
        for i in 0..degree {
            if self.timestamps[offset + i] <= ts && self.timestamps[offset + i] != INVALID_TIMESTAMP {
                count += 1;
            }
        }
        count
    }

    fn has_edge(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> bool {
        let src_idx = src.as_int64().unwrap_or(0) as usize;
        if src_idx >= self.vertex_capacity {
            return false;
        }

        let degree = self.degrees[src_idx] as usize;
        let offset = self.adj_offsets[src_idx];

        for i in 0..degree {
            if self.neighbors[offset + i] == dst
                && self.timestamps[offset + i] <= ts
                && self.timestamps[offset + i] != INVALID_TIMESTAMP
            {
                return true;
            }
        }
        false
    }

    fn compact(&mut self) {
        let mut total_removed = 0u64;

        for src_idx in 0..self.vertex_capacity {
            let degree = self.degrees[src_idx] as usize;
            let offset = self.adj_offsets[src_idx];

            if degree == 0 {
                continue;
            }

            let mut write_idx = 0;
            for read_idx in 0..degree {
                if self.timestamps[offset + read_idx] != INVALID_TIMESTAMP {
                    if write_idx != read_idx {
                        self.neighbors[offset + write_idx] = self.neighbors[offset + read_idx];
                        self.edge_ids[offset + write_idx] = self.edge_ids[offset + read_idx];
                        self.prop_offsets[offset + write_idx] = self.prop_offsets[offset + read_idx];
                        self.timestamps[offset + write_idx] = self.timestamps[offset + read_idx];
                    }
                    write_idx += 1;
                }
            }

            let removed = degree - write_idx;
            if removed > 0 {
                self.degrees[src_idx] = write_idx as u32;
                total_removed += removed as u64;
            }
        }

        if total_removed > 0 {
            self.edge_count.fetch_sub(total_removed, Ordering::Relaxed);
        }
    }

    fn batch_put_edges(
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
            let dst = dst_list[i];
            let edge_id = edge_ids[i];
            let prop_offset = prop_offsets[i];

            let src_idx = src_list[i].as_int64().unwrap_or(0) as usize;
            let degree = self.degrees[src_idx] as usize;
            let capacity = self.capacities[src_idx] as usize;
            let offset = self.adj_offsets[src_idx];

            if degree < capacity {
                self.neighbors[offset + degree] = dst.clone();
                self.edge_ids[offset + degree] = edge_id;
                self.prop_offsets[offset + degree] = prop_offset;
                self.timestamps[offset + degree] = ts;
                self.degrees[src_idx] += 1;
                self.edge_count.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

impl CacheOptimizedCsr {
    pub fn find_deleted_edge(&self, src: VertexId, dst: VertexId) -> Option<EdgeId> {
        let src_idx = src.as_int64().unwrap_or(0) as usize;
        if src_idx >= self.vertex_capacity {
            return None;
        }

        let degree = self.degrees[src_idx] as usize;
        let offset = self.adj_offsets[src_idx];

        for i in 0..degree {
            if self.neighbors[offset + i] == dst && self.timestamps[offset + i] == INVALID_TIMESTAMP {
                return Some(self.edge_ids[offset + i]);
            }
        }
        None
    }

    pub fn used_memory_size(&self) -> usize {
        self.neighbors.len() * std::mem::size_of::<VertexId>()
            + self.edge_ids.len() * std::mem::size_of::<EdgeId>()
            + self.prop_offsets.len() * std::mem::size_of::<u32>()
            + self.timestamps.len() * std::mem::size_of::<Timestamp>()
            + self.adj_offsets.len() * std::mem::size_of::<usize>()
            + self.degrees.len() * std::mem::size_of::<u32>()
            + self.capacities.len() * std::mem::size_of::<u32>()
            + std::mem::size_of::<Self>()
    }

    pub fn compact_with_ts(&mut self, ts: Timestamp, reserve_ratio: f32) -> usize {
        let mut total_removed = 0usize;

        for src_idx in 0..self.vertex_capacity {
            let degree = self.degrees[src_idx] as usize;
            let offset = self.adj_offsets[src_idx];

            if degree == 0 {
                continue;
            }

            let mut write_idx = 0;
            for read_idx in 0..degree {
                let timestamp = self.timestamps[offset + read_idx];
                if timestamp != INVALID_TIMESTAMP && timestamp <= ts {
                    if write_idx != read_idx {
                        self.neighbors[offset + write_idx] = self.neighbors[offset + read_idx];
                        self.edge_ids[offset + write_idx] = self.edge_ids[offset + read_idx];
                        self.prop_offsets[offset + write_idx] = self.prop_offsets[offset + read_idx];
                        self.timestamps[offset + write_idx] = timestamp;
                    }
                    write_idx += 1;
                }
            }

            let removed = degree - write_idx;
            if removed > 0 {
                self.degrees[src_idx] = write_idx as u32;
                let new_capacity = (write_idx as f32 * (1.0 + reserve_ratio)).max(4.0) as usize;
                self.capacities[src_idx] = new_capacity as u32;
                total_removed += removed;
            }
        }

        if total_removed > 0 {
            self.edge_count.fetch_sub(total_removed as u64, Ordering::Relaxed);
        }
        total_removed
    }

    pub fn iter_edges(&self, src: VertexId, ts: Timestamp) -> CacheOptimizedCsrEdgeIterator<'_> {
        CacheOptimizedCsrEdgeIterator::new(self, src, ts)
    }

    pub fn iter(&self, ts: Timestamp) -> CacheOptimizedCsrIterator<'_> {
        CacheOptimizedCsrIterator::new(self, ts)
    }
}

pub struct CacheOptimizedCsrEdgeIterator<'a> {
    csr: &'a CacheOptimizedCsr,
    src: VertexId,
    ts: Timestamp,
    current_idx: usize,
    degree: usize,
}

impl<'a> CacheOptimizedCsrEdgeIterator<'a> {
    pub fn new(csr: &'a CacheOptimizedCsr, src: VertexId, ts: Timestamp) -> Self {
        let src_idx = src.as_int64().unwrap_or(0) as usize;
        let degree = if src_idx < csr.vertex_capacity {
            csr.degrees[src_idx] as usize
        } else {
            0
        };
        Self {
            csr,
            src,
            ts,
            current_idx: 0,
            degree,
        }
    }
}

impl<'a> Iterator for CacheOptimizedCsrEdgeIterator<'a> {
    type Item = Nbr;

    fn next(&mut self) -> Option<Self::Item> {
        let src_idx = self.src.as_int64().unwrap_or(0) as usize;
        if src_idx >= self.csr.vertex_capacity {
            return None;
        }

        let offset = self.csr.adj_offsets[src_idx];

        while self.current_idx < self.degree {
            let idx = offset + self.current_idx;
            self.current_idx += 1;

            let timestamp = self.csr.timestamps[idx];
            if timestamp <= self.ts && timestamp != INVALID_TIMESTAMP {
                return Some(Nbr {
                    neighbor: self.csr.neighbors[idx],
                    edge_id: self.csr.edge_ids[idx],
                    prop_offset: self.csr.prop_offsets[idx],
                    timestamp,
                });
            }
        }
        None
    }
}

pub struct CacheOptimizedCsrIterator<'a> {
    csr: &'a CacheOptimizedCsr,
    current_vertex: usize,
    ts: Timestamp,
}

impl<'a> CacheOptimizedCsrIterator<'a> {
    pub fn new(csr: &'a CacheOptimizedCsr, ts: Timestamp) -> Self {
        Self {
            csr,
            current_vertex: 0,
            ts,
        }
    }
}

impl<'a> Iterator for CacheOptimizedCsrIterator<'a> {
    type Item = (VertexId, Nbr);

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_vertex < self.csr.vertex_capacity {
            let src = VertexId::from_int64(self.current_vertex as i64);
            let degree = self.csr.degrees[self.current_vertex] as usize;
            let offset = self.csr.adj_offsets[self.current_vertex];
            self.current_vertex += 1;

            for i in 0..degree {
                let idx = offset + i;
                let timestamp = self.csr.timestamps[idx];
                if timestamp <= self.ts && timestamp != INVALID_TIMESTAMP {
                    return Some((
                        src,
                        Nbr {
                            neighbor: self.csr.neighbors[idx],
                            edge_id: self.csr.edge_ids[idx],
                            prop_offset: self.csr.prop_offsets[idx],
                            timestamp,
                        },
                    ));
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut csr = CacheOptimizedCsr::new();

        // Insert edges
        assert!(csr.insert_edge(VertexId::from_int64(0), VertexId::from_int64(1), 100, 0, 10));
        assert!(csr.insert_edge(VertexId::from_int64(0), VertexId::from_int64(2), 101, 1, 10));
        assert!(csr.insert_edge(VertexId::from_int64(1), VertexId::from_int64(2), 102, 2, 10));

        // Check edge count
        assert_eq!(csr.edge_count(), 3);

        // Get edges
        let edges = csr.edges_of(VertexId::from_int64(0), 10);
        assert_eq!(edges.len(), 2);

        // Delete edge
        assert!(csr.delete_edge(VertexId::from_int64(0), 100, 20));
        assert_eq!(csr.edge_count(), 2);
    }

    #[test]
    fn test_simd_optimization() {
        let mut csr = CacheOptimizedCsr::new();

        // Insert many edges for one vertex
        for i in 0..100 {
            csr.insert_edge(VertexId::from_int64(0), VertexId::from_int64(i as i64), i as u64, 0, 10);
        }

        // Test SIMD version
        let edges_simd = csr.edges_of_simd(VertexId::from_int64(0), 10);
        let edges_normal = csr.edges_of(VertexId::from_int64(0), 10);

        assert_eq!(edges_simd.len(), edges_normal.len());
    }

    #[test]
    fn test_parallel_insert() {
        let mut csr = CacheOptimizedCsr::new();

        let src_list: Vec<VertexId> = (0..1000).map(VertexId::from_int64).collect();
        let dst_list: Vec<VertexId> = (1000..2000).map(VertexId::from_int64).collect();
        let edge_ids: Vec<EdgeId> = (0..1000).collect();
        let prop_offsets: Vec<u32> = vec![0; 1000];

        csr.batch_insert_parallel(&src_list, &dst_list, &edge_ids, &prop_offsets, 10);

        assert_eq!(csr.edge_count(), 1000);
    }
}
