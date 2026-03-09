//! 插入执行器
//!
//! 负责插入顶点和边数据到存储层

use std::sync::Arc;
use std::time::Instant;

use crate::query::executor::base::{BaseExecutor, ExecutorStats};
use crate::core::{Edge, Vertex};
use crate::query::executor::base::{DBResult, ExecutionResult, Executor, HasStorage};
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::storage::StorageClient;
use parking_lot::Mutex;

/// 插入执行器
///
/// 负责插入新的顶点和边数据
pub struct InsertExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    vertex_data: Option<Vec<Vertex>>,
    edge_data: Option<Vec<Edge>>,
    if_not_exists: bool,
}

impl<S: StorageClient> InsertExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        vertex_data: Option<Vec<Vertex>>,
        edge_data: Option<Vec<Edge>>,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "InsertExecutor".to_string(), storage, expr_context),
            vertex_data,
            edge_data,
            if_not_exists: false,
        }
    }

    pub fn with_vertices(
        id: i64,
        storage: Arc<Mutex<S>>,
        vertex_data: Vec<Vertex>,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "InsertExecutor".to_string(), storage, expr_context),
            vertex_data: Some(vertex_data),
            edge_data: None,
            if_not_exists: false,
        }
    }

    pub fn with_edges(
        id: i64,
        storage: Arc<Mutex<S>>,
        edge_data: Vec<Edge>,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "InsertExecutor".to_string(), storage, expr_context),
            vertex_data: None,
            edge_data: Some(edge_data),
            if_not_exists: false,
        }
    }

    /// 创建带 IF NOT EXISTS 选项的 InsertExecutor
    pub fn with_vertices_if_not_exists(
        id: i64,
        storage: Arc<Mutex<S>>,
        vertex_data: Vec<Vertex>,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "InsertExecutor".to_string(), storage, expr_context),
            vertex_data: Some(vertex_data),
            edge_data: None,
            if_not_exists: true,
        }
    }

    /// 创建带 IF NOT EXISTS 选项的 InsertExecutor（用于边）
    pub fn with_edges_if_not_exists(
        id: i64,
        storage: Arc<Mutex<S>>,
        edge_data: Vec<Edge>,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "InsertExecutor".to_string(), storage, expr_context),
            vertex_data: None,
            edge_data: Some(edge_data),
            if_not_exists: true,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for InsertExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();
        let result = self.do_execute();
        let elapsed = start.elapsed();
        self.base.get_stats_mut().add_total_time(elapsed);
        match result {
            Ok(count) => Ok(ExecutionResult::Count(count)),
            Err(e) => Err(e),
        }
    }

    fn open(&mut self) -> Result<(), crate::core::DBError> {
        Ok(())
    }

    fn close(&mut self) -> Result<(), crate::core::DBError> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        "InsertExecutor"
    }

    fn description(&self) -> &str {
        "Insert executor - inserts vertices and edges into storage"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for InsertExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient + Send + Sync + 'static> InsertExecutor<S> {
    fn do_execute(&mut self) -> DBResult<usize> {
        let mut total_inserted = 0;

        if let Some(vertices) = &self.vertex_data {
            let mut storage = self.get_storage().lock();
            for vertex in vertices {
                // 如果启用了 IF NOT EXISTS，检查顶点是否已存在
                if self.if_not_exists {
                    if storage.get_vertex("default", &vertex.vid)?.is_some() {
                        // 顶点已存在，跳过插入
                        continue;
                    }
                }
                storage.insert_vertex("default", vertex.clone())?;
                total_inserted += 1;
            }
        }

        if let Some(edges) = &self.edge_data {
            let mut storage = self.get_storage().lock();
            for edge in edges {
                storage.insert_edge("default", edge.clone())?;
                total_inserted += 1;
            }
        }

        Ok(total_inserted)
    }
}
