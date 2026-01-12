use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Schema definition for node labels and edge types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub node_labels: BTreeSet<String>,
    pub edge_types: BTreeSet<String>,
    pub property_keys: BTreeSet<String>,
}
