//! 事务模块 - 提供事务支持
//!
//! 提供完整的事务功能：
//! - Transaction: 事务 trait 和实现
//! - MVCC: 多版本并发控制
//! - TransactionLog: 事务日志
//! - Snapshot: 快照隔离

pub mod traits;
pub mod mvcc;
pub mod wal;
pub mod snapshot;

pub use traits::{Transaction, TransactionId, TransactionState, TransactionResult};
pub use mvcc::{MvccManager, Version, VersionVec};
pub use wal::{TransactionLog, LogRecord, LogType};
pub use snapshot::{Snapshot, IsolationLevel};