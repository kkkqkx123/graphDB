//! Transaction Configuration
//!
//! Provides configuration options for transaction support.

use std::time::Duration;

/// WAL synchronization mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalSyncMode {
    /// Sync after every write (safest, slowest)
    Sync,
    /// Sync every N milliseconds
    Periodic(u64),
    /// Async, rely on OS (fastest, least safe)
    Async,
}

impl Default for WalSyncMode {
    fn default() -> Self {
        Self::Periodic(100)
    }
}

/// Transaction isolation level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IsolationLevel {
    /// Read uncommitted - can see uncommitted changes
    ReadUncommitted,
    /// Read committed - only see committed changes
    ReadCommitted,
    /// Snapshot isolation - see a consistent snapshot
    #[default]
    SnapshotIsolation,
    /// Serializable - full ACID guarantees
    Serializable,
}

/// Durability level for transactions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DurabilityLevel {
    /// No durability - data lost on crash
    None,
    /// Async WAL - may lose recent transactions on crash
    #[default]
    Async,
    /// Sync WAL - guaranteed durability
    Sync,
}

/// Transaction configuration
#[derive(Debug, Clone)]
pub struct TransactionConfig {
    /// Whether to enable transaction support
    pub enable_transactions: bool,
    
    /// Whether to enable WAL (Write-Ahead Logging)
    pub enable_wal: bool,
    
    /// Whether to enable crash recovery
    pub enable_recovery: bool,
    
    /// Whether to enable undo log for rollback
    pub enable_undo_log: bool,
    
    /// Transaction isolation level
    pub isolation_level: IsolationLevel,
    
    /// Durability level
    pub durability: DurabilityLevel,
    
    /// WAL sync mode
    pub wal_sync_mode: WalSyncMode,
    
    /// Transaction timeout
    pub transaction_timeout: Duration,
    
    /// Lock timeout
    pub lock_timeout: Duration,
    
    /// Maximum number of concurrent transactions
    pub max_concurrent_transactions: usize,
    
    /// Auto-commit threshold (number of operations before auto-commit)
    pub auto_commit_threshold: usize,
}

impl Default for TransactionConfig {
    fn default() -> Self {
        Self {
            enable_transactions: true,
            enable_wal: true,
            enable_recovery: true,
            enable_undo_log: true,
            isolation_level: IsolationLevel::SnapshotIsolation,
            durability: DurabilityLevel::Async,
            wal_sync_mode: WalSyncMode::default(),
            transaction_timeout: Duration::from_secs(30),
            lock_timeout: Duration::from_secs(10),
            max_concurrent_transactions: 1000,
            auto_commit_threshold: 1000,
        }
    }
}

impl TransactionConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a configuration for development (fast, less safe)
    pub fn development() -> Self {
        Self {
            enable_transactions: true,
            enable_wal: false,
            enable_recovery: false,
            enable_undo_log: true,
            isolation_level: IsolationLevel::ReadCommitted,
            durability: DurabilityLevel::None,
            wal_sync_mode: WalSyncMode::Async,
            transaction_timeout: Duration::from_secs(60),
            lock_timeout: Duration::from_secs(30),
            max_concurrent_transactions: 100,
            auto_commit_threshold: 100,
        }
    }

    /// Create a configuration for production (safe, balanced)
    pub fn production() -> Self {
        Self {
            enable_transactions: true,
            enable_wal: true,
            enable_recovery: true,
            enable_undo_log: true,
            isolation_level: IsolationLevel::SnapshotIsolation,
            durability: DurabilityLevel::Async,
            wal_sync_mode: WalSyncMode::Periodic(100),
            transaction_timeout: Duration::from_secs(30),
            lock_timeout: Duration::from_secs(10),
            max_concurrent_transactions: 1000,
            auto_commit_threshold: 1000,
        }
    }

    /// Create a configuration for testing (fast, no persistence)
    pub fn testing() -> Self {
        Self {
            enable_transactions: false,
            enable_wal: false,
            enable_recovery: false,
            enable_undo_log: false,
            isolation_level: IsolationLevel::ReadUncommitted,
            durability: DurabilityLevel::None,
            wal_sync_mode: WalSyncMode::Async,
            transaction_timeout: Duration::from_secs(5),
            lock_timeout: Duration::from_secs(1),
            max_concurrent_transactions: 10,
            auto_commit_threshold: 10,
        }
    }

    /// Check if transactions are enabled
    pub fn is_transactions_enabled(&self) -> bool {
        self.enable_transactions
    }

    /// Check if WAL is enabled
    pub fn is_wal_enabled(&self) -> bool {
        self.enable_wal
    }

    /// Check if recovery is enabled
    pub fn is_recovery_enabled(&self) -> bool {
        self.enable_recovery
    }

    /// Check if undo log is enabled
    pub fn is_undo_log_enabled(&self) -> bool {
        self.enable_undo_log
    }

    /// Builder pattern: set enable_transactions
    pub fn with_transactions(mut self, enable: bool) -> Self {
        self.enable_transactions = enable;
        self
    }

    /// Builder pattern: set enable_wal
    pub fn with_wal(mut self, enable: bool) -> Self {
        self.enable_wal = enable;
        self
    }

    /// Builder pattern: set enable_recovery
    pub fn with_recovery(mut self, enable: bool) -> Self {
        self.enable_recovery = enable;
        self
    }

    /// Builder pattern: set isolation_level
    pub fn with_isolation_level(mut self, level: IsolationLevel) -> Self {
        self.isolation_level = level;
        self
    }

    /// Builder pattern: set durability
    pub fn with_durability(mut self, durability: DurabilityLevel) -> Self {
        self.durability = durability;
        self
    }
}
