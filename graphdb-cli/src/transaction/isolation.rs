use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

impl IsolationLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            IsolationLevel::ReadUncommitted => "READ UNCOMMITTED",
            IsolationLevel::ReadCommitted => "READ COMMITTED",
            IsolationLevel::RepeatableRead => "REPEATABLE READ",
            IsolationLevel::Serializable => "SERIALIZABLE",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().replace('_', " ").as_str() {
            "READ UNCOMMITTED" => Some(IsolationLevel::ReadUncommitted),
            "READ COMMITTED" => Some(IsolationLevel::ReadCommitted),
            "REPEATABLE READ" => Some(IsolationLevel::RepeatableRead),
            "SERIALIZABLE" => Some(IsolationLevel::Serializable),
            _ => None,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            IsolationLevel::ReadUncommitted => "May read uncommitted data (dirty reads)",
            IsolationLevel::ReadCommitted => "Only read committed data",
            IsolationLevel::RepeatableRead => "Consistent reads within transaction",
            IsolationLevel::Serializable => "Full isolation, transactions appear sequential",
        }
    }
}

impl Default for IsolationLevel {
    fn default() -> Self {
        IsolationLevel::ReadCommitted
    }
}

impl std::fmt::Display for IsolationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
