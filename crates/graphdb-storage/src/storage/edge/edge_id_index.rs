//! Edge ID to location index for O(1) edge lookups.
//!
//! Maintains a fast lookup table from edge_id to its location in the CSR structure.
//! This enables O(1) edge deletion instead of O(degree) scanning.
//!
//! # Performance Trade-off
//!
//! - **Lookup**: O(1) with hash table overhead
//! - **Insert**: O(1) amortized
//! - **Delete**: O(1)
//! - **Memory**: ~16 bytes per edge (key + location info)
//!
//! # Use Cases
//!
//! - High-frequency edge deletion workloads
//! - Graph modification with timestamp-based MVCC
//! - Efficient edge attribute updates

use std::collections::HashMap;

use super::EdgeId;

/// Location information for an edge in CSR storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgeLocation {
    /// Source vertex index
    pub src_idx: u32,
    /// Offset within the vertex's edge list (primary or overflow)
    /// - If < primary_capacity: offset in primary block
    /// - If >= primary_capacity: overflow block index
    pub edge_offset: u32,
}

/// Fast lookup index for edge_id → (src_idx, edge_offset).
///
/// Optional optimization for workloads with frequent edge deletions.
#[derive(Debug, Clone)]
pub struct EdgeIdIndex {
    /// Mapping from edge_id to (src_idx, edge_offset)
    index: HashMap<EdgeId, EdgeLocation>,
}

impl EdgeIdIndex {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            index: HashMap::with_capacity(capacity),
        }
    }

    /// Insert an edge into the index
    pub fn insert(&mut self, edge_id: EdgeId, location: EdgeLocation) {
        self.index.insert(edge_id, location);
    }

    /// Get the location of an edge
    pub fn get(&self, edge_id: EdgeId) -> Option<EdgeLocation> {
        self.index.get(&edge_id).copied()
    }

    /// Remove an edge from the index
    pub fn remove(&mut self, edge_id: EdgeId) -> Option<EdgeLocation> {
        self.index.remove(&edge_id)
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.index.clear();
    }

    /// Number of indexed edges
    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// Update location for an edge (used during compaction)
    pub fn update_location(&mut self, edge_id: EdgeId, new_location: EdgeLocation) {
        self.index.insert(edge_id, new_location);
    }

    /// Memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        self.index.capacity() * (std::mem::size_of::<EdgeId>() + std::mem::size_of::<EdgeLocation>())
    }
}

impl Default for EdgeIdIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_id_index_operations() {
        let mut index = EdgeIdIndex::new();
        let edge_id = EdgeId(42);
        let location = EdgeLocation {
            src_idx: 1,
            edge_offset: 5,
        };

        // Insert
        index.insert(edge_id, location);
        assert_eq!(index.len(), 1);

        // Get
        assert_eq!(index.get(edge_id), Some(location));

        // Update
        let new_location = EdgeLocation {
            src_idx: 2,
            edge_offset: 10,
        };
        index.update_location(edge_id, new_location);
        assert_eq!(index.get(edge_id), Some(new_location));

        // Remove
        assert_eq!(index.remove(edge_id), Some(new_location));
        assert_eq!(index.get(edge_id), None);
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_edge_id_index_multiple_edges() {
        let mut index = EdgeIdIndex::with_capacity(10);

        for i in 0..5 {
            let edge_id = EdgeId(i);
            let location = EdgeLocation {
                src_idx: (i / 2) as u32,
                edge_offset: i as u32,
            };
            index.insert(edge_id, location);
        }

        assert_eq!(index.len(), 5);

        for i in 0..5 {
            let edge_id = EdgeId(i);
            assert!(index.get(edge_id).is_some());
        }
    }
}
