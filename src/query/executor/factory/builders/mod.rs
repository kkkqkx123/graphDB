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
pub struct Builders<S: StorageClient + 'static> {
    data_access: DataAccessBuilder<S>,
    data_modification: DataModificationBuilder<S>,
    data_processing: DataProcessingBuilder<S>,
    join: JoinBuilder<S>,
    set_operation: SetOperationBuilder<S>,
    traversal: TraversalBuilder<S>,
    transformation: TransformationBuilder<S>,
    control_flow: ControlFlowBuilder<S>,
    admin: AdminBuilder<S>,
}

impl<S: StorageClient + 'static> Builders<S> {
    /// Create a new set of builders.
    pub fn new() -> Self {
        Self {
            data_access: DataAccessBuilder::new(),
            data_modification: DataModificationBuilder::new(),
            data_processing: DataProcessingBuilder::new(),
            join: JoinBuilder::new(),
            set_operation: SetOperationBuilder::new(),
            traversal: TraversalBuilder::new(),
            transformation: TransformationBuilder::new(),
            control_flow: ControlFlowBuilder::new(),
            admin: AdminBuilder::new(),
        }
    }

    /// Obtain the data access builder.
    pub fn data_access(&self) -> &DataAccessBuilder<S> {
        &self.data_access
    }

    /// Obtain the data modification builder.
    pub fn data_modification(&self) -> &DataModificationBuilder<S> {
        &self.data_modification
    }

    /// Obtain the data processing builder.
    pub fn data_processing(&self) -> &DataProcessingBuilder<S> {
        &self.data_processing
    }

    /// Obtain the connection builder.
    pub fn join(&self) -> &JoinBuilder<S> {
        &self.join
    }

    /// Obtaining the set operation builder
    pub fn set_operation(&self) -> &SetOperationBuilder<S> {
        &self.set_operation
    }

    /// Obtain the graph traversal builder.
    pub fn traversal(&self) -> &TraversalBuilder<S> {
        &self.traversal
    }

    /// Obtain the Data Conversion Builder
    pub fn transformation(&self) -> &TransformationBuilder<S> {
        &self.transformation
    }

    /// Obtain the control flow builder.
    pub fn control_flow(&self) -> &ControlFlowBuilder<S> {
        &self.control_flow
    }

    /// Obtain the Management Executor Builder
    pub fn admin(&self) -> &AdminBuilder<S> {
        &self.admin
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
