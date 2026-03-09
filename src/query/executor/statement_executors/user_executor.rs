use crate::core::error::{DBError, DBResult};
use crate::core::types::{UserAlterInfo, UserInfo};
use crate::query::executor::admin as admin_executor;
use crate::query::executor::base::{ExecutionResult, Executor};
use crate::query::parser::ast::stmt::{AlterUserStmt, ChangePasswordStmt, CreateUserStmt, DropUserStmt};
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

pub struct UserExecutor<S: StorageClient> {
    id: i64,
    storage: Arc<Mutex<S>>,
}

impl<S: StorageClient> UserExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>) -> Self {
        Self { id, storage }
    }

    pub fn execute_create_user(&self, clause: CreateUserStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        use admin_executor::CreateUserExecutor;

        let user_info =
            UserInfo::new(clause.username, clause.password).map_err(|e| DBError::Storage(e))?;
        let mut executor = CreateUserExecutor::new(
            self.id,
            self.storage.clone(),
            user_info,
            Arc::new(ExpressionAnalysisContext::new()),
        );
        Executor::open(&mut executor)?;
        Executor::execute(&mut executor)
    }

    pub fn execute_alter_user(&self, clause: AlterUserStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        use admin_executor::AlterUserExecutor;

        let mut alter_info = UserAlterInfo::new(clause.username);
        if let Some(is_locked) = clause.is_locked {
            alter_info.is_locked = Some(is_locked);
        }
        let mut executor = AlterUserExecutor::new(
            self.id,
            self.storage.clone(),
            alter_info,
            Arc::new(ExpressionAnalysisContext::new()),
        );
        Executor::open(&mut executor)?;
        Executor::execute(&mut executor)
    }

    pub fn execute_drop_user(&self, clause: DropUserStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        use admin_executor::DropUserExecutor;

        let mut executor = DropUserExecutor::new(
            self.id,
            self.storage.clone(),
            clause.username,
            Arc::new(ExpressionAnalysisContext::new()),
        );
        Executor::open(&mut executor)?;
        Executor::execute(&mut executor)
    }

    pub fn execute_change_password(&self, clause: ChangePasswordStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        use admin_executor::ChangePasswordExecutor;

        let mut executor = ChangePasswordExecutor::new(
            self.id,
            self.storage.clone(),
            clause.username,
            clause.old_password,
            clause.new_password,
            Arc::new(ExpressionAnalysisContext::new()),
        );
        Executor::open(&mut executor)?;
        Executor::execute(&mut executor)
    }
}
