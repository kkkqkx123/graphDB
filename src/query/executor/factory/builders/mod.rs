//! 执行器构建器模块
//!
//! 负责创建各种类型的执行器

pub mod data_access_builder;
pub mod data_modification_builder;
pub mod data_processing_builder;
pub mod join_builder;
pub mod set_operation_builder;
pub mod traversal_builder;
pub mod transformation_builder;
pub mod control_flow_builder;
pub mod admin_builder;

pub use data_access_builder::DataAccessBuilder;
pub use data_modification_builder::DataModificationBuilder;
pub use data_processing_builder::DataProcessingBuilder;
pub use join_builder::JoinBuilder;
pub use set_operation_builder::SetOperationBuilder;
pub use traversal_builder::TraversalBuilder;
pub use transformation_builder::TransformationBuilder;
pub use control_flow_builder::ControlFlowBuilder;
pub use admin_builder::AdminBuilder;

use crate::storage::StorageClient;

/// 构建器集合
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
    /// 创建新的构建器集合
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

    /// 获取数据访问构建器
    pub fn data_access(&self) -> &DataAccessBuilder<S> {
        &self.data_access
    }

    /// 获取数据修改构建器
    pub fn data_modification(&self) -> &DataModificationBuilder<S> {
        &self.data_modification
    }

    /// 获取数据处理构建器
    pub fn data_processing(&self) -> &DataProcessingBuilder<S> {
        &self.data_processing
    }

    /// 获取连接构建器
    pub fn join(&self) -> &JoinBuilder<S> {
        &self.join
    }

    /// 获取集合操作构建器
    pub fn set_operation(&self) -> &SetOperationBuilder<S> {
        &self.set_operation
    }

    /// 获取图遍历构建器
    pub fn traversal(&self) -> &TraversalBuilder<S> {
        &self.traversal
    }

    /// 获取数据转换构建器
    pub fn transformation(&self) -> &TransformationBuilder<S> {
        &self.transformation
    }

    /// 获取控制流构建器
    pub fn control_flow(&self) -> &ControlFlowBuilder<S> {
        &self.control_flow
    }

    /// 获取管理执行器构建器
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
