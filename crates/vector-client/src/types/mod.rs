mod config;
mod filter;
mod point;
mod search;

pub use config::*;
pub use filter::*;
pub use point::*;
pub use search::*;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub type Payload = HashMap<String, serde_json::Value>;
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PointId {
    Num(u64),
    Uuid(String),
}

impl std::fmt::Display for PointId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PointId::Num(n) => write!(f, "{}", n),
            PointId::Uuid(s) => write!(f, "{}", s),
        }
    }
}

impl From<u64> for PointId {
    fn from(n: u64) -> Self {
        PointId::Num(n)
    }
}

impl From<String> for PointId {
    fn from(s: String) -> Self {
        if let Ok(n) = s.parse::<u64>() {
            PointId::Num(n)
        } else {
            PointId::Uuid(s)
        }
    }
}

impl From<&str> for PointId {
    fn from(s: &str) -> Self {
        PointId::from(s.to_string())
    }
}

pub type CollectionName = String;
