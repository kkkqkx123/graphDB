//! 事务管理 API - 核心层
//!
//! 提供与传输层无关的事务管理功能

use crate::transaction::{TransactionManager, TransactionOptions, TransactionId};
use crate::api::core::{CoreResult, CoreError, TransactionHandle, SavepointId};
use std::sync::Arc;

/// 通用事务 API - 核心层
pub struct TransactionApi {
    txn_manager: Arc<TransactionManager>,
}

impl TransactionApi {
    /// 创建新的事务 API 实例
    pub fn new(txn_manager: Arc<TransactionManager>) -> Self {
        Self { txn_manager }
    }

    /// 开始事务
    ///
    /// # 参数
    /// - `options`: 事务选项
    ///
    /// # 返回
    /// 事务句柄
    pub fn begin(&self, options: TransactionOptions) -> CoreResult<TransactionHandle> {
        let txn_id = self
            .txn_manager
            .begin_transaction(options)
            .map_err(|e| CoreError::TransactionFailed(e.to_string()))?;
        Ok(TransactionHandle(txn_id))
    }

    /// 提交事务
    ///
    /// # 参数
    /// - `handle`: 事务句柄
    pub fn commit(&self, handle: TransactionHandle) -> CoreResult<()> {
        self.txn_manager
            .commit_transaction(handle.0)
            .map_err(|e| CoreError::TransactionFailed(e.to_string()))
    }

    /// 回滚（中止）事务
    ///
    /// # 参数
    /// - `handle`: 事务句柄
    pub fn rollback(&self, handle: TransactionHandle) -> CoreResult<()> {
        self.txn_manager
            .abort_transaction(handle.0)
            .map_err(|e| CoreError::TransactionFailed(e.to_string()))
    }

    /// 获取事务状态
    ///
    /// # 参数
    /// - `handle`: 事务句柄
    ///
    /// # 返回
    /// 事务状态字符串
    pub fn get_status(&self, _handle: TransactionHandle) -> CoreResult<String> {
        // 暂时返回 Active，实际需要查询事务状态
        Ok("Active".to_string())
    }

    /// 检查事务是否存在且活跃
    ///
    /// # 参数
    /// - `handle`: 事务句柄
    pub fn is_active(&self, handle: TransactionHandle) -> bool {
        self.txn_manager.is_transaction_active(handle.0)
    }

    /// 获取活跃事务数量
    pub fn active_count(&self) -> usize {
        // 暂时返回 0，实际需要查询
        0
    }
}

impl Clone for TransactionApi {
    fn clone(&self) -> Self {
        Self {
            txn_manager: Arc::clone(&self.txn_manager),
        }
    }
}
