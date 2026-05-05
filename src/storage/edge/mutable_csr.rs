//! Mutable CSR Implementation
//!
//! Mutable CSR with contiguous storage for memory efficiency and cache locality.
//! Uses per-vertex spin locks for fine-grained concurrency.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use super::{EdgeId, Nbr, Timestamp, VertexId, INVALID_TIMESTAMP};

const DEFAULT_VERTEX_CAPACITY: usize = 1024;
const DEFAULT_EDGE_CAPACITY: usize = 4096;
const DEFAULT_VERTEX_DEGREE: usize = 4;

/// Spin lock for per-vertex locking
#[derive(Debug)]
pub struct SpinLock {
    locked: AtomicBool,
}

impl SpinLock {
    pub fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
        }
    }

    #[inline]
    pub fn lock(&self) {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            std::hint::spin_loop();
        }
    }

    #[inline]
    pub fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }

    #[inline]
    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Acquire)
    }
}

impl Default for SpinLock {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII guard for spin lock
pub struct SpinLockGuard<'a> {
    lock: &'a SpinLock,
}

impl<'a> SpinLockGuard<'a> {
    #[inline]
    pub fn new(lock: &'a SpinLock) -> Self {
        lock.lock();
        Self { lock }
    }
}

impl<'a> Drop for SpinLockGuard<'a> {
    #[inline]
    fn drop(&mut self) {
        self.lock.unlock();
    }
}

/// Mutable CSR with contiguous storage
///
/// Memory layout (inspired by neug/GraphScope):
/// - `nbr_list`: Contiguous array of all neighbors
/// - `adj_offsets`: Offset into nbr_list for each vertex
/// - `degrees`: Current edge count per vertex
/// - `capacities`: Allocated capacity per vertex
/// - `locks`: Per-vertex spin lock for concurrency
#[derive(Debug)]
pub struct MutableCsr {
    nbr_list: Vec<Nbr>,
    adj_offsets: Vec<usize>,
    degrees: Vec<u32>,
    capacities: Vec<u32>,
    locks: Vec<SpinLock>,
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
            capacities: self.capacities.clone(),
            locks: (0..self.vertex_capacity).map(|_| SpinLock::new()).collect(),
            edge_count: AtomicU64::new(self.edge_count.load(Ordering::Relaxed)),
            vertex_capacity: self.vertex_capacity,
            total_edge_capacity: self.total_edge_capacity,
        }
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
        let mut capacities = Vec::with_capacity(vertex_cap);

        let mut offset = 0usize;
        for _ in 0..vertex_cap {
            adj_offsets.push(offset);
            capacities.push(DEFAULT_VERTEX_DEGREE as u32);
            offset += DEFAULT_VERTEX_DEGREE;
        }

        nbr_list.resize(offset, Nbr::new(0, 0, 0, INVALID_TIMESTAMP));

        Self {
            nbr_list,
            adj_offsets,
            degrees: vec![0; vertex_cap],
            capacities,
            locks: (0..vertex_cap).map(|_| SpinLock::new()).collect(),
            edge_count: AtomicU64::new(0),
            vertex_capacity: vertex_cap,
            total_edge_capacity: offset,
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
            self.adj_offsets.push(new_total_capacity);
            self.capacities.push(DEFAULT_VERTEX_DEGREE as u32);
            self.degrees.push(0);
            self.locks.push(SpinLock::new());
            new_total_capacity += DEFAULT_VERTEX_DEGREE;
        }

        self.nbr_list
            .resize(new_total_capacity, Nbr::new(0, 0, 0, INVALID_TIMESTAMP));
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

    /// Get the start index in nbr_list for a vertex
    #[inline]
    fn nbr_start(&self, src: VertexId) -> usize {
        let src_idx = src as usize;
        if src_idx < self.adj_offsets.len() {
            self.adj_offsets[src_idx]
        } else {
            0
        }
    }

    /// Get pointer to neighbor array for a vertex
    #[inline]
    fn nbr_ptr(&self, src: VertexId) -> *const Nbr {
        let start = self.nbr_start(src);
        self.nbr_list.as_ptr().wrapping_add(start)
    }

    /// Get mutable pointer to neighbor array for a vertex
    #[inline]
    fn nbr_ptr_mut(&mut self, src: VertexId) -> *mut Nbr {
        let start = self.nbr_start(src);
        self.nbr_list.as_mut_ptr().wrapping_add(start)
    }

    /// Insert an edge
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
            self.ensure_vertex_capacity(src_idx + 1);
        }

        let _guard = SpinLockGuard::new(&self.locks[src_idx]);

        let degree = self.degrees[src_idx] as usize;
        let capacity = self.capacities[src_idx] as usize;
        let offset = self.adj_offsets[src_idx];

        for i in 0..degree {
            let nbr = &self.nbr_list[offset + i];
            if nbr.neighbor == dst && nbr.timestamp != INVALID_TIMESTAMP {
                return false;
            }
        }

        if degree >= capacity {
            return false;
        }

        self.nbr_list[offset + degree] = Nbr::new(dst, edge_id, prop_offset, ts);
        self.degrees[src_idx] += 1;
        self.edge_count.fetch_add(1, Ordering::Relaxed);
        true
    }

    /// Insert edge with automatic capacity expansion
    /// Note: This method may need to release and reacquire locks during expansion
    pub fn insert_edge_with_expand(
        &mut self,
        src: VertexId,
        dst: VertexId,
        edge_id: EdgeId,
        prop_offset: u32,
        ts: Timestamp,
    ) -> bool {
        let src_idx = src as usize;

        if src_idx >= self.vertex_capacity {
            self.ensure_vertex_capacity(src_idx + 1);
        }

        {
            let _guard = SpinLockGuard::new(&self.locks[src_idx]);

            let degree = self.degrees[src_idx] as usize;
            let capacity = self.capacities[src_idx] as usize;
            let offset = self.adj_offsets[src_idx];

            for i in 0..degree {
                let nbr = &self.nbr_list[offset + i];
                if nbr.neighbor == dst && nbr.timestamp != INVALID_TIMESTAMP {
                    return false;
                }
            }

            if degree < capacity {
                self.nbr_list[offset + degree] = Nbr::new(dst, edge_id, prop_offset, ts);
                self.degrees[src_idx] += 1;
                self.edge_count.fetch_add(1, Ordering::Relaxed);
                return true;
            }
        }

        self.expand_vertex_capacity(src_idx);

        {
            let _guard = SpinLockGuard::new(&self.locks[src_idx]);
            let degree = self.degrees[src_idx] as usize;
            let offset = self.adj_offsets[src_idx];
            self.nbr_list[offset + degree] = Nbr::new(dst, edge_id, prop_offset, ts);
            self.degrees[src_idx] += 1;
            self.edge_count.fetch_add(1, Ordering::Relaxed);
        }

        true
    }

    /// Expand capacity for a specific vertex (requires exclusive access)
    fn expand_vertex_capacity(&mut self, src_idx: usize) {
        let old_capacity = self.capacities[src_idx] as usize;
        let new_capacity = ((old_capacity as f64) * 1.5).max(4.0) as usize;
        let additional = new_capacity - old_capacity;

        let insert_pos = self.adj_offsets[src_idx] + old_capacity;
        self.nbr_list.splice(
            insert_pos..insert_pos,
            std::iter::repeat(Nbr::new(0, 0, 0, INVALID_TIMESTAMP)).take(additional),
        );

        for i in (src_idx + 1)..self.vertex_capacity {
            self.adj_offsets[i] += additional;
        }

        self.capacities[src_idx] = new_capacity as u32;
        self.total_edge_capacity += additional;
    }

    /// Delete an edge by edge_id
    pub fn delete_edge(&mut self, src: VertexId, edge_id: EdgeId, ts: Timestamp) -> bool {
        let src_idx = src as usize;
        if src_idx >= self.vertex_capacity {
            return false;
        }

        let _guard = SpinLockGuard::new(&self.locks[src_idx]);

        let degree = self.degrees[src_idx] as usize;
        let offset = self.adj_offsets[src_idx];

        for i in 0..degree {
            let nbr = &mut self.nbr_list[offset + i];
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

    /// Delete edge by destination vertex
    pub fn delete_edge_by_dst(&mut self, src: VertexId, dst: VertexId, ts: Timestamp) -> bool {
        let src_idx = src as usize;
        if src_idx >= self.vertex_capacity {
            return false;
        }

        let _guard = SpinLockGuard::new(&self.locks[src_idx]);

        let degree = self.degrees[src_idx] as usize;
        let offset = self.adj_offsets[src_idx];
        let mut deleted = false;

        for i in 0..degree {
            let nbr = &mut self.nbr_list[offset + i];
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

    /// Revert a deleted edge
    pub fn revert_delete(&mut self, src: VertexId, edge_id: EdgeId, ts: Timestamp) -> bool {
        let src_idx = src as usize;
        if src_idx >= self.vertex_capacity {
            return false;
        }

        let _guard = SpinLockGuard::new(&self.locks[src_idx]);

        let degree = self.degrees[src_idx] as usize;
        let offset = self.adj_offsets[src_idx];

        for i in 0..degree {
            let nbr = &mut self.nbr_list[offset + i];
            if nbr.edge_id == edge_id && nbr.timestamp == INVALID_TIMESTAMP {
                nbr.timestamp = ts;
                self.edge_count.fetch_add(1, Ordering::Relaxed);
                return true;
            }
        }
        false
    }

    /// Find a deleted edge by destination
    pub fn find_deleted_edge(&self, src: VertexId, dst: VertexId) -> Option<EdgeId> {
        let src_idx = src as usize;
        if src_idx >= self.vertex_capacity {
            return None;
        }

        let degree = self.degrees[src_idx] as usize;
        let offset = self.adj_offsets[src_idx];

        for i in 0..degree {
            let nbr = &self.nbr_list[offset + i];
            if nbr.neighbor == dst && nbr.timestamp == INVALID_TIMESTAMP {
                return Some(nbr.edge_id);
            }
        }
        None
    }

    /// Get edges of a vertex at a given timestamp
    pub fn edges_of(&self, src: VertexId, ts: Timestamp) -> Vec<Nbr> {
        let src_idx = src as usize;
        if src_idx >= self.vertex_capacity {
            return Vec::new();
        }

        let degree = self.degrees[src_idx] as usize;
        let offset = self.adj_offsets[src_idx];

        let mut result = Vec::with_capacity(degree);
        for i in 0..degree {
            let nbr = &self.nbr_list[offset + i];
            if nbr.timestamp <= ts && nbr.timestamp != INVALID_TIMESTAMP {
                result.push(*nbr);
            }
        }
        result
    }

    /// Get degree of a vertex at a given timestamp
    pub fn degree(&self, src: VertexId, ts: Timestamp) -> usize {
        let src_idx = src as usize;
        if src_idx >= self.vertex_capacity {
            return 0;
        }

        let degree = self.degrees[src_idx] as usize;
        let offset = self.adj_offsets[src_idx];

        let mut count = 0;
        for i in 0..degree {
            let nbr = &self.nbr_list[offset + i];
            if nbr.timestamp <= ts && nbr.timestamp != INVALID_TIMESTAMP {
                count += 1;
            }
        }
        count
    }

    /// Check if an edge exists
    pub fn has_edge(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> bool {
        let src_idx = src as usize;
        if src_idx >= self.vertex_capacity {
            return false;
        }

        let degree = self.degrees[src_idx] as usize;
        let offset = self.adj_offsets[src_idx];

        for i in 0..degree {
            let nbr = &self.nbr_list[offset + i];
            if nbr.neighbor == dst && nbr.timestamp <= ts && nbr.timestamp != INVALID_TIMESTAMP {
                return true;
            }
        }
        false
    }

    /// Get a specific edge
    pub fn get_edge(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> Option<Nbr> {
        let src_idx = src as usize;
        if src_idx >= self.vertex_capacity {
            return None;
        }

        let degree = self.degrees[src_idx] as usize;
        let offset = self.adj_offsets[src_idx];

        for i in 0..degree {
            let nbr = &self.nbr_list[offset + i];
            if nbr.neighbor == dst && nbr.timestamp <= ts && nbr.timestamp != INVALID_TIMESTAMP {
                return Some(*nbr);
            }
        }
        None
    }

    /// Get edge by edge_id
    pub fn get_edge_by_id(&self, src: VertexId, edge_id: EdgeId, ts: Timestamp) -> Option<Nbr> {
        let src_idx = src as usize;
        if src_idx >= self.vertex_capacity {
            return None;
        }

        let degree = self.degrees[src_idx] as usize;
        let offset = self.adj_offsets[src_idx];

        for i in 0..degree {
            let nbr = &self.nbr_list[offset + i];
            if nbr.edge_id == edge_id && nbr.timestamp <= ts && nbr.timestamp != INVALID_TIMESTAMP {
                return Some(*nbr);
            }
        }
        None
    }

    /// Clear all edges
    pub fn clear(&mut self) {
        for nbr in &mut self.nbr_list {
            *nbr = Nbr::new(0, 0, 0, INVALID_TIMESTAMP);
        }
        for degree in &mut self.degrees {
            *degree = 0;
        }
        self.edge_count.store(0, Ordering::Relaxed);
    }

    /// Compact: remove deleted edges (tombstones)
    pub fn compact(&mut self) {
        let mut total_removed = 0u64;

        for src_idx in 0..self.vertex_capacity {
            let degree = self.degrees[src_idx] as usize;
            let offset = self.adj_offsets[src_idx];

            if degree == 0 {
                continue;
            }

            let mut write_idx = 0;
            for read_idx in 0..degree {
                let nbr = self.nbr_list[offset + read_idx];
                if nbr.timestamp != INVALID_TIMESTAMP {
                    if write_idx != read_idx {
                        self.nbr_list[offset + write_idx] = nbr;
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

    /// Batch insert edges
    pub fn batch_put_edges(
        &mut self,
        src_list: &[VertexId],
        dst_list: &[VertexId],
        edge_ids: &[EdgeId],
        prop_offsets: &[u32],
        ts: Timestamp,
    ) {
        let max_vertex = src_list.iter().max().copied().unwrap_or(0) as usize;
        self.ensure_vertex_capacity(max_vertex + 1);

        for i in 0..src_list.len() {
            let src = src_list[i];
            let dst = dst_list[i];
            let edge_id = edge_ids[i];
            let prop_offset = prop_offsets[i];

            let src_idx = src as usize;
            let degree = self.degrees[src_idx] as usize;
            let capacity = self.capacities[src_idx] as usize;
            let offset = self.adj_offsets[src_idx];

            if degree < capacity {
                self.nbr_list[offset + degree] = Nbr::new(dst, edge_id, prop_offset, ts);
                self.degrees[src_idx] += 1;
                self.edge_count.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Batch delete edges
    pub fn batch_delete_edges(&mut self, edges: &[(VertexId, EdgeId)], ts: Timestamp) {
        for &(src, edge_id) in edges {
            self.delete_edge(src, edge_id, ts);
        }
    }

    /// Create iterator over all edges
    pub fn iter(&self, ts: Timestamp) -> MutableCsrIterator {
        MutableCsrIterator::new(self, ts)
    }

    /// Create iterator over edges of a specific vertex
    pub fn iter_edges(&self, src: VertexId, ts: Timestamp) -> MutableCsrEdgeIterator {
        MutableCsrEdgeIterator::new(self, src, ts)
    }

    /// Dump to bytes
    pub fn dump(&self) -> Vec<u8> {
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

        for nbr in &self.nbr_list {
            result.extend_from_slice(&nbr.neighbor.to_le_bytes());
            result.extend_from_slice(&nbr.edge_id.to_le_bytes());
            result.extend_from_slice(&nbr.prop_offset.to_le_bytes());
            result.extend_from_slice(&nbr.timestamp.to_le_bytes());
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

        let total_edge_capacity =
            u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8])) as usize;
        offset += 8;

        if offset + vertex_capacity * 8 > data.len() {
            return;
        }
        let mut adj_offsets = Vec::with_capacity(vertex_capacity);
        for _ in 0..vertex_capacity {
            let off =
                u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8])) as usize;
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
        if offset + nbr_count * 24 > data.len() {
            return;
        }
        let mut nbr_list = Vec::with_capacity(nbr_count);
        for _ in 0..nbr_count {
            let neighbor =
                u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8]));
            offset += 8;
            let edge_id = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or([0; 8]));
            offset += 8;
            let prop_offset =
                u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap_or([0; 4]));
            offset += 4;
            let timestamp =
                u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap_or([0; 4]));
            offset += 4;

            nbr_list.push(Nbr {
                neighbor,
                edge_id,
                prop_offset,
                timestamp,
            });
        }

        self.vertex_capacity = vertex_capacity;
        self.total_edge_capacity = total_edge_capacity;
        self.adj_offsets = adj_offsets;
        self.degrees = degrees;
        self.capacities = capacities;
        self.nbr_list = nbr_list;
        self.locks = (0..vertex_capacity).map(|_| SpinLock::new()).collect();
        self.edge_count.store(edge_count, Ordering::Relaxed);
    }

    /// Get raw neighbor slice (for internal use)
    pub fn nbr_slice(&self) -> &[Nbr] {
        &self.nbr_list
    }

    /// Get raw neighbor slice mut (for internal use)
    pub fn nbr_slice_mut(&mut self) -> &mut [Nbr] {
        &mut self.nbr_list
    }

    /// Get degrees slice
    pub fn degrees(&self) -> &[u32] {
        &self.degrees
    }

    /// Get capacities slice
    pub fn capacities(&self) -> &[u32] {
        &self.capacities
    }

    /// Get adj_offsets slice
    pub fn adj_offsets(&self) -> &[usize] {
        &self.adj_offsets
    }

    /// Load from parts (for persistence module)
    pub fn load_from_parts(
        &mut self,
        nbr_list: Vec<Nbr>,
        adj_offsets: Vec<usize>,
        degrees: Vec<u32>,
        capacities: Vec<u32>,
        vertex_capacity: usize,
        total_edge_capacity: usize,
        edge_count: u64,
    ) {
        self.nbr_list = nbr_list;
        self.adj_offsets = adj_offsets;
        self.degrees = degrees;
        self.capacities = capacities;
        self.vertex_capacity = vertex_capacity;
        self.total_edge_capacity = total_edge_capacity;
        self.locks = (0..vertex_capacity).map(|_| SpinLock::new()).collect();
        self.edge_count.store(edge_count, Ordering::Relaxed);
    }
}

impl Default for MutableCsr {
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator over all edges in the CSR
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
    type Item = (VertexId, Nbr);

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_vertex < self.csr.vertex_capacity {
            let degree = self.csr.degrees[self.current_vertex] as usize;
            let offset = self.csr.adj_offsets[self.current_vertex];

            while self.current_edge < degree {
                let nbr = self.csr.nbr_list[offset + self.current_edge];
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

/// Iterator over edges of a specific vertex
pub struct MutableCsrEdgeIterator<'a> {
    csr: &'a MutableCsr,
    ts: Timestamp,
    offset: usize,
    degree: usize,
    current: usize,
}

impl<'a> MutableCsrEdgeIterator<'a> {
    pub fn new(csr: &'a MutableCsr, src: VertexId, ts: Timestamp) -> Self {
        let src_idx = src as usize;
        let (offset, degree) = if src_idx < csr.vertex_capacity {
            (csr.adj_offsets[src_idx], csr.degrees[src_idx] as usize)
        } else {
            (0, 0)
        };

        Self {
            csr,
            ts,
            offset,
            degree,
            current: 0,
        }
    }
}

impl<'a> Iterator for MutableCsrEdgeIterator<'a> {
    type Item = Nbr;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current < self.degree {
            let nbr = self.csr.nbr_list[self.offset + self.current];
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
    fn test_spin_lock() {
        let lock = SpinLock::new();
        assert!(!lock.is_locked());

        {
            let _guard = SpinLockGuard::new(&lock);
            assert!(lock.is_locked());
        }

        assert!(!lock.is_locked());
    }

    #[test]
    fn test_basic_insert_and_query() {
        let mut csr = MutableCsr::with_capacity(10, 100);

        assert!(csr.insert_edge(0, 1, 100, 0, 1));
        assert!(csr.insert_edge(0, 2, 101, 0, 1));
        assert!(csr.insert_edge(1, 3, 102, 0, 1));

        assert!(!csr.insert_edge(0, 1, 103, 0, 1));

        assert_eq!(csr.degree(0, 1), 2);
        assert_eq!(csr.degree(1, 1), 1);
        assert_eq!(csr.degree(2, 1), 0);

        assert!(csr.has_edge(0, 1, 1));
        assert!(csr.has_edge(0, 2, 1));
        assert!(csr.has_edge(1, 3, 1));
        assert!(!csr.has_edge(0, 3, 1));

        assert_eq!(csr.edge_count(), 3);
    }

    #[test]
    fn test_delete_edge() {
        let mut csr = MutableCsr::with_capacity(10, 100);

        csr.insert_edge(0, 1, 100, 0, 1);
        csr.insert_edge(0, 2, 101, 0, 1);

        assert!(csr.delete_edge(0, 100, 2));
        assert!(!csr.has_edge(0, 1, 2));
        assert!(csr.has_edge(0, 2, 2));

        assert_eq!(csr.edge_count(), 1);
    }

    #[test]
    fn test_revert_delete() {
        let mut csr = MutableCsr::with_capacity(10, 100);

        csr.insert_edge(0, 1, 100, 0, 1);
        csr.delete_edge(0, 100, 2);

        assert!(csr.revert_delete(0, 100, 3));
        assert!(csr.has_edge(0, 1, 3));
    }

    #[test]
    fn test_compact() {
        let mut csr = MutableCsr::with_capacity(10, 100);

        csr.insert_edge(0, 1, 100, 0, 1);
        csr.insert_edge(0, 2, 101, 0, 1);
        csr.insert_edge(0, 3, 102, 0, 1);

        csr.delete_edge(0, 101, 2);

        csr.compact();

        let edges: Vec<_> = csr.iter_edges(0, 3).collect();
        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn test_dump_and_load() {
        let mut csr1 = MutableCsr::with_capacity(10, 100);

        csr1.insert_edge(0, 1, 100, 0, 1);
        csr1.insert_edge(0, 2, 101, 0, 1);
        csr1.insert_edge(1, 3, 102, 0, 1);

        let data = csr1.dump();

        let mut csr2 = MutableCsr::new();
        csr2.load(&data);

        assert_eq!(csr2.vertex_capacity(), csr1.vertex_capacity());
        assert_eq!(csr2.edge_count(), csr1.edge_count());
        assert!(csr2.has_edge(0, 1, 1));
        assert!(csr2.has_edge(0, 2, 1));
        assert!(csr2.has_edge(1, 3, 1));
    }

    #[test]
    fn test_resize() {
        let mut csr = MutableCsr::with_capacity(2, 10);

        csr.insert_edge(0, 1, 100, 0, 1);
        csr.insert_edge(100, 1, 101, 0, 1);

        assert!(csr.vertex_capacity() >= 101);
        assert!(csr.has_edge(100, 1, 1));
    }

    #[test]
    fn test_iterator() {
        let mut csr = MutableCsr::with_capacity(10, 100);

        csr.insert_edge(0, 1, 100, 0, 1);
        csr.insert_edge(0, 2, 101, 0, 1);
        csr.insert_edge(1, 3, 102, 0, 1);

        let edges: Vec<_> = csr.iter(1).collect();
        assert_eq!(edges.len(), 3);
    }
}
