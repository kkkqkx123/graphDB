#[derive(Debug, Clone)]
pub struct TransactionContextInfo {
    pub id: u64,
    pub timestamp: u32,
    pub is_read_only: bool,
}

impl TransactionContextInfo {
    pub fn new(id: u64, timestamp: u32, is_read_only: bool) -> Self {
        Self {
            id,
            timestamp,
            is_read_only,
        }
    }
}
