mod config;
pub mod distance_utils;
mod filter;
mod point;
mod search;

pub use config::*;
pub use filter::*;
pub use point::*;
pub use search::*;

use std::collections::HashMap;

pub type Payload = HashMap<String, serde_json::Value>;
pub type PointId = String;
pub type CollectionName = String;
