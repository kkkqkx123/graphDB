//! 全文索引管理执行器
//!
//! 提供全文索引的DDL操作支持：
//! - CREATE FULLTEXT INDEX
//! - DROP FULLTEXT INDEX
//! - SHOW FULLTEXT INDEXES
//! - REBUILD FULLTEXT INDEX
use std::sync::Arc;
use parking_lot::Mutex;

use crate::core::error::{DBError, DBResult};
use crate::core::Value;
use crate::index::{
    create_default_fulltext_manager, FulltextIndexConfig, FulltextIndexManager,
    FulltextSchemaType,
};
use crate::query::executor::base::{BaseExecutor, ExecutorStats};
use crate::query::executor::traits::{ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;
use parking_lot::Mutex;

pub struct CreateFTIndexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    index_name: String,
    schema_type: FulltextSchemaType,
    schema_name: String,
    fields: Vec<String>,
    analyzer: Option<String>,
}

impl<S: StorageClient> CreateFTIndexExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        index_name: String,
        schema_type: FulltextSchemaType,
        schema_name: String,
        fields: Vec<String>,
        analyzer: Option<String>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "CreateFTIndexExecutor".to_string(), storage),
            index_name,
            schema_type,
            schema_name,
            fields,
            analyzer,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for CreateFTIndexExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let result = self.do_execute();
        result.map(|_| ExecutionResult::Empty)
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
        "CreateFTIndexExecutor"
    }

    fn description(&self) -> &str {
        "Create fulltext index executor"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient + Send + Sync + 'static> CreateFTIndexExecutor<S> {
    async fn do_execute(&mut self) -> DBResult<()> {
        let storage = self.get_storage().lock();
        let space = storage.get_space("default")?;

        let mut ft_manager = space.get_fulltext_index_manager_mut()?;
        ft_manager.create_fulltext_index(
            self.index_name.clone(),
            self.schema_type.clone(),
            self.schema_name.clone(),
            self.fields.clone(),
            self.analyzer.clone(),
        )?;

        Ok(())
    }
}

pub struct DropFTIndexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    index_name: String,
}

impl<S: StorageClient> DropFTIndexExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, index_name: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "DropFTIndexExecutor".to_string(), storage),
            index_name,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for DropFTIndexExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let result = self.do_execute();
        result.map(|_| ExecutionResult::Empty)
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
        "DropFTIndexExecutor"
    }

    fn description(&self) -> &str {
        "Drop fulltext index executor"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient + Send + Sync + 'static> DropFTIndexExecutor<S> {
    async fn do_execute(&mut self) -> DBResult<()> {
        let storage = self.get_storage().lock();
        let space = storage.get_space("default")?;

        let mut ft_manager = space.get_fulltext_index_manager_mut()?;
        ft_manager.drop_fulltext_index(&self.index_name)?;

        Ok(())
    }
}

pub struct ShowFTIndexesExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
}

impl<S: StorageClient> ShowFTIndexesExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>) -> Self {
        Self {
            base: BaseExecutor::new(id, "ShowFTIndexesExecutor".to_string(), storage),
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for ShowFTIndexesExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let result = self.do_execute()?;
        Ok(ExecutionResult::DataSet(result))
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
        "ShowFTIndexesExecutor"
    }

    fn description(&self) -> &str {
        "Show fulltext indexes executor"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient + Send + Sync + 'static> ShowFTIndexesExecutor<S> {
    async fn do_execute(&mut self) -> DBResult<Vec<Vec<Value>>> {
        let storage = self.get_storage().lock();
        let space = storage.get_space("default")?;

        let ft_manager = space.get_fulltext_index_manager()?;
        let indexes = ft_manager.list_fulltext_indexes()?;

        let mut results = Vec::new();
        results.push(vec![
            Value::String("Name".to_string()),
            Value::String("Schema Type".to_string()),
            Value::String("Schema Name".to_string()),
            Value::String("Fields".to_string()),
            Value::String("Analyzer".to_string()),
        ]);

        for index in indexes {
            let fields_str = index.fields.join(", ");
            let analyzer = index.analyzer.unwrap_or_else(|| "default".to_string());
            let schema_type_str = match index.schema_type {
                FulltextSchemaType::Tag => "Tag",
                FulltextSchemaType::Edge => "Edge",
            };

            results.push(vec![
                Value::String(index.name),
                Value::String(schema_type_str.to_string()),
                Value::String(index.schema_name),
                Value::String(fields_str),
                Value::String(analyzer),
            ]);
        }

        Ok(results)
    }
}

pub struct FulltextIndexScanExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    index_name: String,
    query_string: String,
    limit: usize,
    offset: usize,
}

impl<S: StorageClient> FulltextIndexScanExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        index_name: String,
        query_string: String,
        limit: usize,
        offset: usize,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "FulltextIndexScanExecutor".to_string(), storage),
            index_name,
            query_string,
            limit,
            offset,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for FulltextIndexScanExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let result = self.do_execute()?;
        Ok(ExecutionResult::DataSet(result))
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
        "FulltextIndexScanExecutor"
    }

    fn description(&self) -> &str {
        "Fulltext index scan executor"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient + Send + Sync + 'static> FulltextIndexScanExecutor<S> {
    async fn do_execute(&mut self) -> DBResult<Vec<Vec<Value>>> {
        let storage = self.get_storage().lock();
        let space = storage.get_space("default")?;

        let ft_manager = space.get_fulltext_index_manager()?;
        let query = crate::index::FulltextQuery::new(self.index_name.clone(), self.query_string.clone())
            .with_limit(self.limit)
            .with_offset(self.offset);

        let search_results = ft_manager.search(query)?;

        let mut results = Vec::new();
        results.push(vec![
            Value::String("VertexID".to_string()),
            Value::String("Score".to_string()),
        ]);

        for result in search_results {
            results.push(vec![
                Value::String(result.id),
                Value::Float(result.score as f64),
            ]);
        }

        Ok(results)
    }
}
