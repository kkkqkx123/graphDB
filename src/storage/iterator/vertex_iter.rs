//! Vertex Iterator - provides lazy iteration over vertex records
//!
//! Offers:
//! - VertexScanIterator: Iterator over all vertices in a PropertyGraph
//! - VertexRangeIterator: Iterator over a range of vertices
//! - VertexFilterIterator: Iterator with predicate pushdown support

use crate::core::types::{Timestamp, VertexId};
use crate::storage::iterator::predicate::PredicateEnum;
use crate::storage::vertex::{VertexRecord, VertexTable};
use std::collections::HashMap;

pub type VertexTableScanIterator<'a> = crate::storage::vertex::vertex_table::VertexIterator<'a>;

pub struct VertexScanIterator<'a> {
    tables: Vec<(&'a u16, &'a VertexTable)>,
    current_table_idx: usize,
    current_iter: Option<VertexTableScanIterator<'a>>,
    current_label: u16,
    ts: Timestamp,
}

impl<'a> VertexScanIterator<'a> {
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

impl<'a> Iterator for VertexScanIterator<'a> {
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

pub struct VertexRangeIterator<'a> {
    table: &'a VertexTable,
    ts: Timestamp,
    vids: Vec<VertexId>,
    current_idx: usize,
}

impl<'a> VertexRangeIterator<'a> {
    pub fn new(table: &'a VertexTable, vids: Vec<VertexId>, ts: Timestamp) -> Self {
        Self {
            table,
            ts,
            vids,
            current_idx: 0,
        }
    }
}

impl<'a> Iterator for VertexRangeIterator<'a> {
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

pub struct VertexFilterIterator<'a> {
    iter: VertexTableScanIterator<'a>,
    predicate: PredicateEnum,
}

impl<'a> VertexFilterIterator<'a> {
    pub fn new(table: &'a VertexTable, ts: Timestamp, predicate: PredicateEnum) -> Self {
        Self {
            iter: table.scan(ts),
            predicate,
        }
    }
}

impl<'a> Iterator for VertexFilterIterator<'a> {
    type Item = VertexRecord;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let record = self.iter.next()?;
            let row: Vec<crate::core::Value> = record
                .properties
                .iter()
                .map(|(_, v)| v.clone())
                .collect();

            if self.predicate.evaluate(&row) {
                return Some(record);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::DataType;
    use crate::core::Value;
    use crate::storage::vertex::{PropertyDef, VertexSchema};

    fn create_test_table() -> VertexTable {
        let schema = VertexSchema {
            label_id: 0,
            label_name: "person".to_string(),
            properties: vec![
                PropertyDef::new("name".to_string(), DataType::String),
                PropertyDef::new("age".to_string(), DataType::Int),
            ],
            primary_key_index: 0,
        };

        let mut table = VertexTable::new(0, "person".to_string(), schema);

        let ts = 100u32;
        table
            .insert(
                "1",
                &[
                    ("name".to_string(), Value::String("Alice".to_string())),
                    ("age".to_string(), Value::Int(25)),
                ],
                ts,
            )
            .unwrap();
        table
            .insert(
                "2",
                &[
                    ("name".to_string(), Value::String("Bob".to_string())),
                    ("age".to_string(), Value::Int(30)),
                ],
                ts,
            )
            .unwrap();
        table
            .insert(
                "3",
                &[
                    ("name".to_string(), Value::String("Charlie".to_string())),
                    ("age".to_string(), Value::Int(35)),
                ],
                ts,
            )
            .unwrap();

        table
    }

    #[test]
    fn test_vertex_range_iterator() {
        let table = create_test_table();
        let iter = VertexRangeIterator::new(&table, vec![VertexId::from_int64(1), VertexId::from_int64(2)], 100);
        let vertices: Vec<_> = iter.collect();

        assert_eq!(vertices.len(), 2);
    }

    #[test]
    fn test_vertex_filter_iterator() {
        use crate::storage::iterator::predicate::CompareOp;

        let table = create_test_table();
        let predicate = PredicateEnum::simple("1", CompareOp::Greater, Value::Int(28));
        let iter = VertexFilterIterator::new(&table, 100, predicate);
        let vertices: Vec<_> = iter.collect();

        assert_eq!(vertices.len(), 2);
    }

    #[test]
    fn test_vertex_scan_iterator() {
        let table = create_test_table();
        let mut tables = HashMap::new();
        tables.insert(0u16, table);

        let iter = VertexScanIterator::new(&tables, 100);
        let vertices: Vec<_> = iter.collect();

        assert_eq!(vertices.len(), 3);
    }
}
