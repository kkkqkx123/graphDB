//! 事务管理模块
//!
//! 提供GraphDB的事务管理功能，包括：
//! - 事务生命周期管理（开始、提交、中止）
//! - 保存点管理（创建、回滚、释放）
//! - 两阶段提交（2PC）支持
//! - 事务统计与监控
//!
//! ## 使用示例
//!
//! ```rust
//! use graphdb::transaction::{TransactionManager, TransactionOptions};
//!
//! // 创建事务管理器
//! let manager = TransactionManager::new(db, Default::default());
//!
//! // 开始事务
//! let txn_id = manager.begin_transaction(TransactionOptions::default())?;
//!
//! // 执行操作...
//!
//! // 提交事务
//! manager.commit_transaction(txn_id)?;
//! ```

pub mod context;
pub mod manager;
pub mod types;
pub mod savepoint;
pub mod two_phase;

pub use context::TransactionContext;
pub use manager::TransactionManager;
pub use savepoint::{Savepoint, SavepointId, SavepointInfo, SavepointManager, SavepointState, SavepointStats};
pub use two_phase::{
    ParticipantState, ParticipantVote, ResourceManager, TwoPhaseCoordinator,
    TwoPhaseId, TwoPhaseState, TwoPhaseTransaction,
};
pub use types::*;

/// 事务管理模块版本
pub const VERSION: &str = "1.0.0";

/// 创建默认配置的事务管理器
pub fn create_transaction_manager(
    db: std::sync::Arc<redb::Database>,
) -> TransactionManager {
    TransactionManager::new(db, TransactionManagerConfig::default())
}

/// 创建只读事务选项
pub fn readonly_options() -> TransactionOptions {
    TransactionOptions::new().read_only()
}

/// 创建高性能写事务选项（不保证立即持久化）
pub fn high_performance_write_options() -> TransactionOptions {
    TransactionOptions::new()
        .with_durability(DurabilityLevel::None)
}

/// 创建安全写事务选项（两阶段提交）
pub fn safe_write_options() -> TransactionOptions {
    TransactionOptions::new()
        .with_durability(DurabilityLevel::Immediate)
        .with_two_phase_commit()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn create_test_db() -> (Arc<redb::Database>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let db = Arc::new(redb::Database::create(temp_dir.path().join("test.db")).expect("Failed to create test database"));
        (db, temp_dir)
    }

    #[test]
    fn test_module_version() {
        assert_eq!(VERSION, "1.0.0");
    }

    #[test]
    fn test_create_transaction_manager() {
        let (db, _temp) = create_test_db();
        let manager = create_transaction_manager(db);

        let txn_id = manager.begin_transaction(TransactionOptions::default())
            .expect("Failed to begin transaction");

        manager.commit_transaction(txn_id)
            .expect("Failed to commit transaction");
    }

    #[test]
    fn test_readonly_options() {
        let (db, _temp) = create_test_db();
        let manager = create_transaction_manager(db);

        let options = readonly_options();
        let txn_id = manager.begin_transaction(options).expect("Failed to begin readonly transaction");

        let ctx = manager.get_context(txn_id).expect("Failed to get transaction context");
        assert!(ctx.read_only);

        manager.commit_transaction(txn_id).expect("Failed to commit transaction");
    }

    #[test]
    fn test_high_performance_options() {
        let options = high_performance_write_options();
        assert_eq!(options.durability, DurabilityLevel::None);
        assert!(!options.two_phase_commit);
    }

    #[test]
    fn test_safe_write_options() {
        let options = safe_write_options();
        assert_eq!(options.durability, DurabilityLevel::Immediate);
        assert!(options.two_phase_commit);
    }
}
