use crate::storage::vertex::{LabelId, Timestamp, VertexTable};
use crate::storage::vertex::vertex_table::VertexIterator;
use std::collections::HashMap;

pub struct QueryOps;

impl QueryOps {
    pub fn scan_vertices<'a>(
        vertex_tables: &'a HashMap<LabelId, VertexTable>,
        label: LabelId,
        ts: Timestamp,
    ) -> Option<VertexIterator<'a>> {
        vertex_tables.get(&label).map(|t| t.scan(ts))
    }

    pub fn vertex_count(
        vertex_tables: &HashMap<LabelId, VertexTable>,
        label: LabelId,
        ts: Timestamp,
    ) -> usize {
        vertex_tables
            .get(&label)
            .map(|t| t.vertex_count(ts))
            .unwrap_or(0)
    }
}
