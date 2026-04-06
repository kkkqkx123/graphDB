use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoordinatorStatus {
    Initializing,
    Active,
    Degraded,
    ShuttingDown,
    Error,
}

#[derive(Debug, Clone, Default)]
pub struct CoordinatorStats {
    pub total_indexes: usize,
    pub active_indexes: usize,
    pub failed_operations: u64,
    pub total_operations: u64,
    pub queue_size: usize,
}
