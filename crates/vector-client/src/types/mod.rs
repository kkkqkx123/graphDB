mod point;
mod search;
mod config;
mod filter;

pub use point::*;
pub use search::*;
pub use config::*;
pub use filter::*;

use std::collections::HashMap;

pub type Payload = HashMap<String, serde_json::Value>;
pub type PointId = String;
pub type CollectionName = String;
