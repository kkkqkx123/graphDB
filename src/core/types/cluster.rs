//! Definition of cluster information type

use oxicode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct ClusterInfo {
    pub cluster_id: i32,
    pub nodes: Vec<String>,
    pub total_space: i64,
    pub used_space: i64,
}
