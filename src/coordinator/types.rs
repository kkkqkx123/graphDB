use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoordinatorStatus {
    Initializing,
    Active,
    Degraded,
    ShuttingDown,
    Error,
}

#[derive(Debug, Clone)]
pub struct CoordinatorStats {
    pub total_indexes: usize,
    pub active_indexes: usize,
    pub failed_operations: u64,
    pub total_operations: u64,
    pub queue_size: usize,
}

impl Default for CoordinatorStats {
    fn default() -> Self {
        Self {
            total_indexes: 0,
            active_indexes: 0,
            failed_operations: 0,
            total_operations: 0,
            queue_size: 0,
        }
    }
}
