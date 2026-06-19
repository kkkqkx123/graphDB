//! Immutable CSR Implementation
//!
//! Optimized read-only CSR for batch-loaded or snapshotted data.
//! Used for static graph analysis, fast lookups, and serialization.
//!
//! ## Design
//!
//! - **Flat layout**: No primary/overflow separation, no fragmentation
//! - **Compact storage**: No reserved capacity or atomic fields
//! - **Zero-copy reads**: Direct access via offsets without timestamp filtering
//! - **Build-only mutation**: `batch_put_edge` for construction phase only
//!
//! ## Usage
//!
//! ```ignore
//! // Convert from mutable snapshot
//! let immutable = ImmutableCsr::from_snapshot(&mutable_csr, snapshot_ts);
//!
//! // Or build from scratch
//! let mut builder = ImmutableCsr::builder(1000);
//! builder.batch_put_edge(0, VertexId::from_int64(1), EdgeId(100), 0);
//! let immutable = builder.build();
//!
//! // Fast lookup (no timestamp filtering)
//! let edges = immutable.edges_of(0);
//! ```

use crate::core::{StorageError, StorageResult};
use crate::storage::persistence::{read_u32_le, read_u64_le};

use super::{CsrBase, EdgeId, MutableCsr, MutableCsrTrait, Nbr, Timestamp, VertexId};

/// Immutable CSR: Compact, read-only graph structure
///
/// Suitable for:
/// - Static snapshots of mutable graphs
/// - Batch-loaded data
/// - Persistent storage format
/// - Fast analytical queries
pub struct ImmutableCsr {
    /// Contiguous edge storage, flat layout (no overflow)
    nbr_list: Box<[Nbr]>,

    /// Offset of first edge for each vertex
    /// Length = vertex_capacity + 1 (last entry is total edge count)
    adj_offsets: Box<[u32]>,

    /// Actual edge count per vertex (not capacity)
    /// Used to determine the range [offset[v], offset[v] + degree[v])
    degrees: Box<[u32]>,

    /// Total number of vertices
    vertex_capacity: usize,
}

impl Clone for ImmutableCsr {
    fn clone(&self) -> Self {
        Self {
            nbr_list: self.nbr_list.clone(),
            adj_offsets: self.adj_offsets.clone(),
            degrees: self.degrees.clone(),
            vertex_capacity: self.vertex_capacity,
        }
    }
}

impl std::fmt::Debug for ImmutableCsr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImmutableCsr")
            .field("vertex_capacity", &self.vertex_capacity)
            .field("edge_count", &self.nbr_list.len())
            .finish_non_exhaustive()
    }
}

impl ImmutableCsr {
    /// Create a new immutable CSR from raw components
    ///
    /// # Safety
    ///
    /// Caller must ensure consistency:
    /// - `adj_offsets.len() == vertex_capacity`
    /// - `degrees.len() == vertex_capacity`
    /// - All offsets and degrees are within bounds
    pub fn new(
        nbr_list: Box<[Nbr]>,
        adj_offsets: Box<[u32]>,
        degrees: Box<[u32]>,
        vertex_capacity: usize,
    ) -> Self {
        Self {
            nbr_list,
            adj_offsets,
            degrees,
            vertex_capacity,
        }
    }

    /// Build an immutable CSR from a mutable one, filtering by timestamp
    ///
    /// This merges primary and overflow blocks, removes deleted edges (timestamp > ts),
    /// and produces a compact flat layout.
    ///
    /// # Arguments
    /// - `mutable`: Source mutable CSR
    /// - `ts`: Timestamp cutoff (only edges with timestamp <= ts are included)
    pub fn from_snapshot(mutable: &MutableCsr, ts: Timestamp) -> Self {
        let vertex_capacity = mutable.vertex_capacity();

        // Phase 1: Collect valid edges per vertex
        let mut vertex_edges: Vec<Vec<Nbr>> = vec![Vec::new(); vertex_capacity];

        for src_idx in 0..vertex_capacity {
            let edges = mutable.edges_of(src_idx as u32, ts);
            vertex_edges[src_idx] = edges;
        }

        // Phase 2: Build flat layout
        let mut nbr_list = Vec::new();
        let mut adj_offsets = Vec::with_capacity(vertex_capacity);
        let mut degrees = Vec::with_capacity(vertex_capacity);

        for edges in &vertex_edges {
            adj_offsets.push(nbr_list.len() as u32);
            degrees.push(edges.len() as u32);
            nbr_list.extend_from_slice(edges);
        }

        Self {
            nbr_list: nbr_list.into_boxed_slice(),
            adj_offsets: adj_offsets.into_boxed_slice(),
            degrees: degrees.into_boxed_slice(),
            vertex_capacity,
        }
    }

    /// Create a builder for batch construction
    pub fn builder(vertex_capacity: usize) -> ImmutableCsrBuilder {
        ImmutableCsrBuilder::new(vertex_capacity)
    }

    /// Get edges of a vertex as a reference (no timestamp filtering)
    ///
    /// Returns a slice of all edges for the given vertex.
    pub fn edges_of_ref(&self, src_vid: u32) -> &[Nbr] {
        let src_idx = src_vid as usize;
        if src_idx >= self.vertex_capacity {
            return &[];
        }

        let offset = self.adj_offsets[src_idx] as usize;
        let degree = self.degrees[src_idx] as usize;
        &self.nbr_list[offset..offset + degree]
    }

    /// Get edges of a vertex as a vector (for MutableCsrTrait compatibility)
    pub fn edges_of(&self, src_vid: u32) -> Vec<Nbr> {
        self.edges_of_ref(src_vid).to_vec()
    }

    /// Get a specific edge by destination vertex (no timestamp filtering)
    ///
    /// This is the reference implementation; use with care.
    pub fn get_edge_unchecked(&self, src_vid: u32, dst: VertexId) -> Option<Nbr> {
        self.edges_of_ref(src_vid)
            .iter()
            .copied()
            .find(|nbr| nbr.neighbor == dst)
    }

    /// Get a specific edge by destination vertex
    ///
    /// For MutableCsrTrait compatibility. Timestamp parameter is ignored
    /// since ImmutableCsr does not support versioning.
    pub fn get_edge(&self, src_vid: u32, dst: VertexId, _ts: Timestamp) -> Option<Nbr> {
        self.get_edge_unchecked(src_vid, dst)
    }

    /// Count total edges
    pub fn edge_count(&self) -> u64 {
        self.nbr_list.len() as u64
    }

    /// Get memory usage (in bytes)
    pub fn used_memory_size(&self) -> usize {
        let edges_size = self.nbr_list.len() * std::mem::size_of::<Nbr>();
        let offsets_size = self.adj_offsets.len() * std::mem::size_of::<u32>();
        let degrees_size = self.degrees.len() * std::mem::size_of::<u32>();
        edges_size + offsets_size + degrees_size + std::mem::size_of::<Self>()
    }

    /// Dump to bytes for persistence
    ///
    /// Format:
    /// - vertex_capacity (u64)
    /// - edge_count (u64)
    /// - adj_offsets (u32 * vertex_capacity)
    /// - degrees (u32 * vertex_capacity)
    /// - nbr_list (Nbr * edge_count)
    pub fn dump(&self) -> Vec<u8> {
        let mut result = Vec::new();

        result.extend_from_slice(&(self.vertex_capacity as u64).to_le_bytes());
        result.extend_from_slice(&(self.nbr_list.len() as u64).to_le_bytes());

        for &offset in self.adj_offsets.iter() {
            result.extend_from_slice(&offset.to_le_bytes());
        }

        for &degree in self.degrees.iter() {
            result.extend_from_slice(&degree.to_le_bytes());
        }

        for nbr in self.nbr_list.iter() {
            write_vertex_id(&mut result, nbr.neighbor);
            result.extend_from_slice(&nbr.edge_id.to_le_bytes());
            result.extend_from_slice(&nbr.prop_offset.to_le_bytes());
            result.extend_from_slice(&nbr.create_ts.to_le_bytes());
            result.extend_from_slice(&nbr.delete_ts.to_le_bytes());
        }

        result
    }

    /// Load from bytes
    pub fn load(&mut self, data: &[u8]) -> StorageResult<()> {
        if data.len() < 16 {
            return Err(StorageError::deserialize_error(
                "ImmutableCsr data too short for header",
            ));
        }

        let mut offset = 0;
        let vertex_capacity = read_u64_le(data, &mut offset)? as usize;
        let edge_count = read_u64_le(data, &mut offset)? as usize;

        let mut adj_offsets = Vec::with_capacity(vertex_capacity);
        for _ in 0..vertex_capacity {
            adj_offsets.push(read_u32_le(data, &mut offset)?);
        }

        let mut degrees = Vec::with_capacity(vertex_capacity);
        for _ in 0..vertex_capacity {
            degrees.push(read_u32_le(data, &mut offset)?);
        }

        let mut nbr_list = Vec::with_capacity(edge_count);
        for _ in 0..edge_count {
            let neighbor = read_vertex_id(data, &mut offset)?;
            let raw_edge_id = read_u64_le(data, &mut offset)?;
            let prop_offset = read_u32_le(data, &mut offset)?;
            let create_ts = read_u32_le(data, &mut offset)?;
            let delete_ts = read_u32_le(data, &mut offset)?;

            nbr_list.push(Nbr::with_delete_ts(
                neighbor,
                EdgeId(raw_edge_id),
                prop_offset,
                create_ts,
                delete_ts,
            ));
        }

        *self = Self {
            nbr_list: nbr_list.into_boxed_slice(),
            adj_offsets: adj_offsets.into_boxed_slice(),
            degrees: degrees.into_boxed_slice(),
            vertex_capacity,
        };

        Ok(())
    }
}

/// Builder for constructing ImmutableCsr via batch operations
pub struct ImmutableCsrBuilder {
    vertex_capacity: usize,
    vertex_edges: Vec<Vec<Nbr>>,
}

impl ImmutableCsrBuilder {
    pub fn new(vertex_capacity: usize) -> Self {
        Self {
            vertex_capacity: vertex_capacity.max(1),
            vertex_edges: vec![Vec::new(); vertex_capacity.max(1)],
        }
    }

    /// Add an edge during the build phase
    ///
    /// Caller must ensure edges are added in order (no deduplication).
    pub fn batch_put_edge(
        &mut self,
        src_vid: u32,
        dst: VertexId,
        edge_id: EdgeId,
        prop_offset: u32,
    ) -> bool {
        let src_idx = src_vid as usize;
        if src_idx >= self.vertex_capacity {
            return false;
        }
        self.vertex_edges[src_idx].push(Nbr::new(dst, edge_id, prop_offset, 0));
        true
    }

    /// Build the final immutable CSR
    pub fn build(self) -> ImmutableCsr {
        let mut nbr_list = Vec::new();
        let mut adj_offsets = Vec::with_capacity(self.vertex_capacity);
        let mut degrees = Vec::with_capacity(self.vertex_capacity);

        for edges in &self.vertex_edges {
            adj_offsets.push(nbr_list.len() as u32);
            degrees.push(edges.len() as u32);
            nbr_list.extend_from_slice(edges);
        }

        ImmutableCsr {
            nbr_list: nbr_list.into_boxed_slice(),
            adj_offsets: adj_offsets.into_boxed_slice(),
            degrees: degrees.into_boxed_slice(),
            vertex_capacity: self.vertex_capacity,
        }
    }
}

impl CsrBase for ImmutableCsr {
    fn vertex_capacity(&self) -> usize {
        self.vertex_capacity
    }

    fn edge_count(&self) -> u64 {
        self.nbr_list.len() as u64
    }

    fn dump(&self) -> Vec<u8> {
        ImmutableCsr::dump(self)
    }

    fn load(&mut self, data: &[u8]) -> StorageResult<()> {
        ImmutableCsr::load(self, data)
    }
}

impl MutableCsrTrait for ImmutableCsr {
    fn insert_edge(
        &mut self,
        _src_vid: u32,
        _dst: VertexId,
        _edge_id: EdgeId,
        _prop_offset: u32,
        _ts: Timestamp,
    ) -> bool {
        // Immutable: all writes rejected
        false
    }

    fn delete_edge(&mut self, _src_vid: u32, _edge_id: EdgeId, _ts: Timestamp) -> bool {
        // Immutable: all writes rejected
        false
    }

    fn delete_edge_by_dst(&mut self, _src_vid: u32, _dst: VertexId, _ts: Timestamp) -> bool {
        // Immutable: all writes rejected
        false
    }

    fn delete_edge_by_offset(&mut self, _src_vid: u32, _offset: i32, _ts: Timestamp) -> bool {
        // Immutable: all writes rejected
        false
    }

    fn revert_delete_by_offset(&mut self, _src_vid: u32, _offset: i32, _ts: Timestamp) -> bool {
        // Immutable: all writes rejected
        false
    }

    fn get_edge(&self, src_vid: u32, dst: VertexId, ts: Timestamp) -> Option<Nbr> {
        ImmutableCsr::get_edge(self, src_vid, dst, ts)
    }

    fn edges_of(&self, src_vid: u32, _ts: Timestamp) -> Vec<Nbr> {
        ImmutableCsr::edges_of(self, src_vid)
    }

    fn compact_with_ts(&mut self, _ts: Timestamp, _reserve_ratio: f32) -> usize {
        // Already compact, no-op
        0
    }

    fn used_memory_size(&self) -> usize {
        ImmutableCsr::used_memory_size(self)
    }
}

fn write_vertex_id(out: &mut Vec<u8>, id: VertexId) {
    let bytes = id.as_bytes();
    out.push(bytes.len() as u8);
    out.extend_from_slice(bytes);
}

fn read_vertex_id(data: &[u8], offset: &mut usize) -> StorageResult<VertexId> {
    if *offset >= data.len() {
        return Err(StorageError::deserialize_error(
            "ImmutableCsr data too short for vertex id length",
        ));
    }

    let len = data[*offset] as usize;
    *offset += 1;
    if data.len().saturating_sub(*offset) < len {
        return Err(StorageError::deserialize_error(
            "ImmutableCsr data too short for vertex id bytes",
        ));
    }

    let id = VertexId::from_bytes(data[*offset..*offset + len].to_vec());
    *offset += len;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_immutable_csr_builder() {
        let mut builder = ImmutableCsr::builder(5);

        builder.batch_put_edge(0, VertexId::from_int64(1), EdgeId(100), 0);
        builder.batch_put_edge(0, VertexId::from_int64(2), EdgeId(101), 4);
        builder.batch_put_edge(1, VertexId::from_int64(0), EdgeId(102), 8);

        let csr = builder.build();

        assert_eq!(csr.vertex_capacity(), 5);
        assert_eq!(csr.edge_count(), 3);

        let edges_0 = csr.edges_of_ref(0);
        assert_eq!(edges_0.len(), 2);
        assert_eq!(edges_0[0].neighbor, VertexId::from_int64(1));
        assert_eq!(edges_0[1].neighbor, VertexId::from_int64(2));

        let edges_1 = csr.edges_of_ref(1);
        assert_eq!(edges_1.len(), 1);
        assert_eq!(edges_1[0].neighbor, VertexId::from_int64(0));

        let edges_2 = csr.edges_of_ref(2);
        assert_eq!(edges_2.len(), 0);
    }

    #[test]
    fn test_immutable_csr_get_edge() {
        let mut builder = ImmutableCsr::builder(3);
        builder.batch_put_edge(0, VertexId::from_int64(1), EdgeId(100), 0);
        builder.batch_put_edge(0, VertexId::from_int64(2), EdgeId(101), 4);

        let csr = builder.build();

        let edge = csr.get_edge_unchecked(0, VertexId::from_int64(1));
        assert!(edge.is_some());
        assert_eq!(edge.unwrap().edge_id, EdgeId(100));

        let edge = csr.get_edge_unchecked(0, VertexId::from_int64(3));
        assert!(edge.is_none());

        let edge = csr.get_edge_unchecked(5, VertexId::from_int64(1));
        assert!(edge.is_none());
    }

    #[test]
    fn test_immutable_csr_dump_load() {
        let mut builder = ImmutableCsr::builder(3);
        builder.batch_put_edge(0, VertexId::from_int64(1), EdgeId(100), 0);
        builder.batch_put_edge(1, VertexId::from_int64(2), EdgeId(101), 4);
        builder.batch_put_edge(2, VertexId::from_int64(0), EdgeId(102), 8);

        let csr1 = builder.build();
        let data = csr1.dump();

        let mut csr2 = ImmutableCsr::builder(3).build();
        csr2.load(&data).unwrap();

        assert_eq!(csr2.vertex_capacity(), 3);
        assert_eq!(csr2.edge_count(), 3);

        let edges_0 = csr2.edges_of_ref(0);
        assert_eq!(edges_0.len(), 1);
        assert_eq!(edges_0[0].neighbor, VertexId::from_int64(1));
    }

    #[test]
    fn test_immutable_csr_reject_writes() {
        let csr = ImmutableCsr::builder(3).build();
        let mut csr_mutable: Box<dyn MutableCsrTrait> = Box::new(csr);

        // All write operations should be rejected
        assert!(!csr_mutable.insert_edge(0, VertexId::from_int64(1), EdgeId(100), 0, 1));
        assert!(!csr_mutable.delete_edge(0, EdgeId(100), 1));
        assert!(!csr_mutable.delete_edge_by_dst(0, VertexId::from_int64(1), 1));
        assert!(!csr_mutable.delete_edge_by_offset(0, 0, 1));
        assert!(!csr_mutable.revert_delete_by_offset(0, 0, 1));
    }

    #[test]
    fn test_immutable_csr_memory_size() {
        let mut builder = ImmutableCsr::builder(100);
        for src in 0..100 {
            builder.batch_put_edge(src, VertexId::from_int64(1), EdgeId(src as u64), 0);
        }

        let csr = builder.build();
        let size = csr.used_memory_size();

        // Should include: edges (100 * Nbr) + offsets (100 * u32) + degrees (100 * u32)
        let expected = 100 * std::mem::size_of::<Nbr>()
                     + 100 * std::mem::size_of::<u32>()
                     + 100 * std::mem::size_of::<u32>()
                     + std::mem::size_of::<ImmutableCsr>();
        assert!(size >= expected);
    }

    #[test]
    fn test_immutable_csr_from_snapshot() {
        let mut mutable = MutableCsr::with_capacity(3, 10);

        mutable.insert_edge(0, VertexId::from_int64(1), EdgeId(100), 0, 10);
        mutable.insert_edge(0, VertexId::from_int64(2), EdgeId(101), 4, 20);
        mutable.insert_edge(1, VertexId::from_int64(0), EdgeId(102), 8, 15);
        mutable.insert_edge(2, VertexId::from_int64(3), EdgeId(103), 12, 25);

        // Take snapshot at ts=20, should include edges with timestamp <= 20
        let immutable = ImmutableCsr::from_snapshot(&mutable, 20);

        assert_eq!(immutable.vertex_capacity(), 3);
        assert_eq!(immutable.edge_count(), 3); // edges at ts: 10, 20, 15

        let edges_0 = immutable.edges_of(0);
        assert_eq!(edges_0.len(), 2);

        let edges_1 = immutable.edges_of(1);
        assert_eq!(edges_1.len(), 1);

        let edges_2 = immutable.edges_of(2);
        assert_eq!(edges_2.len(), 0); // edge at ts=25 is after cutoff
    }

    #[test]
    fn test_immutable_csr_trait_implementation() {
        let csr = ImmutableCsr::builder(3).build();

        // Should implement CsrBase
        assert_eq!(csr.vertex_capacity(), 3);
        assert_eq!(csr.edge_count(), 0);

        let data = csr.dump();
        assert!(!data.is_empty());
    }

    #[test]
    fn test_immutable_csr_mutable_csr_trait() {
        let csr = ImmutableCsr::builder(2).build();

        // Test via MutableCsrTrait methods
        let csr_trait: &dyn MutableCsrTrait = &csr;
        let edges = csr_trait.edges_of(0, 999);
        assert_eq!(edges.len(), 0);

        // get_edge with timestamp should be supported
        let edge = csr_trait.get_edge(0, VertexId::from_int64(1), 999);
        assert!(edge.is_none());

        // compact_with_ts should be no-op
        let mut csr_mut = csr;
        assert_eq!(csr_mut.compact_with_ts(100, 0.25), 0);
    }
}
