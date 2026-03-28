//! Batch Operation Module
//!
//! Supports efficient high-volume data import

use crate::api::core::{CoreError, CoreResult};
use crate::api::embedded::session::Session;
use crate::core::{Edge, Vertex};
use crate::storage::StorageClient;

/// Batch Inserter
///
/// For efficient batch insertion of vertex and edge data
///
/// # Examples
///
/// ```rust
/// use graphdb::api::embedded::{GraphDatabase, DatabaseConfig};
/// use graphdb::core::{Vertex, Edge, Value};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let db = GraphDatabase::open("my_db")?;
/// let session = db.session()?;
///
// Create a batch inserter that automatically refreshes every 100 entries
/// let mut inserter = session.batch_inserter(100);
///
// Add vertices
/// for i in 0..1000 {
///     let vertex = Vertex::with_vid(Value::Int(i));
///     inserter.add_vertex(vertex);
/// }
///
// Perform batch insertion
/// let result = inserter.execute()?;
/// println!("Inserted {} vertices", result.vertices_inserted);
/// # Ok(())
/// # }
/// ```
pub struct BatchInserter<'sess, S: StorageClient + Clone + 'static> {
    session: &'sess Session<S>,
    batch_size: usize,
    vertex_buffer: Vec<Vertex>,
    edge_buffer: Vec<Edge>,
    total_inserted: BatchResult,
}

/// Batch operation results
#[derive(Debug, Clone, Default)]
pub struct BatchResult {
    /// Number of vertices inserted
    pub vertices_inserted: usize,
    /// Number of inserted edges
    pub edges_inserted: usize,
    /// error message
    pub errors: Vec<BatchError>,
}

/// batch error
#[derive(Debug, Clone)]
pub struct BatchError {
    /// Index where the error occurred
    pub index: usize,
    /// Error item type
    pub item_type: BatchItemType,
    /// error message
    pub error: String,
}

/// Batch item type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchItemType {
    /// vertice
    Vertex,
    /// suffix of a noun of locality
    Edge,
}

impl<'sess, S: StorageClient + Clone + 'static> BatchInserter<'sess, S> {
    /// Creating a new batch inserter
    pub(crate) fn new(session: &'sess Session<S>, batch_size: usize) -> Self {
        Self {
            session,
            batch_size: batch_size.max(1), // Ensure at least 1
            vertex_buffer: Vec::with_capacity(batch_size),
            edge_buffer: Vec::with_capacity(batch_size),
            total_inserted: BatchResult {
                vertices_inserted: 0,
                edges_inserted: 0,
                errors: Vec::new(),
            },
        }
    }

    /// Adding Vertices
    ///
    /// # Parameters
    /// - `vertex` - the vertex to be inserted
    ///
    /// # Back
    /// - Return to itself, supporting chain calls
    pub fn add_vertex(&mut self, vertex: Vertex) -> &mut Self {
        self.vertex_buffer.push(vertex);

        // Automatically refreshes if batch size is reached
        if self.vertex_buffer.len() >= self.batch_size {
            let _ = self.flush_vertices();
        }

        self
    }

    /// Add Edge
    ///
    /// # Parameters
    /// - `edge` - the edge to be inserted
    ///
    /// # Back
    /// Return itself, supporting chained calls.
    pub fn add_edge(&mut self, edge: Edge) -> &mut Self {
        self.edge_buffer.push(edge);

        // Automatically refreshes if batch size is reached
        if self.edge_buffer.len() >= self.batch_size {
            let _ = self.flush_edges();
        }

        self
    }

    /// Adding multiple vertices
    ///
    /// # Parameters
    /// - `vertices` - a list of vertices to be inserted
    pub fn add_vertices(&mut self, vertices: Vec<Vertex>) -> &mut Self {
        for vertex in vertices {
            self.add_vertex(vertex);
        }
        self
    }

    /// Adding multiple edges
    ///
    /// # Parameters
    /// - `edges` - a list of edges to be inserted
    pub fn add_edges(&mut self, edges: Vec<Edge>) -> &mut Self {
        for edge in edges {
            self.add_edge(edge);
        }
        self
    }

    /// Perform batch insertion
    ///
    /// Flush all buffered data and return results
    ///
    /// # Return
    /// - Returns batch operation results on success
    /// - Return error on failure
    pub fn execute(mut self) -> CoreResult<BatchResult> {
        // Refresh the remaining vertices
        self.flush_vertices()?;

        // Refresh remaining edges
        self.flush_edges()?;

        Ok(self.total_inserted)
    }

    /// Flush Vertex Buffer
    fn flush_vertices(&mut self) -> CoreResult<()> {
        if self.vertex_buffer.is_empty() {
            return Ok(());
        }

        // Get current space name
        let space_name = self
            .session
            .space_name()
            .ok_or_else(|| CoreError::InvalidParameter("No graph space selected".to_string()))?;

        // Remove vertices from the buffer
        let vertices_to_insert: Vec<Vertex> = std::mem::take(&mut self.vertex_buffer);
        let count = vertices_to_insert.len();

        // Calling the storage layer's batch insertion interface
        let mut storage = self.session.storage();
        match storage.batch_insert_vertices(space_name, vertices_to_insert) {
            Ok(_) => {
                // Insertion successful; update the count.
                self.total_inserted.vertices_inserted += count;
            }
            Err(e) => {
                // The insertion failed, and an error was recorded; however, the error is not returned immediately.
                // In this way, the caller can obtain the partially successful results as well as all the errors through the BatchResult.
                self.total_inserted.errors.push(BatchError {
                    index: self.total_inserted.vertices_inserted,
                    item_type: BatchItemType::Vertex,
                    error: format!("Batch vertex insertion failed: {}", e),
                });
            }
        }

        Ok(())
    }

    /// Refresh the side buffer.
    fn flush_edges(&mut self) -> CoreResult<()> {
        if self.edge_buffer.is_empty() {
            return Ok(());
        }

        // Obtain the current name of the space.
        let space_name = self
            .session
            .space_name()
            .ok_or_else(|| CoreError::InvalidParameter("No graph space selected".to_string()))?;

        // Extract the edges from the buffer.
        let edges_to_insert: Vec<Edge> = std::mem::take(&mut self.edge_buffer);
        let count = edges_to_insert.len();

        // Call the batch insertion interface of the storage layer.
        let mut storage = self.session.storage();
        match storage.batch_insert_edges(space_name, edges_to_insert) {
            Ok(_) => {
                // Insertion successful; update the count.
                self.total_inserted.edges_inserted += count;
            }
            Err(e) => {
                // The insertion failed, and an error was recorded; however, the error is not returned immediately.
                // In this way, the caller can obtain the partially successful results as well as all the errors through the BatchResult.
                self.total_inserted.errors.push(BatchError {
                    index: self.total_inserted.edges_inserted,
                    item_type: BatchItemType::Edge,
                    error: format!("Batch insertion of edges failed: {}", e),
                });
            }
        }

        Ok(())
    }

    /// Get the number of vertices in the current buffer.
    pub fn buffered_vertices(&self) -> usize {
        self.vertex_buffer.len()
    }

    /// Get the number of edges in the current buffer.
    pub fn buffered_edges(&self) -> usize {
        self.edge_buffer.len()
    }

    /// Obtain the batch size
    pub fn batch_size(&self) -> usize {
        self.batch_size
    }

    /// Check whether there is any buffered data.
    pub fn has_buffered_data(&self) -> bool {
        !self.vertex_buffer.is_empty() || !self.edge_buffer.is_empty()
    }
}

impl BatchResult {
    /// Obtain the total number of insertions.
    pub fn total_inserted(&self) -> usize {
        self.vertices_inserted + self.edges_inserted
    }

    /// Check for any errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get the number of errors
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Merge the results of another batch.
    pub fn merge(&mut self, other: BatchResult) {
        self.vertices_inserted += other.vertices_inserted;
        self.edges_inserted += other.edges_inserted;
        self.errors.extend(other.errors);
    }
}

impl BatchError {
    /// Create a new batch of errors.
    pub fn new(index: usize, item_type: BatchItemType, error: impl Into<String>) -> Self {
        Self {
            index,
            item_type,
            error: error.into(),
        }
    }
}

/// Batch operation configuration
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Batch size
    pub batch_size: usize,
    /// Should the submission be done automatically?
    pub auto_commit: bool,
    /// Should I ignore the errors and continue with the process?
    pub continue_on_error: bool,
    /// Maximum number of errors
    pub max_errors: Option<usize>,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            auto_commit: true,
            continue_on_error: true,
            max_errors: Some(100),
        }
    }
}

impl BatchConfig {
    /// Create a new configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the batch size
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size.max(1);
        self
    }

    /// Set whether to submit automatically.
    pub fn with_auto_commit(mut self, auto_commit: bool) -> Self {
        self.auto_commit = auto_commit;
        self
    }

    /// Set whether to continue processing the errors.
    pub fn with_continue_on_error(mut self, continue_on_error: bool) -> Self {
        self.continue_on_error = continue_on_error;
        self
    }

    /// Set the maximum number of allowed errors
    pub fn with_max_errors(mut self, max_errors: Option<usize>) -> Self {
        self.max_errors = max_errors;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_result_default() {
        let result = BatchResult::default();
        assert_eq!(result.vertices_inserted, 0);
        assert_eq!(result.edges_inserted, 0);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_batch_result_total_inserted() {
        let result = BatchResult {
            vertices_inserted: 100,
            edges_inserted: 50,
            errors: Vec::new(),
        };
        assert_eq!(result.total_inserted(), 150);
    }

    #[test]
    fn test_batch_result_merge() {
        let mut result1 = BatchResult {
            vertices_inserted: 100,
            edges_inserted: 50,
            errors: vec![BatchError::new(0, BatchItemType::Vertex, "error1")],
        };

        let result2 = BatchResult {
            vertices_inserted: 200,
            edges_inserted: 100,
            errors: vec![BatchError::new(1, BatchItemType::Edge, "error2")],
        };

        result1.merge(result2);

        assert_eq!(result1.vertices_inserted, 300);
        assert_eq!(result1.edges_inserted, 150);
        assert_eq!(result1.errors.len(), 2);
    }

    #[test]
    fn test_batch_config_default() {
        let config = BatchConfig::default();
        assert_eq!(config.batch_size, 1000);
        assert!(config.auto_commit);
        assert!(config.continue_on_error);
        assert_eq!(config.max_errors, Some(100));
    }

    #[test]
    fn test_batch_config_builder() {
        let config = BatchConfig::new()
            .with_batch_size(500)
            .with_auto_commit(false)
            .with_continue_on_error(false)
            .with_max_errors(Some(50));

        assert_eq!(config.batch_size, 500);
        assert!(!config.auto_commit);
        assert!(!config.continue_on_error);
        assert_eq!(config.max_errors, Some(50));
    }

    #[test]
    fn test_batch_config_min_batch_size() {
        let config = BatchConfig::new().with_batch_size(0);
        assert_eq!(config.batch_size, 1); // The minimum value is 1.
    }
}
