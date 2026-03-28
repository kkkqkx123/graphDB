//! Executor Builder Module
//!
//! Responsible for creating various types of actuators

pub mod admin_builder;
pub mod control_flow_builder;
pub mod data_access_builder;
pub mod data_modification_builder;
pub mod data_processing_builder;
pub mod join_builder;
pub mod set_operation_builder;
pub mod transformation_builder;
pub mod traversal_builder;

pub use admin_builder::AdminBuilder;
pub use control_flow_builder::ControlFlowBuilder;
pub use data_access_builder::DataAccessBuilder;
pub use data_modification_builder::DataModificationBuilder;
pub use data_processing_builder::DataProcessingBuilder;
pub use join_builder::JoinBuilder;
pub use set_operation_builder::SetOperationBuilder;
pub use transformation_builder::TransformationBuilder;
pub use traversal_builder::TraversalBuilder;

use crate::storage::StorageClient;

/// Collection of builders
///
/// Simplified structure - all builder methods are now associated functions.
/// This struct serves as a marker type and provides a unified interface through
/// the individual builder types.
pub struct Builders<S: StorageClient + Send + 'static> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient + Send + 'static> Builders<S> {
    /// Create a new set of builders.
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<S: StorageClient + 'static> Clone for Builders<S> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<S: StorageClient + 'static> Default for Builders<S> {
    fn default() -> Self {
        Self::new()
    }
}
