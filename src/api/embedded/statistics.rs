//! Session-level change statistics module
//!
//! Provides statistics such as the number of rows affected by query execution, last insertion ID, etc.

use std::sync::atomic::{AtomicU64, Ordering};

/// Session-level change statistics
///
/// Records the execution of queries in the session, including:
/// - Number of rows affected by the last operation
/// - Total session changes
/// - Vertex ID of the last inserted vertex
/// - Last inserted edge ID
#[derive(Debug)]
pub struct SessionStatistics {
    /// Number of rows affected by the last operation
    last_changes: AtomicU64,
    /// Total number of session changes
    total_changes: AtomicU64,
    /// ID of the last vertex inserted
    last_insert_vertex_id: AtomicU64,
    /// Last inserted edge ID
    last_insert_edge_id: AtomicU64,
    /// Whether there is a vertex ID (0 means invalid)
    has_vertex_id: AtomicU64,
    /// Whether there is an edge ID (0 means invalid)
    has_edge_id: AtomicU64,
}

impl SessionStatistics {
    /// Creating a new statistics instance
    pub fn new() -> Self {
        Self {
            last_changes: AtomicU64::new(0),
            total_changes: AtomicU64::new(0),
            last_insert_vertex_id: AtomicU64::new(0),
            last_insert_edge_id: AtomicU64::new(0),
            has_vertex_id: AtomicU64::new(0),
            has_edge_id: AtomicU64::new(0),
        }
    }

    /// Record the number of lines changed
    ///
    /// # Parameters
    /// - `count` - number of lines affected
    pub fn record_changes(&self, count: u64) {
        self.last_changes.store(count, Ordering::SeqCst);
        self.total_changes.fetch_add(count, Ordering::SeqCst);
    }

    /// Record Vertex Insertion
    ///
    /// # Parameters
    /// - `id` - the ID of the inserted vertex
    pub fn record_vertex_insert(&self, id: i64) {
        if id > 0 {
            self.last_insert_vertex_id
                .store(id as u64, Ordering::SeqCst);
            self.has_vertex_id.store(1, Ordering::SeqCst);
            self.last_changes.store(1, Ordering::SeqCst);
            self.total_changes.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// Record-side insertion
    ///
    /// # Parameters
    /// - `id` - the ID of the inserted side
    pub fn record_edge_insert(&self, id: i64) {
        if id > 0 {
            self.last_insert_edge_id.store(id as u64, Ordering::SeqCst);
            self.has_edge_id.store(1, Ordering::SeqCst);
            self.last_changes.store(1, Ordering::SeqCst);
            self.total_changes.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// Get the number of rows affected by the last operation
    pub fn last_changes(&self) -> u64 {
        self.last_changes.load(Ordering::SeqCst)
    }

    /// Getting the number of total session changes
    pub fn total_changes(&self) -> u64 {
        self.total_changes.load(Ordering::SeqCst)
    }

    /// Get the ID of the last inserted vertex
    ///
    /// Returns None for no records
    pub fn last_insert_vertex_id(&self) -> Option<i64> {
        if self.has_vertex_id.load(Ordering::SeqCst) != 0 {
            Some(self.last_insert_vertex_id.load(Ordering::SeqCst) as i64)
        } else {
            None
        }
    }

    /// Get the last inserted edge ID
    ///
    /// Returning None indicates that no records were found.
    pub fn last_insert_edge_id(&self) -> Option<i64> {
        if self.has_edge_id.load(Ordering::SeqCst) != 0 {
            Some(self.last_insert_edge_id.load(Ordering::SeqCst) as i64)
        } else {
            None
        }
    }

    /// Reset last change record
    ///
    /// Usually called before executing a new query
    pub fn reset_last(&self) {
        self.last_changes.store(0, Ordering::SeqCst);
        self.has_vertex_id.store(0, Ordering::SeqCst);
        self.has_edge_id.store(0, Ordering::SeqCst);
    }

    /// Reset all statistics
    pub fn reset_all(&self) {
        self.last_changes.store(0, Ordering::SeqCst);
        self.total_changes.store(0, Ordering::SeqCst);
        self.last_insert_vertex_id.store(0, Ordering::SeqCst);
        self.last_insert_edge_id.store(0, Ordering::SeqCst);
        self.has_vertex_id.store(0, Ordering::SeqCst);
        self.has_edge_id.store(0, Ordering::SeqCst);
    }
}

impl Default for SessionStatistics {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for SessionStatistics {
    fn clone(&self) -> Self {
        Self {
            last_changes: AtomicU64::new(self.last_changes.load(Ordering::SeqCst)),
            total_changes: AtomicU64::new(self.total_changes.load(Ordering::SeqCst)),
            last_insert_vertex_id: AtomicU64::new(
                self.last_insert_vertex_id.load(Ordering::SeqCst),
            ),
            last_insert_edge_id: AtomicU64::new(self.last_insert_edge_id.load(Ordering::SeqCst)),
            has_vertex_id: AtomicU64::new(self.has_vertex_id.load(Ordering::SeqCst)),
            has_edge_id: AtomicU64::new(self.has_edge_id.load(Ordering::SeqCst)),
        }
    }
}

/// Statistical information on query results
///
/// Statistics extracted from query results
#[derive(Debug, Clone, Default)]
pub struct QueryStatistics {
    /// Number of rows affected
    pub rows_affected: u64,
    /// Number of rows returned
    pub rows_returned: u64,
    /// List of inserted vertex IDs
    pub inserted_vertex_ids: Vec<i64>,
    /// List of inserted edge IDs
    pub inserted_edge_ids: Vec<i64>,
    /// Number of vertices updated
    pub vertices_updated: u64,
    /// Updated number of edges
    pub edges_updated: u64,
    /// Number of vertices deleted
    pub vertices_deleted: u64,
    /// Number of edges deleted
    pub edges_deleted: u64,
}

impl QueryStatistics {
    /// Creating empty statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Created from query result metadata
    ///
    /// # Parameters
    /// - `metadata` - query result metadata
    pub fn from_metadata(metadata: &crate::api::core::ExecutionMetadata) -> Self {
        Self {
            rows_affected: metadata.rows_returned,
            rows_returned: metadata.rows_returned,
            ..Default::default()
        }
    }

    /// Merging another statistic
    pub fn merge(&mut self, other: &QueryStatistics) {
        self.rows_affected += other.rows_affected;
        self.rows_returned += other.rows_returned;
        self.inserted_vertex_ids
            .extend_from_slice(&other.inserted_vertex_ids);
        self.inserted_edge_ids
            .extend_from_slice(&other.inserted_edge_ids);
        self.vertices_updated += other.vertices_updated;
        self.edges_updated += other.edges_updated;
        self.vertices_deleted += other.vertices_deleted;
        self.edges_deleted += other.edges_deleted;
    }

    /// Getting the total number of changes
    pub fn total_changes(&self) -> u64 {
        self.rows_affected
            + self.vertices_updated
            + self.edges_updated
            + self.vertices_deleted
            + self.edges_deleted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_statistics_basic() {
        let stats = SessionStatistics::new();

        assert_eq!(stats.last_changes(), 0);
        assert_eq!(stats.total_changes(), 0);
        assert_eq!(stats.last_insert_vertex_id(), None);
        assert_eq!(stats.last_insert_edge_id(), None);
    }

    #[test]
    fn test_record_changes() {
        let stats = SessionStatistics::new();

        stats.record_changes(5);
        assert_eq!(stats.last_changes(), 5);
        assert_eq!(stats.total_changes(), 5);

        stats.record_changes(3);
        assert_eq!(stats.last_changes(), 3);
        assert_eq!(stats.total_changes(), 8);
    }

    #[test]
    fn test_record_vertex_insert() {
        let stats = SessionStatistics::new();

        stats.record_vertex_insert(100);
        assert_eq!(stats.last_insert_vertex_id(), Some(100));
        assert_eq!(stats.last_changes(), 1);
        assert_eq!(stats.total_changes(), 1);

        // Invalid IDs should not be logged
        stats.record_vertex_insert(0);
        assert_eq!(stats.last_insert_vertex_id(), Some(100)); // Of course! Please provide the text you would like to have translated.
    }

    #[test]
    fn test_record_edge_insert() {
        let stats = SessionStatistics::new();

        stats.record_edge_insert(200);
        assert_eq!(stats.last_insert_edge_id(), Some(200));
        assert_eq!(stats.last_changes(), 1);
        assert_eq!(stats.total_changes(), 1);
    }

    #[test]
    fn test_reset() {
        let stats = SessionStatistics::new();

        stats.record_changes(5);
        stats.record_vertex_insert(100);

        stats.reset_last();
        assert_eq!(stats.last_changes(), 0);
        assert_eq!(stats.last_insert_vertex_id(), None);
        assert_eq!(stats.total_changes(), 6); // The total amount remains unchanged.

        stats.reset_all();
        assert_eq!(stats.total_changes(), 0);
    }

    #[test]
    fn test_query_statistics() {
        let mut stats = QueryStatistics::new();
        stats.rows_affected = 10;
        stats.rows_returned = 5;
        stats.inserted_vertex_ids = vec![1, 2, 3];

        assert_eq!(stats.total_changes(), 10);

        let mut other = QueryStatistics::new();
        other.rows_affected = 5;
        other.inserted_vertex_ids = vec![4, 5];

        stats.merge(&other);
        assert_eq!(stats.rows_affected, 15);
        assert_eq!(stats.inserted_vertex_ids.len(), 5);
    }
}
