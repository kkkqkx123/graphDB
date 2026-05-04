//! Edge Iterator - provides lazy iteration over edge records
//!
//! Offers:
//! - EdgeTableIterator: Lazy iterator over edges in an EdgeTable
//! - PropertyGraphEdgeIterator: Iterator over all edges in a PropertyGraph

use crate::storage::edge::{EdgeRecord, EdgeTable, EdgeId, VertexId, Timestamp, Nbr, EdgeSchema, PropertyDef};

pub struct EdgeTableIterator<'a> {
    table: &'a EdgeTable,
    ts: Timestamp,
    current_vertex: usize,
    current_nbr_idx: usize,
    current_nbrs: Vec<Nbr>,
}

impl<'a> EdgeTableIterator<'a> {
    pub fn new(table: &'a EdgeTable, ts: Timestamp) -> Self {
        let vertex_capacity = table.vertex_capacity();
        Self {
            table,
            ts,
            current_vertex: 0,
            current_nbr_idx: 0,
            current_nbrs: Vec::new(),
        }
    }
}

impl<'a> Iterator for EdgeTableIterator<'a> {
    type Item = EdgeRecord;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current_nbr_idx < self.current_nbrs.len() {
                let nbr = &self.current_nbrs[self.current_nbr_idx];
                self.current_nbr_idx += 1;

                let properties = if nbr.prop_offset > 0 {
                    self.table.get_properties(nbr.prop_offset)
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };

                return Some(EdgeRecord {
                    edge_id: nbr.edge_id,
                    src_vid: self.current_vertex as VertexId - 1,
                    dst_vid: nbr.neighbor,
                    properties,
                });
            }

            if self.current_vertex >= self.table.vertex_capacity() {
                return None;
            }

            let src = self.current_vertex as VertexId;
            self.current_vertex += 1;
            self.current_nbrs = self.table.edges_of(src, self.ts);
            self.current_nbr_idx = 0;
        }
    }
}

pub struct EdgeTableRangeIterator<'a> {
    table: &'a EdgeTable,
    ts: Timestamp,
    src_vertices: Vec<VertexId>,
    current_vertex_idx: usize,
    current_nbr_idx: usize,
    current_nbrs: Vec<Nbr>,
}

impl<'a> EdgeTableRangeIterator<'a> {
    pub fn new(table: &'a EdgeTable, src_vertices: Vec<VertexId>, ts: Timestamp) -> Self {
        Self {
            table,
            ts,
            src_vertices,
            current_vertex_idx: 0,
            current_nbr_idx: 0,
            current_nbrs: Vec::new(),
        }
    }
}

impl<'a> Iterator for EdgeTableRangeIterator<'a> {
    type Item = EdgeRecord;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current_nbr_idx < self.current_nbrs.len() {
                let nbr = &self.current_nbrs[self.current_nbr_idx];
                self.current_nbr_idx += 1;

                let properties = if nbr.prop_offset > 0 {
                    self.table.get_properties(nbr.prop_offset)
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };

                let src_vid = self.src_vertices[self.current_vertex_idx - 1];
                return Some(EdgeRecord {
                    edge_id: nbr.edge_id,
                    src_vid,
                    dst_vid: nbr.neighbor,
                    properties,
                });
            }

            if self.current_vertex_idx >= self.src_vertices.len() {
                return None;
            }

            let src = self.src_vertices[self.current_vertex_idx];
            self.current_vertex_idx += 1;
            self.current_nbrs = self.table.edges_of(src, self.ts);
            self.current_nbr_idx = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::DataType;

    fn create_test_schema() -> EdgeSchema {
        EdgeSchema {
            label_id: 0,
            label_name: "knows".to_string(),
            src_label: 1,
            dst_label: 2,
            properties: vec![
                PropertyDef::new("weight".to_string(), DataType::Double),
            ],
            oe_strategy: crate::storage::edge::EdgeStrategy::Multiple,
            ie_strategy: crate::storage::edge::EdgeStrategy::Multiple,
        }
    }

    fn create_test_table() -> EdgeTable {
        let schema = create_test_schema();
        let mut table = EdgeTable::new(schema);

        let ts = 100u32;
        table.insert_edge(1, 2, &[], ts).unwrap();
        table.insert_edge(1, 3, &[], ts).unwrap();
        table.insert_edge(2, 3, &[], ts).unwrap();

        table
    }

    #[test]
    fn test_edge_table_iterator() {
        let table = create_test_table();
        let iter = EdgeTableIterator::new(&table, 100);
        let edges: Vec<_> = iter.collect();

        assert_eq!(edges.len(), 3);
    }

    #[test]
    fn test_edge_table_range_iterator() {
        let table = create_test_table();
        let iter = EdgeTableRangeIterator::new(&table, vec![1, 2], 100);
        let edges: Vec<_> = iter.collect();

        assert_eq!(edges.len(), 3);
    }
}
