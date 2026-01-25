//! 用户管理执行器
//!
//! 提供用户密码管理和认证功能。

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::query::executor::base::{BaseExecutor, Executor, HasStorage};
use crate::storage::StorageEngine;

/// 密码信息
#[derive(Debug, Clone)]
pub struct PasswordInfo {
    pub username: String,
    pub old_password: String,
    pub new_password: String,
}

/// 变更密码执行器
///
/// 该执行器负责变更用户密码。
#[derive(Debug)]
pub struct ChangePasswordExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    password_info: PasswordInfo,
}

impl<S: StorageEngine> ChangePasswordExecutor<S> {
    /// 创建新的 ChangePasswordExecutor
    pub fn new(id: i64, storage: Arc<Mutex<S>>, password_info: PasswordInfo) -> Self {
        Self {
            base: BaseExecutor::new(id, "ChangePasswordExecutor".to_string(), storage),
            password_info,
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for ChangePasswordExecutor<S> {
    async fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let storage_guard = storage.lock().map_err(|e| {
            crate::core::error::DBError::StorageError(format!("Storage lock poisoned: {}", e))
        })?;

        let result = storage_guard.change_password(&self.password_info);

        match result {
            Ok(true) => Ok(ExecutionResult::Success),
            Ok(false) => Ok(ExecutionResult::Error(format!("Invalid old password or user not found"))),
            Err(e) => Ok(ExecutionResult::Error(format!("Failed to change password: {}", e))),
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
        "ChangePasswordExecutor"
    }

    fn description(&self) -> &str {
        "Changes user password"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}
