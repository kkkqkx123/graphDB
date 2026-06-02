//! Index Type Classification
//!
//! This module defines the classification of index types in the graph database.
//! This module defines the common types for property indexes in the graph database.
//!
//! Secondary indexes are built on property values and support complex queries.
//! They are decoupled from the CSR structure and use BTreeMap for storage.
//!
//! - `VertexIndexManager`: Index on vertex properties
//! - `EdgeIndexManager`: Index on edge properties

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IndexEstimate {
    pub total_entries: usize,
    pub visible_entries: usize,
    pub tombstone_entries: usize,
}

impl IndexEstimate {
    pub fn new(total_entries: usize, visible_entries: usize, tombstone_entries: usize) -> Self {
        Self {
            total_entries,
            visible_entries,
            tombstone_entries,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.total_entries == 0
    }
}
