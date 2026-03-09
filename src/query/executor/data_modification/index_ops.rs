//! 索引操作执行器
//!
//! 负责创建和删除索引

use std::sync::Arc;
use std::time::Instant;

use crate::query::executor::base::{BaseExecutor, ExecutorStats};
use crate::core::types::Index;
use crate::query::executor::base::{DBResult, ExecutionResult, Executor, HasStorage};
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::storage::StorageClient;
use parking_lot::Mutex;

/// 创建索引执行器
///
/// 负责在存储层创建索引
pub struct CreateIndexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    index_name: String,
    index_type: crate::core::types::IndexType,
    properties: Vec<String>,
    tag_name: Option<String>,
}

impl<S: StorageClient> CreateIndexExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        index_name: String,
        index_type: crate::core::types::IndexType,
        properties: Vec<String>,
        tag_name: Option<String>,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "CreateIndexExecutor".to_string(), storage, expr_context),
            index_name,
            index_type,
            properties,
            tag_name,
        }
    }
}

impl<S: StorageClient> HasStorage<S> for CreateIndexExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for CreateIndexExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();
        let result = self.do_execute();
        let elapsed = start.elapsed();
        self.base.get_stats_mut().add_total_time(elapsed);
        match result {
            Ok(_) => Ok(ExecutionResult::Empty),
            Err(e) => Err(e),
        }
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        "CreateIndexExecutor"
    }

    fn description(&self) -> &str {
        "Create index executor - creates indexes in storage"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient + Send + Sync + 'static> CreateIndexExecutor<S> {
    fn do_execute(&mut self) -> DBResult<()> {
        let mut storage = self.get_storage().lock();

        let target_name = self
            .tag_name
            .clone()
            .or_else(|| Some(self.index_name.clone()))
            .unwrap_or_default();

        let index_type = self.index_type.clone();
        let index = Index::new(
            0,
            self.index_name.clone(),
            0,
            target_name,
            Vec::new(),
            self.properties.clone(),
            index_type.clone(),
            false,
        );

        match index_type {
            crate::core::types::IndexType::TagIndex => {
                storage.create_tag_index("default", &index)?;
            }
            crate::core::types::IndexType::EdgeIndex => {
                storage.create_edge_index("default", &index)?;
            }
        }

        Ok(())
    }
}

/// 删除索引执行器
///
/// 负责从存储层删除索引
pub struct DropIndexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    _index_name: String,
}

impl<S: StorageClient> DropIndexExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        _index_name: String,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "DropIndexExecutor".to_string(), storage, expr_context),
            _index_name,
        }
    }
}

impl<S: StorageClient> HasStorage<S> for DropIndexExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for DropIndexExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();
        let result = self.do_execute();
        let elapsed = start.elapsed();
        self.base.get_stats_mut().add_total_time(elapsed);
        match result {
            Ok(_) => Ok(ExecutionResult::Empty),
            Err(e) => Err(e),
        }
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        "DropIndexExecutor"
    }

    fn description(&self) -> &str {
        "Drop index executor - drops indexes from storage"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient + Send + Sync + 'static> DropIndexExecutor<S> {
    fn do_execute(&mut self) -> DBResult<()> {
        let mut storage = self.get_storage().lock();

        storage.drop_tag_index("default", &self._index_name)?;

        Ok(())
    }
}
