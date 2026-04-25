//! Batch operation
//!
//! Provide efficient batch document adding, updating, deleting operations
//!
//! # Example of use
//!
//! ```rust
//! use inversearch_service::document::Batch;
//! use serde_json::json;
//!
//! let mut batch = Batch::new(1000); // 批量大小 1000
//!
//! // Add operation
//! let doc1 = json!({"title": "Doc 1"});
//! let doc2 = json!({"title": "Doc 2"});
//! batch.add(1, &doc1);
//! batch.add(2, &doc2);
//!
//! // Perform batch operations
//! // index.execute_batch(&mut batch)?;
//! ```

use crate::DocId;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Global Batch Operation ID Counter
static BATCH_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

/// Generate unique batch operation IDs
fn generate_batch_id() -> usize {
    BATCH_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Batch operation type
#[derive(Debug, Clone)]
pub enum BatchOperation<'a> {
    Add(DocId, &'a Value),
    Update(DocId, &'a Value),
    Remove(DocId),
    Replace(DocId, &'a Value),
}

/// Batch operation status
#[derive(Debug, Clone, PartialEq)]
pub enum BatchStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
    RolledBack,
}

/// Batch manipulation of metadata
#[derive(Debug, Clone)]
pub struct BatchMetadata {
    pub batch_id: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub status: BatchStatus,
    pub total_operations: usize,
    pub completed_operations: usize,
    pub failed_operations: usize,
}

/// batch operation buffer
#[derive(Debug, Clone)]
pub struct Batch<'a> {
    operations: Vec<BatchOperation<'a>>,
    max_size: usize,
    atomic: bool,
    transactional: bool,
    metadata: Option<BatchMetadata>,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> Batch<'a> {
    /// Creating a new batch operation
    pub fn new(max_size: usize) -> Self {
        Batch {
            operations: Vec::with_capacity(max_size),
            max_size,
            atomic: false,
            transactional: false,
            metadata: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Creating Atomic Batch Operations
    pub fn new_atomic(max_size: usize) -> Self {
        let mut batch = Self::new(max_size);
        batch.atomic = true;
        batch
    }

    /// Creating Transactional Batch Operations
    pub fn new_transactional(max_size: usize) -> Self {
        let mut batch = Self::new(max_size);
        batch.transactional = true;
        batch
    }

    /// Adding Documents
    pub fn add(&mut self, id: DocId, content: &'a Value) {
        self.operations.push(BatchOperation::Add(id, content));
    }

    /// Update Documentation
    pub fn update(&mut self, id: DocId, content: &'a Value) {
        self.operations.push(BatchOperation::Update(id, content));
    }

    /// Delete Document
    pub fn remove(&mut self, id: DocId) {
        self.operations.push(BatchOperation::Remove(id));
    }

    /// Replacement Document
    pub fn replace(&mut self, id: DocId, content: &'a Value) {
        self.operations.push(BatchOperation::Replace(id, content));
    }

    /// Check if a refresh is needed
    pub fn should_flush(&self) -> bool {
        self.operations.len() >= self.max_size
    }

    /// Get the number of operations
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// Check if it is empty
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// Empty the operation queue.
    pub fn clear(&mut self) {
        self.operations.clear();
    }

    /// Remove all operations
    pub fn drain(&mut self) -> Vec<BatchOperation<'a>> {
        self.operations.drain(..).collect()
    }

    /// Get Operation References
    pub fn operations(&self) -> &[BatchOperation<'a>] {
        &self.operations
    }

    /// Setting up batch metadata
    pub fn set_metadata(&mut self, metadata: BatchMetadata) {
        self.metadata = Some(metadata);
    }

    /// Getting bulk metadata
    pub fn metadata(&self) -> Option<&BatchMetadata> {
        self.metadata.as_ref()
    }

    /// Check for atomicity
    pub fn is_atomic(&self) -> bool {
        self.atomic
    }

    /// Check if it is transactional
    pub fn is_transactional(&self) -> bool {
        self.transactional
    }
}

/// Batch operation actuator (generalized version)
pub struct BatchExecutorFn<'a, A, U, R>
where
    A: FnMut(DocId, &Value) -> Result<(), crate::error::InversearchError>,
    U: FnMut(DocId, &Value) -> Result<(), crate::error::InversearchError>,
    R: FnMut(DocId, &Value) -> Result<(), crate::error::InversearchError>,
{
    add_fn: A,
    update_fn: U,
    remove_fn: R,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a, A, U, R> BatchExecutorFn<'a, A, U, R>
where
    A: FnMut(DocId, &Value) -> Result<(), crate::error::InversearchError>,
    U: FnMut(DocId, &Value) -> Result<(), crate::error::InversearchError>,
    R: FnMut(DocId, &Value) -> Result<(), crate::error::InversearchError>,
{
    /// Creating a new actuator
    pub fn new(add_fn: A, update_fn: U, remove_fn: R) -> Self {
        BatchExecutorFn {
            add_fn,
            update_fn,
            remove_fn,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Perform a single operation
    pub fn execute(&mut self, op: &BatchOperation) -> Result<(), crate::error::InversearchError> {
        match op {
            BatchOperation::Add(id, content) => (self.add_fn)(*id, content),
            BatchOperation::Update(id, content) => (self.update_fn)(*id, content),
            BatchOperation::Remove(id) => (self.remove_fn)(*id, &json!({})),
            BatchOperation::Replace(_, _) => Ok(()),
        }
    }

    /// Perform batch operations
    pub fn execute_batch(&mut self, batch: &Batch) -> Result<(), crate::error::InversearchError> {
        for op in &batch.operations {
            self.execute(op)?;
        }
        Ok(())
    }

    /// Execute and clear
    pub fn execute_and_clear(
        &mut self,
        batch: &mut Batch,
    ) -> Result<(), crate::error::InversearchError> {
        self.execute_batch(batch)?;
        batch.clear();
        Ok(())
    }
}

/// Batch operation results
#[derive(Debug, Clone)]
pub struct BatchResult {
    pub batch_id: usize,
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub errors: Vec<(DocId, String)>,
    pub duration_ms: u64,
}

/// Batch Operated Actuators
///
/// Used to perform batch document operations, supporting parallel processing and customizing the number of worker threads
pub struct BatchExecutor {
    parallel: bool,
    max_workers: usize,
}

impl BatchExecutor {
    /// Creating a new batch operation executor
    pub fn new(_batch_size: usize) -> Self {
        BatchExecutor {
            parallel: false,
            max_workers: 4,
        }
    }

    /// Enable parallel processing
    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    /// Setting the maximum number of worker threads
    pub fn with_max_workers(mut self, max_workers: usize) -> Self {
        self.max_workers = max_workers;
        self
    }

    /// Perform a batch add operation
    pub fn execute_batch_add(
        &self,
        operations: &[(DocId, &Value)],
        document: &mut super::Document,
    ) -> BatchResult {
        let start = std::time::Instant::now();
        let mut successful = 0;
        let mut failed = 0;
        let mut errors = Vec::new();

        for &(id, content) in operations {
            match document.add(id, content) {
                Ok(_) => successful += 1,
                Err(e) => {
                    failed += 1;
                    errors.push((id, e.to_string()));
                }
            }
        }

        BatchResult {
            batch_id: generate_batch_id(),
            total_operations: operations.len(),
            successful_operations: successful,
            failed_operations: failed,
            errors,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Perform batch update operations
    pub fn execute_batch_update(
        &self,
        operations: &[(DocId, &Value)],
        document: &mut super::Document,
    ) -> BatchResult {
        let start = std::time::Instant::now();
        let mut successful = 0;
        let mut failed = 0;
        let mut errors = Vec::new();

        for &(id, content) in operations {
            match document.update(id, content) {
                Ok(_) => successful += 1,
                Err(e) => {
                    failed += 1;
                    errors.push((id, e.to_string()));
                }
            }
        }

        BatchResult {
            batch_id: generate_batch_id(),
            total_operations: operations.len(),
            successful_operations: successful,
            failed_operations: failed,
            errors,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Performing batch deletion operations
    pub fn execute_batch_remove(
        &self,
        operations: &[DocId],
        document: &mut super::Document,
    ) -> BatchResult {
        let start = std::time::Instant::now();
        let mut successful = 0;
        let mut failed = 0;
        let mut errors = Vec::new();

        for &id in operations {
            match document.remove(id) {
                Ok(_) => successful += 1,
                Err(e) => {
                    failed += 1;
                    errors.push((id, e.to_string()));
                }
            }
        }

        BatchResult {
            batch_id: generate_batch_id(),
            total_operations: operations.len(),
            successful_operations: successful,
            failed_operations: failed,
            errors,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Perform mixed batch operations
    pub fn execute_batch_mixed<'a>(
        &self,
        operations: &[BatchOperation<'a>],
        document: &mut super::Document,
    ) -> BatchResult {
        let start = std::time::Instant::now();
        let mut successful = 0;
        let mut failed = 0;
        let mut errors = Vec::new();

        for operation in operations {
            match operation {
                BatchOperation::Add(id, content) => match document.add(*id, content) {
                    Ok(_) => successful += 1,
                    Err(e) => {
                        failed += 1;
                        errors.push((*id, e.to_string()));
                    }
                },
                BatchOperation::Update(id, content) => match document.update(*id, content) {
                    Ok(_) => successful += 1,
                    Err(e) => {
                        failed += 1;
                        errors.push((*id, e.to_string()));
                    }
                },
                BatchOperation::Remove(id) => match document.remove(*id) {
                    Ok(_) => successful += 1,
                    Err(e) => {
                        failed += 1;
                        errors.push((*id, e.to_string()));
                    }
                },
                BatchOperation::Replace(id, content) => match document.remove(*id) {
                    Ok(_) => match document.add(*id, content) {
                        Ok(_) => successful += 1,
                        Err(e) => {
                            failed += 1;
                            errors.push((*id, e.to_string()));
                        }
                    },
                    Err(e) => {
                        failed += 1;
                        errors.push((*id, e.to_string()));
                    }
                },
            }
        }

        BatchResult {
            batch_id: generate_batch_id(),
            total_operations: operations.len(),
            successful_operations: successful,
            failed_operations: failed,
            errors,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }
}

impl Default for BatchExecutor {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_batch_add() {
        let mut batch = Batch::new(100);

        let doc1 = json!({"title": "Doc 1"});
        let doc2 = json!({"title": "Doc 2"});
        batch.add(1, &doc1);
        batch.add(2, &doc2);

        assert_eq!(batch.len(), 2);
        assert!(!batch.is_empty());
    }

    #[test]
    fn test_batch_update() {
        let mut batch = Batch::new(100);

        let doc = json!({"title": "Updated"});
        batch.update(1, &doc);

        assert_eq!(batch.len(), 1);
    }

    #[test]
    fn test_batch_remove() {
        let mut batch = Batch::new(100);

        batch.remove(1);
        batch.remove(2);

        assert_eq!(batch.len(), 2);
    }

    #[test]
    fn test_batch_mixed_operations() {
        let mut batch = Batch::new(100);

        let doc1 = json!({"title": "New"});
        let doc2 = json!({"title": "Updated"});
        batch.add(1, &doc1);
        batch.update(2, &doc2);
        batch.remove(3);

        assert_eq!(batch.len(), 3);
    }

    #[test]
    fn test_batch_clear() {
        let mut batch = Batch::new(100);

        let doc1 = json!({"title": "Doc 1"});
        let doc2 = json!({"title": "Doc 2"});
        batch.add(1, &doc1);
        batch.add(2, &doc2);

        batch.clear();

        assert!(batch.is_empty());
    }

    #[test]
    fn test_batch_drain() {
        let mut batch = Batch::new(100);

        let doc1 = json!({"title": "Doc 1"});
        let doc2 = json!({"title": "Doc 2"});
        batch.add(1, &doc1);
        batch.add(2, &doc2);

        let ops = batch.drain();

        assert!(batch.is_empty());
        assert_eq!(ops.len(), 2);
    }

    #[test]
    fn test_batch_executor_fn() {
        let mut batch = Batch::new(100);
        let doc1 = json!({"title": "Doc 1"});
        let doc2 = json!({"title": "Doc 2"});
        batch.add(1, &doc1);
        batch.add(2, &doc2);

        let mut add_count = 0;
        let mut update_count = 0;
        let mut remove_count = 0;

        let mut executor = BatchExecutorFn::new(
            |_, _| {
                add_count += 1;
                Ok(())
            },
            |_, _| {
                update_count += 1;
                Ok(())
            },
            |_, _| {
                remove_count += 1;
                Ok(())
            },
        );

        executor.execute_batch(&batch).unwrap();

        assert_eq!(add_count, 2);
        assert_eq!(update_count, 0);
        assert_eq!(remove_count, 0);
    }

    #[test]
    fn test_batch_executor_creation() {
        let executor = BatchExecutor::new(100);
        assert!(!executor.parallel);
    }

    #[test]
    fn test_batch_executor_with_parallel() {
        let executor = BatchExecutor::new(100).with_parallel(true);
        assert!(executor.parallel);
    }

    #[test]
    fn test_batch_executor_with_max_workers() {
        let executor = BatchExecutor::new(100).with_max_workers(8);
        assert_eq!(executor.max_workers, 8);
    }

    #[test]
    fn test_batch_executor_default() {
        let executor = BatchExecutor::default();
        assert!(!executor.parallel);
    }

    #[test]
    fn test_batch_result_structure() {
        let result = BatchResult {
            batch_id: 1,
            total_operations: 10,
            successful_operations: 8,
            failed_operations: 2,
            errors: vec![(1, "Error".to_string()), (2, "Error".to_string())],
            duration_ms: 100,
        };

        assert_eq!(result.batch_id, 1);
        assert_eq!(result.total_operations, 10);
        assert_eq!(result.successful_operations, 8);
        assert_eq!(result.failed_operations, 2);
        assert_eq!(result.errors.len(), 2);
        assert_eq!(result.duration_ms, 100);
    }

    #[test]
    fn test_batch_metadata_creation() {
        let metadata = BatchMetadata {
            batch_id: 1,
            created_at: chrono::Utc::now(),
            status: BatchStatus::Pending,
            total_operations: 100,
            completed_operations: 0,
            failed_operations: 0,
        };

        assert_eq!(metadata.batch_id, 1);
        assert_eq!(metadata.status, BatchStatus::Pending);
        assert_eq!(metadata.total_operations, 100);
    }

    #[test]
    fn test_batch_status_equality() {
        assert_eq!(BatchStatus::Pending, BatchStatus::Pending);
        assert_eq!(BatchStatus::InProgress, BatchStatus::InProgress);
        assert_eq!(BatchStatus::Completed, BatchStatus::Completed);
        assert_ne!(BatchStatus::Pending, BatchStatus::InProgress);
    }

    #[test]
    fn test_batch_atomic() {
        let batch = Batch::new_atomic(100);
        assert!(batch.is_atomic());
        assert!(!batch.is_transactional());
    }

    #[test]
    fn test_batch_transactional() {
        let batch = Batch::new_transactional(100);
        assert!(batch.is_transactional());
        assert!(!batch.is_atomic());
    }

    #[test]
    fn test_batch_should_flush() {
        let mut batch = Batch::new(10);

        assert!(!batch.should_flush());

        let docs: Vec<serde_json::Value> = (0..10)
            .map(|i| json!({"title": format!("Doc {}", i)}))
            .collect();

        for (i, doc) in docs.iter().enumerate() {
            batch.add(i as DocId, doc);
        }

        assert!(batch.should_flush());
    }

    #[test]
    fn test_batch_replace_operation() {
        let mut batch = Batch::new(100);
        let doc1 = json!({"title": "Original"});
        let doc2 = json!({"title": "Replaced"});

        batch.replace(1, &doc1);
        batch.replace(2, &doc2);

        assert_eq!(batch.len(), 2);
    }
}
