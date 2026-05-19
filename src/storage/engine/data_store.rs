use std::collections::HashMap;

use parking_lot::RwLock;

use crate::core::types::LabelId;
use crate::storage::edge::EdgeTable;
use crate::storage::vertex::VertexTable;

pub struct GraphDataStore {
    pub vertex_tables: RwLock<HashMap<LabelId, VertexTable>>,
    pub edge_tables: RwLock<HashMap<(LabelId, LabelId, LabelId), EdgeTable>>,
    pub vertex_label_names: RwLock<HashMap<String, LabelId>>,
    pub edge_label_names: RwLock<HashMap<String, LabelId>>,
    pub vertex_label_counter: RwLock<LabelId>,
    pub edge_label_counter: RwLock<LabelId>,
}

impl GraphDataStore {
    pub fn new() -> Self {
        Self {
            vertex_tables: RwLock::new(HashMap::new()),
            edge_tables: RwLock::new(HashMap::new()),
            vertex_label_names: RwLock::new(HashMap::new()),
            edge_label_names: RwLock::new(HashMap::new()),
            vertex_label_counter: RwLock::new(0),
            edge_label_counter: RwLock::new(0),
        }
    }
}

impl Default for GraphDataStore {
    fn default() -> Self {
        Self::new()
    }
}
