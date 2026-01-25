//! 图查询执行器
//!
//! 提供图查询语言（Cypher/NGQL）的执行功能
//! 支持MATCH、CREATE、DELETE等图操作语句

use crate::core::error::{DBError, DBResult, QueryError};
use crate::query::executor::admin as admin_executor;
use crate::query::executor::traits::{ExecutionResult, Executor, HasStorage};
use crate::query::parser::ast::stmt::{AlterStmt, ChangePasswordStmt, DescStmt, DropStmt, Stmt};
use crate::storage::StorageEngine;
use crate::common::thread::ThreadPool;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

/// 图查询执行器
///
/// 提供图查询执行的基础功能，包括：
/// - 执行上下文管理
/// - 语句分发
/// - 错误处理
/// - 资源管理
pub struct GraphQueryExecutor<S: StorageEngine> {
    /// 执行器ID
    id: i64,
    /// 执行器名称
    name: String,
    /// 执行器描述
    description: String,
    /// 存储引擎引用
    storage: Arc<Mutex<S>>,
    /// 线程池用于并行执行查询
    thread_pool: Option<Arc<ThreadPool>>,
    /// 是否已打开
    is_open: bool,
    /// 执行统计信息
    stats: crate::query::executor::traits::ExecutorStats,
}

impl<S: StorageEngine> std::fmt::Debug for GraphQueryExecutor<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphQueryExecutor")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("description", &self.description)
            .field("is_open", &self.is_open)
            .field("stats", &self.stats)
            .finish()
    }
}

impl<S: StorageEngine + 'static> GraphQueryExecutor<S> {
    /// 创建新的图查询执行器
    pub fn new(id: i64, storage: Arc<Mutex<S>>) -> Self {
        let thread_pool = Some(Arc::new(ThreadPool::new(4)));
        Self {
            id,
            name: "GraphQueryExecutor".to_string(),
            description: "图查询语言执行器".to_string(),
            storage,
            thread_pool,
            is_open: false,
            stats: crate::query::executor::traits::ExecutorStats::new(),
        }
    }

    /// 带名称创建执行器
    pub fn with_name(id: i64, name: String, storage: Arc<Mutex<S>>) -> Self {
        let thread_pool = Some(Arc::new(ThreadPool::new(4)));
        Self {
            id,
            name,
            description: "图查询语言执行器".to_string(),
            storage,
            thread_pool,
            is_open: false,
            stats: crate::query::executor::traits::ExecutorStats::new(),
        }
    }

    /// 带名称和描述创建执行器
    pub fn with_description(
        id: i64,
        name: String,
        description: String,
        storage: Arc<Mutex<S>>,
    ) -> Self {
        let thread_pool = Some(Arc::new(ThreadPool::new(4)));
        Self {
            id,
            name,
            description,
            storage,
            thread_pool,
            is_open: false,
            stats: crate::query::executor::traits::ExecutorStats::new(),
        }
    }

    /// 执行具体的语句
    async fn execute_statement(
        &mut self,
        statement: Stmt,
    ) -> Result<ExecutionResult, DBError> {
        match statement {
            Stmt::Match(clause) => self.execute_match(clause).await,
            Stmt::Create(clause) => self.execute_create(clause).await,
            Stmt::Delete(clause) => self.execute_delete(clause).await,
            Stmt::Update(clause) => self.execute_update(clause).await,
            Stmt::Query(clause) => self.execute_query(clause).await,
            Stmt::Go(clause) => self.execute_go(clause).await,
            Stmt::Fetch(clause) => self.execute_fetch(clause).await,
            Stmt::Lookup(clause) => self.execute_lookup(clause).await,
            Stmt::FindPath(clause) => self.execute_find_path(clause).await,
            Stmt::Use(clause) => self.execute_use(clause).await,
            Stmt::Show(clause) => self.execute_show(clause).await,
            Stmt::Explain(clause) => self.execute_explain(clause).await,
            Stmt::Subgraph(clause) => self.execute_subgraph(clause).await,
            Stmt::Insert(clause) => self.execute_insert(clause).await,
            Stmt::Merge(clause) => self.execute_merge(clause).await,
            Stmt::Unwind(clause) => self.execute_unwind(clause).await,
            Stmt::Return(clause) => self.execute_return(clause).await,
            Stmt::With(clause) => self.execute_with(clause).await,
            Stmt::Set(clause) => self.execute_set(clause).await,
            Stmt::Remove(clause) => self.execute_remove(clause).await,
            Stmt::Pipe(clause) => self.execute_pipe(clause).await,
            Stmt::Drop(clause) => self.execute_drop(clause).await,
            Stmt::Desc(clause) => self.execute_desc(clause).await,
            Stmt::Alter(clause) => self.execute_alter(clause).await,
            Stmt::ChangePassword(clause) => self.execute_change_password(clause).await,
        }
    }

    async fn execute_match(&mut self, _clause: crate::query::parser::ast::stmt::MatchStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("MATCH语句执行未实现".to_string())))
    }

    async fn execute_create(&mut self, _clause: crate::query::parser::ast::stmt::CreateStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("CREATE语句执行未实现".to_string())))
    }

    async fn execute_delete(&mut self, _clause: crate::query::parser::ast::stmt::DeleteStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("DELETE语句执行未实现".to_string())))
    }

    async fn execute_update(&mut self, _clause: crate::query::parser::ast::stmt::UpdateStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("UPDATE语句执行未实现".to_string())))
    }

    async fn execute_query(&mut self, _clause: crate::query::parser::ast::stmt::QueryStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("QUERY语句执行未实现".to_string())))
    }

    async fn execute_go(&mut self, _clause: crate::query::parser::ast::stmt::GoStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("GO语句执行未实现".to_string())))
    }

    async fn execute_fetch(&mut self, _clause: crate::query::parser::ast::stmt::FetchStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("FETCH语句执行未实现".to_string())))
    }

    async fn execute_lookup(&mut self, _clause: crate::query::parser::ast::stmt::LookupStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("LOOKUP语句执行未实现".to_string())))
    }

    async fn execute_find_path(&mut self, _clause: crate::query::parser::ast::stmt::FindPathStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("FIND PATH语句执行未实现".to_string())))
    }

    async fn execute_use(&mut self, _clause: crate::query::parser::ast::stmt::UseStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("USE语句执行未实现".to_string())))
    }

    async fn execute_show(&mut self, _clause: crate::query::parser::ast::stmt::ShowStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("SHOW语句执行未实现".to_string())))
    }

    async fn execute_explain(&mut self, _clause: crate::query::parser::ast::stmt::ExplainStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("EXPLAIN语句执行未实现".to_string())))
    }

    async fn execute_subgraph(&mut self, _clause: crate::query::parser::ast::stmt::SubgraphStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("SUBGRAPH语句执行未实现".to_string())))
    }

    async fn execute_insert(&mut self, _clause: crate::query::parser::ast::stmt::InsertStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("INSERT语句执行未实现".to_string())))
    }

    async fn execute_merge(&mut self, _clause: crate::query::parser::ast::stmt::MergeStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("MERGE语句执行未实现".to_string())))
    }

    async fn execute_unwind(&mut self, _clause: crate::query::parser::ast::stmt::UnwindStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("UNWIND语句执行未实现".to_string())))
    }

    async fn execute_return(&mut self, _clause: crate::query::parser::ast::stmt::ReturnStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("RETURN语句执行未实现".to_string())))
    }

    async fn execute_with(&mut self, _clause: crate::query::parser::ast::stmt::WithStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("WITH语句执行未实现".to_string())))
    }

    async fn execute_set(&mut self, _clause: crate::query::parser::ast::stmt::SetStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("SET语句执行未实现".to_string())))
    }

    async fn execute_remove(&mut self, _clause: crate::query::parser::ast::stmt::RemoveStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("REMOVE语句执行未实现".to_string())))
    }

    async fn execute_pipe(&mut self, _clause: crate::query::parser::ast::stmt::PipeStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("PIPE语句执行未实现".to_string())))
    }

    async fn execute_drop(&mut self, clause: DropStmt) -> Result<ExecutionResult, DBError> {
        use crate::query::parser::ast::stmt::DropTarget;
        let id = self.id;

        match clause.target {
            DropTarget::Space(space_name) => {
                let mut executor = admin_executor::DropSpaceExecutor::new(id, self.storage.clone(), space_name);
                executor.open()?;
                executor.execute().await.map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))
            }
            DropTarget::Tag { space_name, tag_name } => {
                let mut executor = admin_executor::DropTagExecutor::new(id, self.storage.clone(), space_name, tag_name);
                executor.open()?;
                executor.execute().await.map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))
            }
            DropTarget::Edge { space_name, edge_name } => {
                let mut executor = admin_executor::DropEdgeExecutor::new(id, self.storage.clone(), space_name, edge_name);
                executor.open()?;
                executor.execute().await.map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))
            }
            DropTarget::TagIndex { space_name, index_name } => {
                let mut executor = admin_executor::DropTagIndexExecutor::new(id, self.storage.clone(), space_name, index_name);
                executor.open()?;
                executor.execute().await.map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))
            }
            DropTarget::EdgeIndex { space_name, index_name } => {
                let mut executor = admin_executor::DropEdgeIndexExecutor::new(id, self.storage.clone(), space_name, index_name);
                executor.open()?;
                executor.execute().await.map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))
            }
        }
    }

    async fn execute_desc(&mut self, clause: DescStmt) -> Result<ExecutionResult, DBError> {
        use crate::query::parser::ast::stmt::DescTarget;
        let id = self.id;

        match clause.target {
            DescTarget::Space(space_name) => {
                let mut executor = admin_executor::DescSpaceExecutor::new(id, self.storage.clone(), space_name);
                executor.open()?;
                executor.execute().await.map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))
            }
            DescTarget::Tag { space_name, tag_name } => {
                let mut executor = admin_executor::DescTagExecutor::new(id, self.storage.clone(), space_name, tag_name);
                executor.open()?;
                executor.execute().await.map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))
            }
            DescTarget::Edge { space_name, edge_name } => {
                let mut executor = admin_executor::DescEdgeExecutor::new(id, self.storage.clone(), space_name, edge_name);
                executor.open()?;
                executor.execute().await.map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))
            }
        }
    }

    async fn execute_alter(&mut self, clause: AlterStmt) -> Result<ExecutionResult, DBError> {
        use crate::query::parser::ast::stmt::AlterTarget;
        use admin_executor::{AlterEdgeExecutor, AlterTagExecutor, AlterEdgeInfo, AlterTagInfo, AlterTagItem, AlterEdgeItem};
        let id = self.id;

        match clause.target {
            AlterTarget::Tag { space_name, tag_name, additions, deletions: _ } => {
                let mut items = Vec::new();
                for prop in additions {
                    items.push(AlterTagItem::add_property(prop));
                }
                let alter_info = AlterTagInfo::new(space_name, tag_name).with_items(items);
                let mut executor = AlterTagExecutor::new(id, self.storage.clone(), alter_info);
                executor.open()?;
                executor.execute().await.map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))
            }
            AlterTarget::Edge { space_name, edge_name, additions, deletions: _ } => {
                let mut items = Vec::new();
                for prop in additions {
                    items.push(AlterEdgeItem::add_property(prop));
                }
                let alter_info = AlterEdgeInfo::new(space_name, edge_name).with_items(items);
                let mut executor = AlterEdgeExecutor::new(id, self.storage.clone(), alter_info);
                executor.open()?;
                executor.execute().await.map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))
            }
        }
    }

    async fn execute_change_password(&mut self, clause: ChangePasswordStmt) -> Result<ExecutionResult, DBError> {
        use admin_executor::{ChangePasswordExecutor, PasswordInfo};
        let id = self.id;

        let password_info = PasswordInfo {
            username: clause.username,
            old_password: clause.old_password,
            new_password: clause.new_password,
        };
        let mut executor = ChangePasswordExecutor::new(id, self.storage.clone(), password_info);
        executor.open()?;
        executor.execute().await.map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))
    }
}

#[async_trait]
impl<S: StorageEngine> Executor<S> for GraphQueryExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        Err(DBError::Query(QueryError::ExecutionError("需要先设置要执行的语句".to_string())))
    }

    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn open(&mut self) -> DBResult<()> {
        self.is_open = true;
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        self.is_open = false;
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.is_open
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        &self.stats
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        &mut self.stats
    }
}

#[async_trait]
impl<S: StorageEngine> HasStorage<S> for GraphQueryExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        &self.storage
    }
}
