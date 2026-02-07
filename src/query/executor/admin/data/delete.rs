//! 删除执行器
//!
//! 提供顶点和边的删除功能。

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;

/// 删除目标类型
#[derive(Debug, Clone)]
pub enum DeleteTarget {
    Vertex(String),
    Edge(String, String, i64),
}

/// 删除执行器
///
/// 该执行器负责删除图中的顶点或边。
#[derive(Debug)]
pub struct DeleteExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    space_name: String,
    target: DeleteTarget,
}

impl<S: StorageClient> DeleteExecutor<S> {
    /// 创建新的 DeleteExecutor
    pub fn new(id: i64, storage: Arc<Mutex<S>>, space_name: String, target: DeleteTarget) -> Self {
        Self {
            base: BaseExecutor::new(id, "DeleteExecutor".to_string(), storage),
            space_name,
            target,
        }
    }

    /// 创建删除顶点的 DeleteExecutor
    pub fn delete_vertex(id: i64, storage: Arc<Mutex<S>>, space_name: String, vertex_id: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "DeleteExecutor".to_string(), storage),
            space_name,
            target: DeleteTarget::Vertex(vertex_id),
        }
    }

    /// 创建删除边的 DeleteExecutor
    pub fn delete_edge(id: i64, storage: Arc<Mutex<S>>, space_name: String,
                       src_vertex_id: String, dst_vertex_id: String, rank: i64) -> Self {
        Self {
            base: BaseExecutor::new(id, "DeleteExecutor".to_string(), storage),
            space_name,
            target: DeleteTarget::Edge(src_vertex_id, dst_vertex_id, rank),
        }
    }
}

#[async_trait]
impl<S: StorageClient + Send + Sync + 'static> Executor<S> for DeleteExecutor<S> {
    async fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let mut storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::Storage(
                crate::core::error::StorageError::DbError(format!("Storage lock poisoned: {}", e))
            )
        })?;

        let result = match &self.target {
            DeleteTarget::Vertex(vertex_id) => {
                storage_guard.delete_vertex_data(&self.space_name, vertex_id)
            }
            DeleteTarget::Edge(src_id, dst_id, rank) => {
                storage_guard.delete_edge_data(&self.space_name, src_id, dst_id, *rank)
            }
        };

        match result {
            Ok(true) => Ok(ExecutionResult::Success),
            Ok(false) => {
                let target_desc = match &self.target {
                    DeleteTarget::Vertex(id) => format!("vertex {}", id),
                    DeleteTarget::Edge(src, dst, rank) => format!("edge from {} to {} with rank {}", src, dst, rank),
                };
                Ok(ExecutionResult::Error(format!("{} not found", target_desc)))
            }
            Err(e) => Ok(ExecutionResult::Error(format!("Failed to delete: {}", e))),
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
        "DeleteExecutor"
    }

    fn description(&self) -> &str {
        "Deletes a vertex or edge"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> crate::query::executor::base::HasStorage<S> for DeleteExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}
