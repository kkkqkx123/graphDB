use crate::core::types::TransactionId;

#[derive(Debug, Clone)]
pub struct TransactionContextInfo {
    pub id: TransactionId,
    pub timestamp: u32,
    pub is_read_only: bool,
}

impl TransactionContextInfo {
    pub fn new(id: TransactionId, timestamp: u32, is_read_only: bool) -> Self {
        Self {
            id,
            timestamp,
            is_read_only,
        }
    }
}
