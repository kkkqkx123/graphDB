//! Vertex Iterator - provides lazy iteration over vertex records
//!
//! Offers:
//! - PropertyGraphVertexIterator: Iterator over all vertices in a PropertyGraph
//! - VertexTableRangeIterator: Iterator over a range of vertices

use crate::storage::vertex::{VertexRecord, VertexTable, VertexId, Timestamp};
use std::collections::HashMap;

pub struct PropertyGraphVertexIterator<'a> {
    tables: Vec<(&'a u16, &'a VertexTable)>,
    current_table_idx: usize,
    current_iter: Option<crate::storage::vertex::vertex_table::VertexIterator<'a>>,
    current_label: u16,
    ts: Timestamp,
}

impl<'a> PropertyGraphVertexIterator<'a> {
    pub fn new(tables: &'a HashMap<u16, VertexTable>, ts: Timestamp) -> Self {
        let tables: Vec<_> = tables.iter().collect();
        let (current_label, current_iter) = if let Some((label, table)) = tables.first() {
            (**label, Some(table.scan(ts)))
        } else {
            (0, None)
        };

        Self {
            tables,
            current_table_idx: 0,
            current_iter,
            current_label,
            ts,
        }
    }
}

impl<'a> Iterator for PropertyGraphVertexIterator<'a> {
    type Item = (u16, VertexRecord);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut iter) = self.current_iter {
                if let Some(record) = iter.next() {
                    return Some((self.current_label, record));
                }
            }

            self.current_table_idx += 1;
            if self.current_table_idx < self.tables.len() {
                let (label, table) = self.tables[self.current_table_idx];
                self.current_label = *label;
                self.current_iter = Some(table.scan(self.ts));
            } else {
                return None;
            }
        }
    }
}

pub struct VertexTableRangeIterator<'a> {
    table: &'a VertexTable,
    ts: Timestamp,
    vids: Vec<VertexId>,
    current_idx: usize,
}

impl<'a> VertexTableRangeIterator<'a> {
    pub fn new(table: &'a VertexTable, vids: Vec<VertexId>, ts: Timestamp) -> Self {
        Self {
            table,
            ts,
            vids,
            current_idx: 0,
        }
    }
}

impl<'a> Iterator for VertexTableRangeIterator<'a> {
    type Item = VertexRecord;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_idx < self.vids.len() {
            let vid = self.vids[self.current_idx];
            self.current_idx += 1;

            if let Some(internal_id) = self.table.get_internal_id(&vid.to_string(), self.ts) {
                if let Some(record) = self.table.get_by_internal_id(internal_id, self.ts) {
                    return Some(record);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::vertex::{VertexSchema, PropertyDef};
    use crate::core::DataType;

    fn create_test_table() -> VertexTable {
        let schema = VertexSchema {
            label_id: 0,
            label_name: "person".to_string(),
            properties: vec![
                PropertyDef::new("name".to_string(), DataType::String),
            ],
            primary_key_index: 0,
        };

        let mut table = VertexTable::new(0, "person".to_string(), schema);

        let ts = 100u32;
        table.insert("1", &[("name".to_string(), crate::core::Value::String("Alice".to_string()))], ts).unwrap();
        table.insert("2", &[("name".to_string(), crate::core::Value::String("Bob".to_string()))], ts).unwrap();

        table
    }

    #[test]
    fn test_vertex_table_range_iterator() {
        let table = create_test_table();
        let iter = VertexTableRangeIterator::new(&table, vec![1, 2], 100);
        let vertices: Vec<_> = iter.collect();

        assert_eq!(vertices.len(), 2);
    }
}
