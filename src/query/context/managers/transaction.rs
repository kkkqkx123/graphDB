//! 事务管理 - 提供基本的事务支持

use crate::core::error::{ManagerError, ManagerResult};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 事务ID类型
pub type TransactionId = i64;

/// 事务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    Active,
    Committed,
    Aborted,
}

/// 事务隔离级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

impl Default for IsolationLevel {
    fn default() -> Self {
        IsolationLevel::ReadCommitted
    }
}

/// 事务信息
#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: TransactionId,
    pub state: TransactionState,
    pub isolation_level: IsolationLevel,
    pub start_time: i64,
    pub operations: Vec<TransactionOperation>,
}

/// 事务操作类型
#[derive(Debug, Clone)]
pub enum TransactionOperation {
    CreateTag {
        space_id: i32,
        tag_id: i32,
        tag_name: String,
    },
    DropTag {
        space_id: i32,
        tag_id: i32,
    },
    CreateEdgeType {
        space_id: i32,
        edge_type_id: i32,
        edge_type_name: String,
    },
    DropEdgeType {
        space_id: i32,
        edge_type_id: i32,
    },
    CreateIndex {
        space_id: i32,
        index_name: String,
    },
    DropIndex {
        space_id: i32,
        index_name: String,
    },
}

/// 事务管理器
#[derive(Debug, Clone)]
pub struct TransactionManager {
    transactions: Arc<RwLock<HashMap<TransactionId, Transaction>>>,
    next_transaction_id: Arc<RwLock<TransactionId>>,
}

impl TransactionManager {
    pub fn new() -> Self {
        Self {
            transactions: Arc::new(RwLock::new(HashMap::new())),
            next_transaction_id: Arc::new(RwLock::new(1)),
        }
    }

    /// 开始新事务
    pub fn begin_transaction(
        &self,
        isolation_level: Option<IsolationLevel>,
    ) -> ManagerResult<TransactionId> {
        let mut next_id = self
            .next_transaction_id
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let transaction_id = *next_id;
        *next_id += 1;
        drop(next_id);

        let start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| ManagerError::Other(e.to_string()))?
            .as_secs() as i64;

        let transaction = Transaction {
            id: transaction_id,
            state: TransactionState::Active,
            isolation_level: isolation_level.unwrap_or_default(),
            start_time,
            operations: Vec::new(),
        };

        let mut transactions = self
            .transactions
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        transactions.insert(transaction_id, transaction);

        Ok(transaction_id)
    }

    /// 提交事务
    pub fn commit_transaction(&self, transaction_id: TransactionId) -> ManagerResult<()> {
        let mut transactions = self
            .transactions
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        let transaction = transactions.get_mut(&transaction_id).ok_or_else(|| {
            ManagerError::TransactionError(format!("事务 {} 不存在", transaction_id))
        })?;

        if transaction.state != TransactionState::Active {
            return Err(ManagerError::TransactionError(format!(
                "事务 {} 已处于 {:?} 状态，无法提交",
                transaction_id, transaction.state
            )));
        }

        transaction.state = TransactionState::Committed;
        Ok(())
    }

    /// 回滚事务
    pub fn rollback_transaction(&self, transaction_id: TransactionId) -> ManagerResult<()> {
        let mut transactions = self
            .transactions
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        let transaction = transactions.get_mut(&transaction_id).ok_or_else(|| {
            ManagerError::TransactionError(format!("事务 {} 不存在", transaction_id))
        })?;

        if transaction.state != TransactionState::Active {
            return Err(ManagerError::TransactionError(format!(
                "事务 {} 已处于 {:?} 状态，无法回滚",
                transaction_id, transaction.state
            )));
        }

        transaction.state = TransactionState::Aborted;
        Ok(())
    }

    /// 获取事务信息
    pub fn get_transaction(&self, transaction_id: TransactionId) -> ManagerResult<Transaction> {
        let transactions = self
            .transactions
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        transactions.get(&transaction_id).cloned().ok_or_else(|| {
            ManagerError::TransactionError(format!("事务 {} 不存在", transaction_id))
        })
    }

    /// 检查事务是否存在
    pub fn has_transaction(&self, transaction_id: TransactionId) -> bool {
        match self.transactions.read() {
            Ok(transactions) => transactions.contains_key(&transaction_id),
            Err(_) => false,
        }
    }

    /// 记录事务操作
    pub fn record_operation(
        &self,
        transaction_id: TransactionId,
        operation: TransactionOperation,
    ) -> ManagerResult<()> {
        let mut transactions = self
            .transactions
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        let transaction = transactions.get_mut(&transaction_id).ok_or_else(|| {
            ManagerError::TransactionError(format!("事务 {} 不存在", transaction_id))
        })?;

        if transaction.state != TransactionState::Active {
            return Err(ManagerError::TransactionError(format!(
                "事务 {} 已处于 {:?} 状态，无法记录操作",
                transaction_id, transaction.state
            )));
        }

        transaction.operations.push(operation);
        Ok(())
    }

    /// 获取事务操作列表
    pub fn get_operations(
        &self,
        transaction_id: TransactionId,
    ) -> ManagerResult<Vec<TransactionOperation>> {
        let transactions = self
            .transactions
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        let transaction = transactions.get(&transaction_id).ok_or_else(|| {
            ManagerError::TransactionError(format!("事务 {} 不存在", transaction_id))
        })?;

        Ok(transaction.operations.clone())
    }

    /// 清理已完成的事务
    pub fn cleanup_transactions(&self) -> ManagerResult<usize> {
        let mut transactions = self
            .transactions
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        let before_count = transactions.len();
        transactions.retain(|_, t| t.state == TransactionState::Active);
        let after_count = transactions.len();

        Ok(before_count - after_count)
    }

    /// 获取活动事务数量
    pub fn active_transaction_count(&self) -> ManagerResult<usize> {
        let transactions = self
            .transactions
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        Ok(transactions
            .values()
            .filter(|t| t.state == TransactionState::Active)
            .count())
    }
}

impl Default for TransactionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_begin_transaction() {
        let manager = TransactionManager::new();
        let transaction_id = manager
            .begin_transaction(None)
            .expect("Failed to begin transaction");
        assert!(transaction_id > 0);
        assert!(manager.has_transaction(transaction_id));
    }

    #[test]
    fn test_commit_transaction() {
        let manager = TransactionManager::new();
        let transaction_id = manager
            .begin_transaction(None)
            .expect("Failed to begin transaction");

        manager
            .commit_transaction(transaction_id)
            .expect("Failed to commit transaction");

        let transaction = manager
            .get_transaction(transaction_id)
            .expect("Failed to get transaction");
        assert_eq!(transaction.state, TransactionState::Committed);
    }

    #[test]
    fn test_rollback_transaction() {
        let manager = TransactionManager::new();
        let transaction_id = manager
            .begin_transaction(None)
            .expect("Failed to begin transaction");

        manager
            .rollback_transaction(transaction_id)
            .expect("Failed to rollback transaction");

        let transaction = manager
            .get_transaction(transaction_id)
            .expect("Failed to get transaction");
        assert_eq!(transaction.state, TransactionState::Aborted);
    }

    #[test]
    fn test_record_operation() {
        let manager = TransactionManager::new();
        let transaction_id = manager
            .begin_transaction(None)
            .expect("Failed to begin transaction");

        let operation = TransactionOperation::CreateTag {
            space_id: 1,
            tag_id: 1,
            tag_name: "person".to_string(),
        };

        manager
            .record_operation(transaction_id, operation)
            .expect("Failed to record operation");

        let operations = manager
            .get_operations(transaction_id)
            .expect("Failed to get operations");
        assert_eq!(operations.len(), 1);
    }

    #[test]
    fn test_active_transaction_count() {
        let manager = TransactionManager::new();

        let tx1 = manager
            .begin_transaction(None)
            .expect("Failed to begin transaction 1");
        let tx2 = manager
            .begin_transaction(None)
            .expect("Failed to begin transaction 2");

        let count = manager
            .active_transaction_count()
            .expect("Failed to get active transaction count");
        assert_eq!(count, 2);

        manager
            .commit_transaction(tx1)
            .expect("Failed to commit transaction 1");

        let count_after = manager
            .active_transaction_count()
            .expect("Failed to get active transaction count");
        assert_eq!(count_after, 1);
    }

    #[test]
    fn test_cleanup_transactions() {
        let manager = TransactionManager::new();

        let tx1 = manager
            .begin_transaction(None)
            .expect("Failed to begin transaction 1");
        let tx2 = manager
            .begin_transaction(None)
            .expect("Failed to begin transaction 2");

        manager
            .commit_transaction(tx1)
            .expect("Failed to commit transaction 1");
        manager
            .rollback_transaction(tx2)
            .expect("Failed to rollback transaction 2");

        let cleaned = manager
            .cleanup_transactions()
            .expect("Failed to cleanup transactions");
        assert_eq!(cleaned, 2);

        let count = manager
            .active_transaction_count()
            .expect("Failed to get active transaction count");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_commit_nonexistent_transaction() {
        let manager = TransactionManager::new();
        let result = manager.commit_transaction(999);
        assert!(result.is_err());
    }

    #[test]
    fn test_rollback_nonexistent_transaction() {
        let manager = TransactionManager::new();
        let result = manager.rollback_transaction(999);
        assert!(result.is_err());
    }

    #[test]
    fn test_commit_committed_transaction() {
        let manager = TransactionManager::new();
        let transaction_id = manager
            .begin_transaction(None)
            .expect("Failed to begin transaction");

        manager
            .commit_transaction(transaction_id)
            .expect("Failed to commit transaction");

        let result = manager.commit_transaction(transaction_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_record_operation_on_committed_transaction() {
        let manager = TransactionManager::new();
        let transaction_id = manager
            .begin_transaction(None)
            .expect("Failed to begin transaction");

        manager
            .commit_transaction(transaction_id)
            .expect("Failed to commit transaction");

        let operation = TransactionOperation::CreateTag {
            space_id: 1,
            tag_id: 1,
            tag_name: "person".to_string(),
        };

        let result = manager.record_operation(transaction_id, operation);
        assert!(result.is_err());
    }
}
