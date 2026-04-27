//! CreateSpaceExecutor – Creates an executor for working with graph spaces.
//!
//! Responsible for creating new graph spaces (single node).

use std::sync::Arc;

use crate::core::types::DataType;
use crate::core::types::SpaceInfo;
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::storage::StorageClient;
use parking_lot::Mutex;

impl SpaceInfo {
    pub fn from_executor(executor_info: &ExecutorSpaceInfo) -> Self {
        let vid_type = match executor_info.vid_type.as_str() {
            "INT64" | "BIGINT" => DataType::BigInt,
            "INT32" | "INT" | "INTEGER" => DataType::Int,
            "INT16" | "SMALLINT" => DataType::SmallInt,
            _ => DataType::String,
        };

        // Use the automatically generated Space ID.
        let space_id = crate::core::types::generate_space_id();

        Self {
            space_id,
            space_name: executor_info.space_name.clone(),
            vid_type,
            tags: Vec::new(),
            edge_types: Vec::new(),
            version: crate::core::types::MetadataVersion::default(),
            comment: None,
            storage_path: None,
            isolation_level: crate::core::types::IsolationLevel::default(),
        }
    }
}

/// Graph space information (used internally by the actuator)
#[derive(Debug, Clone)]
pub struct ExecutorSpaceInfo {
    pub space_name: String,
    pub vid_type: String,
}

impl ExecutorSpaceInfo {
    pub fn new(space_name: String) -> Self {
        Self {
            space_name,
            vid_type: "FIXED_STRING(32)".to_string(),
        }
    }

    pub fn with_vid_type(mut self, vid_type: String) -> Self {
        self.vid_type = vid_type;
        self
    }
}

/// Create a graph space executor
///
/// This executor is responsible for creating new graph spaces in the storage layer.
#[derive(Debug)]
pub struct CreateSpaceExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    space_info: ExecutorSpaceInfo,
    if_not_exists: bool,
}

impl<S: StorageClient> CreateSpaceExecutor<S> {
    /// Create a new instance of the CreateSpaceExecutor class.
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        space_info: ExecutorSpaceInfo,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "CreateSpaceExecutor".to_string(), storage, expr_context),
            space_info,
            if_not_exists: false,
        }
    }

    /// Create a CreateSpaceExecutor with the IF NOT EXISTS option
    pub fn with_if_not_exists(
        id: i64,
        storage: Arc<Mutex<S>>,
        space_info: ExecutorSpaceInfo,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "CreateSpaceExecutor".to_string(), storage, expr_context),
            space_info,
            if_not_exists: true,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for CreateSpaceExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let mut storage_guard = storage.lock();

        let metadata_space_info = SpaceInfo::from_executor(&self.space_info);
        let result = storage_guard.create_space(&metadata_space_info);

        match result {
            Ok(true) => Ok(ExecutionResult::Success),
            Ok(false) => {
                if self.if_not_exists {
                    Ok(ExecutionResult::Success)
                } else {
                    Ok(ExecutionResult::Error(format!(
                        "Space '{}' already exists",
                        self.space_info.space_name
                    )))
                }
            }
            Err(e) => Ok(ExecutionResult::Error(format!(
                "Failed to create space: {}",
                e
            ))),
        }
    }

    fn open(&mut self) -> crate::query::executor::base::DBResult<()> {
        self.base.open()
    }

    fn close(&mut self) -> crate::query::executor::base::DBResult<()> {
        self.base.close()
    }

    fn is_open(&self) -> bool {
        self.base.is_open()
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        "CreateSpaceExecutor"
    }

    fn description(&self) -> &str {
        "Creates a new graph space"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> crate::query::executor::base::HasStorage<S> for CreateSpaceExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}
